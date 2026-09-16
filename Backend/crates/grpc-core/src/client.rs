//! gRPC 客户端工厂
//!
//! 提供统一的泛型 gRPC 客户端 `GrpcClient<T>`、连接配置 `GrpcClientConfig`
//! 以及便捷的连接函数 `connect_grpc`。

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::Channel;

/// gRPC 客户端配置
#[derive(Debug, Clone)]
pub struct GrpcClientConfig {
    /// 连接超时
    pub connect_timeout: Duration,
    /// 请求超时
    pub request_timeout: Duration,
    /// 重连间隔
    pub reconnect_interval: Duration,
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            reconnect_interval: Duration::from_secs(30),
        }
    }
}

/// 泛型 gRPC 客户端包装器
///
/// 内部使用 `Arc<RwLock<Option<T>>>`，支持后台重连时直接替换内部值。
pub struct GrpcClient<T: Send + Sync + 'static> {
    inner: Arc<RwLock<Option<T>>>,
    addr: String,
    config: GrpcClientConfig,
}

impl<T: Send + Sync + 'static> Clone for GrpcClient<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            addr: self.addr.clone(),
            config: self.config.clone(),
        }
    }
}

impl<T: Send + Sync + 'static> GrpcClient<T> {
    /// 创建一个新的断开状态的客户端
    pub fn new(addr: String, config: GrpcClientConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            addr,
            config,
        }
    }

    /// 获取内部客户端引用（如果有）
    pub async fn get_inner(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.read().await.clone()
    }

    /// 设置内部客户端
    pub async fn set_inner(&self, inner: T) {
        *self.inner.write().await = Some(inner);
    }

    /// 断开连接
    pub async fn disconnect(&self) {
        *self.inner.write().await = None;
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        self.inner.read().await.is_some()
    }

    /// 获取地址
    #[must_use]
    pub fn addr(&self) -> &str {
        &self.addr
    }

    /// 获取配置
    #[must_use]
    pub const fn config(&self) -> &GrpcClientConfig {
        &self.config
    }
}

/// gRPC 服务间鉴权客户端拦截器 (审计修复 2026-08-04)
///
/// 为每个请求注入 `x-grpc-token` 请求头(取 `GRPC_AUTH_TOKEN` 环境变量)。
/// 使用命名类型而非闭包, 以便在 `GrpcClient<T>` 泛型中可命名。
#[derive(Clone)]
pub struct GrpcTokenInterceptor {
    token: Option<tonic::metadata::MetadataValue<tonic::metadata::Ascii>>,
}

impl GrpcTokenInterceptor {
    /// 从环境变量 `GRPC_AUTH_TOKEN` 创建拦截器(未配置时为 None, 请求不带 token 头)
    #[must_use]
    pub fn from_env() -> Self {
        // Secret中的值可能包含尾部CRLF, trim避免比较失败
        let token = std::env::var("GRPC_AUTH_TOKEN" )
            .ok()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .and_then(|t| tonic::metadata::MetadataValue::try_from(t).ok());
        Self { token }
    }
}

impl tonic::service::Interceptor for GrpcTokenInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        if let Some(t) = &self.token {
            req.metadata_mut().insert("x-grpc-token" , t.clone());
        }
        Ok(req)
    }
}

/// 连接 gRPC 服务
///
/// 使用给定的地址和工厂函数创建并返回 `GrpcClient<T>`。
///
/// 审计修复 (2026-08-04): 服务间鉴权——channel 统一经 `GrpcTokenInterceptor`
/// 注入 `x-grpc-token` 请求头, 服务端由 `common::grpc_auth_interceptor` 校验。
pub async fn connect_grpc<T, F>(
    addr: impl Into<String>,
    config: &GrpcClientConfig,
    factory: F,
) -> anyhow::Result<GrpcClient<T>>
where
    T: Send + Sync + 'static,
    F: FnOnce(
        tonic::service::interceptor::InterceptedService<Channel, GrpcTokenInterceptor>,
    ) -> T,
{
    let addr = addr.into();

    let channel = tonic::transport::Endpoint::from_shared(addr.clone())?
        // 审计修复 (B6-架构评审): 分别应用请求超时(per-request deadline)与连接超时。
        // 原代码仅 .timeout(connect_timeout) 把连接超时误用为请求超时, 且 request_timeout 字段完全闲置
        .timeout(config.request_timeout)
        .connect_timeout(config.connect_timeout)
        .connect()
        .await?;

    let intercepted = tonic::service::interceptor::InterceptedService::new(
        channel,
        GrpcTokenInterceptor::from_env(),
    );

    let client = factory(intercepted);
    let wrapper = GrpcClient {
        inner: Arc::new(RwLock::new(Some(client))),
        addr: addr.clone(),
        config: config.clone(),
    };

    Ok(wrapper)
}

/// 从环境变量或配置创建 gRPC 客户端地址
///
/// 环境变量格式: `{SERVICE_NAME}_GRPC_ADDR`，例如 `AUTH_SERVICE_GRPC_ADDR`
#[must_use]
pub fn create_grpc_addr(service_name: &str, default_port: u16) -> String {
    let env_key = format!(
        "{}_GRPC_ADDR" ,
        service_name.to_uppercase().replace('-', "_" )
    );
    std::env::var(&env_key).unwrap_or_else(|_| format!("http://localhost:{default_port}" ))
}
