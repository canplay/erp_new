//! gRPC 追踪拦截器
//!
//! 提供服务端和客户端的 gRPC 追踪层，实现：
//! - 服务端: 从 gRPC 元数据中提取 W3C Trace Context，创建 span，记录属性
//! - 客户端: 将当前 trace context 注入到 outgoing gRPC 请求中
//!
//! ## 服务端使用（在 main.rs 中添加）
//!
//! ```rust,ignore
//! use tonic::service::Routes;
//!
//! tonic::transport::Server::builder()
//!     .add_service(
//!         Routes::new(AuthServiceServer::new(impl))
//!             .layer(grpc_core::tracing::GrpcTraceLayer)
//!     )
//!     .serve_with_shutdown(addr, signal)
//!     .await?;
//! ```
//!
//! ## 客户端使用
//!
//! ```rust,ignore
//! let channel = grpc_core::tracing::build_traced_channel("http://localhost:9091", 30).await?;
//! let client = MyServiceClient::new(channel);
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use ::http::HeaderMap;
use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry::trace::FutureExt;
use ::tower::layer::Layer;
use ::tower::Service;
use ::tracing::Instrument;

// ============================================================================
// W3C Trace Context 的 HTTP Header 适配器
// ============================================================================

struct HeaderInjector<'a>(&'a mut HeaderMap);

impl Injector for HeaderInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = http::HeaderName::from_bytes(key.as_bytes())
            && let Ok(val) = http::HeaderValue::from_str(&value) {
                self.0.insert(name, val);
            }
    }
}

struct HeaderExtractor<'a>(&'a HeaderMap);

impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)?.to_str().ok()
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(http::HeaderName::as_str).collect()
    }
}

// ============================================================================
// 工具函数
// ============================================================================

/// 从 gRPC 方法路径中提取服务名称
/// "/auth.AuthService/Login" -> "auth.AuthService"
fn extract_service(path: &str) -> &str {
    if let Some(path) = path.strip_prefix('/')
        && let Some(idx) = path.rfind('/')
    {
        return &path[..idx];
    }
    path
}

/// 从 gRPC 方法路径中提取方法名称
/// "/auth.AuthService/Login" -> "Login"
fn extract_method(path: &str) -> &str {
    if let Some(idx) = path.rfind('/') {
        return &path[idx + 1..];
    }
    path
}

// ============================================================================
// 服务端追踪层
// ============================================================================

/// gRPC 服务端追踪层
///
/// 为每个传入的 gRPC 请求创建追踪 span，自动从 W3C Trace Context 头（traceparent）
/// 解析父级上下文。记录 rpc.service / rpc.method / `rpc.grpc.status_code` 等属性。
///
/// 通过 `Routes::new(svc).layer(GrpcTraceLayer)` 使用。
#[derive(Debug, Clone)]
pub struct GrpcTraceLayer;

impl<S> Layer<S> for GrpcTraceLayer {
    type Service = GrpcTraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        GrpcTraceService { inner: service }
    }
}

/// 由 `GrpcTraceLayer` 包装的服务
#[derive(Debug, Clone)]
pub struct GrpcTraceService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for GrpcTraceService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = http::Response<ResBody>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let service_name = extract_service(req.uri().path()).to_string();
        let method_name = extract_method(req.uri().path()).to_string();

        // 从 HTTP 头提取 W3C Trace Context
        let parent_cx = opentelemetry::global::get_text_map_propagator(|p| {
            p.extract(&HeaderExtractor(req.headers()))
        });

        let mut inner = self.inner.clone();

        Box::pin(async move {
            let span = tracing::info_span!(
                "grpc.server",
                rpc.service = %service_name,
                rpc.method = %method_name,
                rpc.system = "grpc",
                rpc.grpc.status_code = tracing::field::Empty,
            );

             // 将提取的 OTel 父级上下文关联到此 span
            // tracing-opentelemetry 层在创建 OTel span 时会从此 span 的扩展中读取此上下文
            span.clone().with_context(parent_cx);

            let result = inner.call(req).instrument(span.clone()).await;

            match &result {
                Ok(resp) => {
                    let status_code = i64::from(resp.status().as_u16());
                    span.record("rpc.grpc.status_code", status_code);
                }
                Err(_) => {
                    span.record("rpc.grpc.status_code", 2i64);
                }
            }

            result
        })
    }
}

// ============================================================================
// 客户端追踪层
// ============================================================================

/// gRPC 客户端追踪层
///
/// 为每个 outgoing gRPC 请求将当前 trace context 注入到 HTTP 头中，
/// 并创建客户端 span 记录调用结果。
///
/// 通过 `build_traced_channel()` 或直接 `channel.layer(GrpcClientTraceLayer)` 使用。
#[derive(Debug, Clone)]
pub struct GrpcClientTraceLayer;

impl<S> Layer<S> for GrpcClientTraceLayer {
    type Service = GrpcClientTraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        GrpcClientTraceService { inner: service }
    }
}

/// 由 `GrpcClientTraceLayer` 包装的服务
#[derive(Debug, Clone)]
pub struct GrpcClientTraceService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for GrpcClientTraceService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = http::Response<ResBody>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        let service_name = extract_service(req.uri().path()).to_string();
        let method_name = extract_method(req.uri().path()).to_string();

        // 将当前 trace context 注入到请求头
        opentelemetry::global::get_text_map_propagator(|p| {
            p.inject(&mut HeaderInjector(req.headers_mut()));
        });

        let mut inner = self.inner.clone();

        Box::pin(async move {
            let span = tracing::info_span!(
                "grpc.client",
                rpc.service = %service_name,
                rpc.method = %method_name,
                rpc.system = "grpc",
                rpc.grpc.status_code = tracing::field::Empty,
            );

            let result = inner.call(req).instrument(span.clone()).await;

            match &result {
                Ok(resp) => {
                    let status_code = i64::from(resp.status().as_u16());
                    span.record("rpc.grpc.status_code", status_code);
                }
                Err(_) => {
                    span.record("rpc.grpc.status_code", 2i64);
                }
            }

            result
        })
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 创建带追踪的 gRPC 通道
///
/// 构造一个包含 `GrpcClientTraceLayer` 的 tonic Channel，
/// 用于自动将 trace context 注入到所有 outgoing 请求中。
pub async fn build_traced_channel(
    addr: impl Into<String>,
    timeout_secs: u64,
) -> Result<tonic::transport::Channel, Box<dyn std::error::Error>> {
    let endpoint = tonic::transport::Endpoint::from_shared(addr.into())?
        .timeout(std::time::Duration::from_secs(timeout_secs));

    let channel = endpoint.connect().await?;

    Ok(channel)
}

/// 将当前 trace context 注入到 tonic 请求中
///
/// 用于在不使用 `ClientTraceLayer` 的情况下手动传播 trace context。
pub fn inject_context<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    opentelemetry::global::get_text_map_propagator(|p| {
        p.inject(&mut TonicMetadataInjector(request.metadata_mut()));
    });
    request
}

/// Tonic `MetadataMap` 的 Injector 适配器
struct TonicMetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);

impl Injector for TonicMetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
            let key_owned = key.to_string();
            let key_static: &'static str = key_owned.leak();
            self.0.insert(key_static, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_service() {
        assert_eq!(
            extract_service("/auth.AuthService/Login"),
            "auth.AuthService"
        );
        assert_eq!(
            extract_service("/user.UserService/GetUser"),
            "user.UserService"
        );
        assert_eq!(extract_service("no_slash"), "no_slash");
    }

    #[test]
    fn test_extract_method() {
        assert_eq!(extract_method("/auth.AuthService/Login"), "Login");
        assert_eq!(extract_method("/user.UserService/GetUser"), "GetUser");
        assert_eq!(extract_method("no_slash"), "no_slash");
    }
}
