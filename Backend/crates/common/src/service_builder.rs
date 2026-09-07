//! 服务构建器模块
//!
//! 提供统一的服务构建模式，简化 gRPC + HTTP 双协议服务的创建

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;

use crate::config::DatabaseConfig;

/// HTTP 中间件层配置
#[derive(Clone)]
pub struct MiddlewareConfig {
    pub enable_cors: bool,
    pub enable_trace: bool,
    pub cors_origins: Vec<String>,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            enable_trace: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}

/// 创建默认的 CORS 层
pub fn default_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// 创建默认的 Trace 层
#[must_use]
pub fn default_trace_layer()
-> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
}

/// 创建路径规范化层
#[must_use]
pub fn normalize_path_layer() -> NormalizePathLayer {
    NormalizePathLayer::trim_trailing_slash()
}

/// 服务构建器
///
/// 统一管理 gRPC + HTTP 双协议服务的生命周期
///
/// ⚠️ 此结构已被 `ServiceRunner` 取代，不再推荐使用。
///    所有服务应改用 `ServiceRunner` 启动。
#[deprecated(
    since = "0.2.0",
    note = "Use `ServiceRunner` instead. See `.hermes/plans/2026-08-15_163000-backend-optimization.md` for migration details."
)]
#[allow(deprecated)]
#[allow(dead_code)]
pub struct ServiceBuilder2 {
    service_name: String,
    http_port: u16,
    grpc_port: Option<u16>,
    db_config: Option<DatabaseConfig>,
    middleware_config: MiddlewareConfig,
    router: Option<Router>,
}

#[allow(deprecated)]
impl ServiceBuilder2 {
    /// 创建新的服务构建器
    #[must_use]
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            http_port: 8080,
            grpc_port: None,
            db_config: None,
            middleware_config: MiddlewareConfig::default(),
            router: None,
        }
    }

    /// 设置 HTTP 端口
    #[must_use]
    pub const fn with_http_port(mut self, port: u16) -> Self {
        self.http_port = port;
        self
    }

    /// 设置 gRPC 端口
    #[must_use]
    pub const fn with_grpc_port(mut self, port: u16) -> Self {
        self.grpc_port = Some(port);
        self
    }

    /// 设置数据库配置
    #[must_use]
    pub fn with_db_pool(mut self, config: DatabaseConfig) -> Self {
        self.db_config = Some(config);
        self
    }

    /// 设置中间件配置
    #[must_use]
    pub fn with_middleware(mut self, config: MiddlewareConfig) -> Self {
        self.middleware_config = config;
        self
    }

    /// 设置 HTTP 路由
    #[must_use]
    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// 获取 HTTP 端口
    #[must_use]
    pub const fn http_port(&self) -> u16 {
        self.http_port
    }

    /// 获取服务名称
    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// 创建 CORS 层
    pub fn create_cors_layer(&self) -> CorsLayer {
        let origin_list: Vec<String> = self
            .middleware_config
            .cors_origins
            .iter()
            .filter(|o| !o.is_empty())
            .cloned()
            .collect();
        crate::http::cors_layer(&origin_list)
    }
}

#[allow(deprecated)]
impl Default for ServiceBuilder2 {
    fn default() -> Self {
        Self::new("service")
    }
}
