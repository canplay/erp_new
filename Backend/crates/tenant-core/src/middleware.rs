//! Axum 中间件 - 从请求中提取租户信息
//!
//! 提供简单的函数式中间件，用于提取 `tenant_id` 并设置到上下文中。

use crate::context::{clear_tenant_context, set_tenant_context, TenantContext};
use crate::TenantId;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// 从请求头中提取租户 ID（通用版本，支持任意 body 类型）
pub fn extract_tenant_id_from_request<B>(req: &Request<B>) -> Option<TenantId> {
    // 优先从 x-tenant-id 头获取
    if let Some(value) = req.headers().get("x-tenant-id" ) {
        if let Ok(s) = value.to_str() {
            if let Ok(id) = s.parse::<i64>() {
                return Some(TenantId::new(id));
            }
        }
    }

    // 从 authorization JWT 中提取 tenant_id claim
    if let Some(auth) = req.headers().get("authorization" ) {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer " ) {
                if let Some(tenant_id) = extract_from_jwt(token) {
                    return Some(tenant_id);
                }
            }
        }
    }

    None
}

/// 从 JWT token 中解析 tenant_id claim
fn extract_from_jwt(token: &str) -> Option<TenantId> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let payload = parts[1];
    let decoded = decode_base64_url(payload).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;
    let value: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    value.get("tenant_id" )?.as_i64().map(TenantId::new)
}

/// Base64 URL 解码
fn decode_base64_url(input: &str) -> Result<Vec<u8>, ()> {
    let input = input.replace('-', "+" ).replace('_', "/" );
    let len = input.len();
    let padding = if len.is_multiple_of(4) {
        0
    } else {
        4 - len % 4
    };
    let padded = format!("{}{}" , input, "=".repeat(padding));

    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&padded)
        .map_err(|_| ())
}

/// 创建租户提取中间件层
///
/// 该中间件从请求中提取 `tenant_id` 并设置到 thread-local 上下文中。
///
/// 使用方式：
/// ```rust,ignore
/// use tenant_core::middleware::tenant_extraction_middleware;
///
/// let app = Router::new()
///     .route("/api/data" , get(handler))
///     .layer(tenant_extraction_middleware());
/// ```
#[derive(Clone)]
pub struct TenantLayer;

impl<S> Layer<S> for TenantLayer {
    type Service = TenantMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TenantMiddleware { inner }
    }
}

/// 租户提取中间件服务
#[derive(Clone)]
pub struct TenantMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for TenantMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let tenant_id = extract_tenant_id_from_request(&req);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if let Some(tid) = tenant_id {
                let ctx = TenantContext::new(tid);
                set_tenant_context(ctx);
            }

            let result = inner.call(req).await;

            if tenant_id.is_some() {
                clear_tenant_context();
            }

            result
        })
    }
}

/// 便利函数：创建租户提取中间件层
pub fn tenant_extraction_middleware() -> TenantLayer {
    TenantLayer
}

/// 强制要求租户 ID 的中间件层
///
/// 如果没有 `x-tenant-id` 头，返回 400 Bad Request。
#[derive(Clone)]
pub struct RequireTenantLayer;

impl<S> Layer<S> for RequireTenantLayer {
    type Service = RequireTenantMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireTenantMiddleware { inner }
    }
}

/// 强制要求租户 ID 的中间件
#[derive(Clone)]
pub struct RequireTenantMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for RequireTenantMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let tenant_id = extract_tenant_id_from_request(&req);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            match tenant_id {
                Some(tid) => {
                    let ctx = TenantContext::new(tid);
                    set_tenant_context(ctx);

                    let result = inner.call(req).await;
                    clear_tenant_context();
                    result
                }
                None => {
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Missing x-tenant-id header" ))
                        .unwrap_or_else(|_| Response::new(Body::from("Bad Request" )));
                    Ok(response)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tenant_id_from_header() {
        let req = Request::builder()
            .header("x-tenant-id" , "42" )
            .body(())
            .unwrap();
        let id = extract_tenant_id_from_request(&req);
        assert_eq!(id, Some(TenantId::new(42)));
    }

    #[test]
    fn test_extract_tenant_id_from_jwt() {
        let payload = r#"{"tenant_id": 123, "sub": 1}"#;
        let encoded = base64_url_encode(payload.as_bytes());
        let token = format!("header.{}.signature" , encoded);

        let req = Request::builder()
            .header("authorization" , format!("Bearer {}" , token))
            .body(())
            .unwrap();
        let id = extract_tenant_id_from_request(&req);
        assert_eq!(id, Some(TenantId::new(123)));
    }

    #[test]
    fn test_extract_tenant_id_none() {
        let req = Request::builder().body(()).unwrap();
        let id = extract_tenant_id_from_request(&req);
        assert!(id.is_none());
    }

    #[test]
    fn test_extract_tenant_id_invalid() {
        let req = Request::builder()
            .header("x-tenant-id" , "not_a_number" )
            .body(())
            .unwrap();
        let id = extract_tenant_id_from_request(&req);
        assert!(id.is_none());
    }

    fn base64_url_encode(input: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .encode(input)
            .replace('+', "-" )
            .replace('/', "_" )
            .replace('=', "" )
    }
}
