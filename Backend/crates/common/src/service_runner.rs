//! 服务运行器 - 统一的 gRPC + HTTP 服务启动逻辑
//!
//! 提供标准化的服务启动、优雅关闭和健康检查功能
//! 所有微服务都应使用此模块来启动服务
//!
//! # 示例
//!
//! ```rust,ignore
//! use common::service_runner::{ServiceRunner, HttpServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = HttpServerConfig::from_env(8080);
//!     // 使用示例
//!     Ok(())
//! }
//! ```

use axum::{Router, routing::get};
use tokio::signal;

use crate::config::ServiceDatabaseConfig;

/// HTTP 服务启动配置
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// HTTP 监听端口
    pub port: u16,
    /// HTTP 监听地址
    pub host: String,
    /// 是否启用健康检查端点 /health
    pub enable_health_check: bool,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_health_check: true,
        }
    }
}

impl HttpServerConfig {
    /// 从环境变量创建配置
    ///
    /// 优先使用 `HTTP_PORT` 环境变量，否则使用 port 参数
    /// 支持 `INTERNAL_ONLY=true` 设置为 127.0.0.1（仅内网访问）
    #[must_use]
    pub fn from_env(port: u16) -> Self {
        let actual_port: u16 = std::env::var("HTTP_PORT" )
            .unwrap_or_else(|_| port.to_string())
            .parse()
            .unwrap_or(port);

        // 检查是否仅内网访问
        let host = if std::env::var("INTERNAL_ONLY" )
            .unwrap_or_default()
            .to_lowercase()
            == "true"
        {
            tracing::info!("服务配置为仅内网访问，绑定 127.0.0.1" );
            "127.0.0.1".to_string()
        } else {
            "0.0.0.0".to_string()
        };

        Self {
            port: actual_port,
            host,
            enable_health_check: true,
        }
    }

    /// 获取监听地址
    #[must_use]
    pub fn listen_addr(&self) -> String {
        format!("{}:{}" , self.host, self.port)
    }
}

/// gRPC 服务启动配置
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    /// gRPC 监听端口
    pub port: u16,
    /// gRPC 监听地址
    pub host: String,
}

impl GrpcServerConfig {
    /// 创建 gRPC 配置（默认仅内网访问）
    #[must_use]
    pub fn new(port: u16) -> Self {
        // gRPC 服务默认仅内网访问，除非设置 PUBLIC_GRPC=true
        let host = if std::env::var("PUBLIC_GRPC" )
            .unwrap_or_default()
            .to_lowercase()
            == "true"
        {
            "0.0.0.0".to_string()
        } else {
            tracing::debug!("gRPC 服务默认仅内网访问" );
            "127.0.0.1".to_string()
        };
        Self { port, host }
    }

    /// 获取监听地址
    #[must_use]
    pub fn listen_addr(&self) -> String {
        format!("{}:{}" , self.host, self.port)
    }
}

/// 统一的服务启动器
pub struct ServiceRunner {
    /// 服务名称
    service_name: String,
    /// HTTP 端口
    http_port: u16,
    /// gRPC 端口
    grpc_port: Option<u16>,
    /// 是否使用环境变量覆盖
    use_env_overrides: bool,
    /// 是否禁用 HTTP 服务器（纯 gRPC 模式）
    no_http: bool,
}

/// 服务运行器句柄 - `build()` 返回的结果
pub struct ServiceRunnerHandle {
    /// 服务名称
    pub service_name: String,
    /// 数据库连接池
    pool: Option<sqlx::PgPool>,
    /// HTTP 监听地址
    pub http_addr: String,
    /// gRPC 监听地址
    pub grpc_addr: Option<String>,
    /// 关闭信号接收器
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
}

impl ServiceRunnerHandle {
    /// 获取数据库连接池
    #[must_use]
    pub const fn pool(&self) -> Option<&sqlx::PgPool> {
        self.pool.as_ref()
    }

    /// 获取 HTTP 地址
    #[must_use]
    pub fn http_addr(&self) -> &str {
        &self.http_addr
    }

    /// 获取 gRPC 地址
    #[must_use]
    pub fn grpc_addr(&self) -> Option<&str> {
        self.grpc_addr.as_deref()
    }

    /// 创建关闭 Future，用于与 axum 或 tonic 集成
    pub fn create_shutdown_future(&self) -> impl std::future::Future<Output = ()> + Send + 'static {
        let mut rx = self.shutdown_rx.resubscribe();
        async move {
            let _ = rx.recv().await;
        }
    }
}

impl ServiceRunner {
    /// 创建新的服务运行器
    #[must_use]
    pub fn new(service_name: &str, http_port: u16) -> Self {
        Self {
            service_name: service_name.to_string(),
            http_port,
            grpc_port: None,
            use_env_overrides: false,
            no_http: false,
        }
    }

    /// 设置 gRPC 端口
    #[must_use]
    pub const fn with_grpc_port(mut self, port: u16) -> Self {
        self.grpc_port = Some(port);
        self
    }

    /// 启用环境变量覆盖
    #[must_use]
    pub const fn with_env_overrides(mut self) -> Self {
        self.use_env_overrides = true;
        self
    }

    /// 禁用 HTTP 服务器（纯 gRPC 模式）
    ///
    /// `设置此标志后，http_addr()` 返回空字符串。
    /// 适用于仅需要 gRPC 通信的后端服务。
    #[must_use]
    pub const fn with_no_http(mut self) -> Self {
        self.no_http = true;
        self
    }

    /// 仅启用 gRPC 模式（禁用 HTTP）
    ///
    /// 将 HTTP 端口设为 0，与 `ServiceRunner::new(name, 0)` 效果相同，
    /// 但语义更明确，调用者无需理解"port=0 即禁用 HTTP"的内部约定。
    #[must_use]
    pub const fn grpc_only(mut self) -> Self {
        self.http_port = 0;
        self
    }

    /// 构建并启动服务（含数据库连接池）
    ///
    /// 初始化服务、解析端口、建立数据库连接池、设置优雅关闭。
    pub async fn build(&mut self) -> Result<ServiceRunnerHandle, Box<dyn std::error::Error>> {
        self.build_inner(true).await
    }

    /// 构建并启动服务（不含数据库连接池）
    ///
    /// 适用于 API 网关等不需要数据库的服务。
    /// 与 `build()` 的区别在于不会创建 `PgPool`。
    pub async fn build_without_db(
        &mut self,
    ) -> Result<ServiceRunnerHandle, Box<dyn std::error::Error>> {
        self.build_inner(false).await
    }

    /// 内部构建逻辑
    async fn build_inner(
        &mut self,
        create_pool: bool,
    ) -> Result<ServiceRunnerHandle, Box<dyn std::error::Error>> {
        // 初始化服务
        init_service(&self.service_name);

        // 确定端口（可能来自环境变量）
        let http_port = self.resolve_http_port();
        let grpc_port = self.resolve_grpc_port();

        // 绑定地址
        let http_host = self.resolve_http_host();
        let grpc_host = self.resolve_grpc_host();

        // 数据库连接池（可选）
        let pool = if create_pool {
            let database_url = self.resolve_database_url();
            // 从环境变量读取连接池配置（DB_POOL_*），默认 10/2
            let max_connections = std::env::var("DB_POOL_MAX_CONNECTIONS" )
                .ok().and_then(|v| v.parse().ok()).unwrap_or(10);
            let min_connections = std::env::var("DB_POOL_MIN_CONNECTIONS" )
                .ok().and_then(|v| v.parse().ok()).unwrap_or(2);
            let connect_timeout = std::env::var("DB_POOL_CONNECT_TIMEOUT" )
                .ok().and_then(|v| v.parse().ok()).unwrap_or(30);
            let idle_timeout = std::env::var("DB_POOL_IDLE_TIMEOUT" )
                .ok().and_then(|v| v.parse().ok()).unwrap_or(600);
            let max_lifetime = std::env::var("DB_POOL_MAX_LIFETIME" )
                .ok().and_then(|v| v.parse().ok()).unwrap_or(1800);
            let pool_opts = sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_connections)
                .min_connections(min_connections)
                .acquire_timeout(std::time::Duration::from_secs(connect_timeout))
                .idle_timeout(std::time::Duration::from_secs(idle_timeout))
                .max_lifetime(std::time::Duration::from_secs(max_lifetime));

            let pool = pool_opts.connect(&database_url).await?;

            // 如果配置了 schema 隔离，设置 search_path
            if let Ok(schema) = std::env::var("SERVICE_SCHEMA" ) {
                let sql = format!("SET search_path TO {schema}" );
                // B11 豁免: schema 迁移 SQL 运行时动态生成, 无法用编译期宏
        sqlx::query(&sql).execute(&pool).await?;
                tracing::info!(
                    "Schema 隔离已启用: service={}, schema={}" ,
                    self.service_name, schema
                );
            }

            tracing::info!(
                "数据库连接池已建立: {} (max={}, min={})" ,
                self.service_name,
                max_connections,
                min_connections
            );
            Some(pool)
        } else {
            None
        };

        // 优雅关闭信号
        let shutdown_rx = setup_shutdown_handler();

        // HTTP 地址（no_http 或端口为 0 时返回空）
        let http_addr = if self.no_http || http_port == 0 {
            String::new()
        } else {
            format!("{http_host}:{http_port}" )
        };

        Ok(ServiceRunnerHandle {
            service_name: self.service_name.clone(),
            pool,
            http_addr,
            grpc_addr: grpc_port.map(|p| format!("{grpc_host}:{p}" )),
            shutdown_rx,
        })
    }

    /// 解析 HTTP 端口
    fn resolve_http_port(&self) -> u16 {
        if self.use_env_overrides {
            Self::get_http_port_from_env(self.http_port)
        } else {
            self.http_port
        }
    }

    /// 解析 gRPC 端口
    fn resolve_grpc_port(&self) -> Option<u16> {
        if self.use_env_overrides {
            self.grpc_port.map(Self::get_grpc_port_from_env)
        } else {
            self.grpc_port
        }
    }

    /// 解析 HTTP 绑定地址
    fn resolve_http_host(&self) -> String {
        if self.use_env_overrides
            && std::env::var("INTERNAL_ONLY" )
                .unwrap_or_default()
                .to_lowercase()
                == "true"
        {
            tracing::info!(
                "服务 {} HTTP 配置为仅内网访问，绑定 127.0.0.1" ,
                self.service_name
            );
            "127.0.0.1".to_string()
        } else {
            "0.0.0.0".to_string()
        }
    }

    /// 解析 gRPC 绑定地址
    fn resolve_grpc_host(&self) -> String {
        if self.use_env_overrides {
            if std::env::var("PUBLIC_GRPC" )
                .unwrap_or_default()
                .to_lowercase()
                == "true"
            {
                tracing::info!(
                    "服务 {} gRPC 配置为公网访问，绑定 0.0.0.0" ,
                    self.service_name
                );
                "0.0.0.0".to_string()
            } else {
                tracing::info!(
                    "服务 {} gRPC 配置为仅内网访问，绑定 127.0.0.1" ,
                    self.service_name
                );
                "127.0.0.1".to_string()
            }
        } else {
            "0.0.0.0".to_string()
        }
    }

    /// 解析数据库 URL
    ///
    /// 优先级：
    /// 1. `{SERVICE_NAME}_DB_URL`（服务专用）
    /// 2. `SERVICE_DB_URL`（通用）
    /// 3. `DATABASE_URL`（全局回退）
    fn resolve_database_url(&self) -> String {
        ServiceDatabaseConfig::resolve_url(&self.service_name)
    }
}

impl ServiceRunner {
    /// 启动带健康检查的 HTTP 路由
    ///
    /// 将健康检查端点添加到现有路由
    pub fn with_health_check(router: Router) -> Router {
        router.route(
            "/health" ,
            get(|| async {
                axum::Json(serde_json::json!({
                    "status": "ok" ,
                    "service": "myai" ,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }),
        )
    }

    /// 获取 HTTP 服务端口（从 `HTTP_PORT` 环境变量）
    ///
    /// 从环境变量获取端口，如果未设置则使用默认值
    #[must_use]
    pub fn get_http_port_from_env(default_port: u16) -> u16 {
        std::env::var("HTTP_PORT" )
            .unwrap_or_else(|_| default_port.to_string())
            .parse()
            .unwrap_or(default_port)
    }

    /// 获取 gRPC 服务端口（从 `GRPC_PORT` 环境变量）
    ///
    /// 独立的 gRPC 端口配置，与 HTTP 端口互不干扰。
    /// 如果未设置则使用默认值。
    #[must_use]
    pub fn get_grpc_port_from_env(default_port: u16) -> u16 {
        std::env::var("GRPC_PORT" )
            .unwrap_or_else(|_| default_port.to_string())
            .parse()
            .unwrap_or(default_port)
    }

    /// 获取服务端口
    ///
    /// 从 `HTTP_PORT` 环境变量获取端口，如果未设置则使用默认值
    /// 此方法为向后兼容保留，新代码请使用 `get_http_port_from_env`
    #[must_use]
    pub fn get_port_from_env(default_port: u16) -> u16 {
        Self::get_http_port_from_env(default_port)
    }

    /// 获取服务地址
    #[must_use]
    pub fn get_listen_addr(port: u16) -> String {
        format!("0.0.0.0:{port}" )
    }
}

/// 健康检查响应
#[must_use]
pub fn health_check() -> &'static str {
    "OK"
}

/// 健康检查 JSON 响应
#[must_use]
pub fn health_check_json() -> &'static str {
    r#"{"status":"ok" }"#
}

/// 检查配置并设置默认值的辅助函数
#[must_use]
pub fn ensure_required_env(var_name: &str, default: Option<&str>) -> String {
    std::env::var(var_name).unwrap_or_else(|_| {
        if let Some(default) = default {
            tracing::warn!("{var_name} not set, using default" );
            default.to_string()
        } else {
            tracing::warn!("{var_name} not set" );
            String::new()
        }
    })
}

/// 确保数据库 URL 配置正确
///
/// 检查顺序：
/// 1. `SERVICE_DB_URL`（通用服务专用）
/// 2. `DATABASE_URL`（全局回退）
#[must_use]
pub fn ensure_database_url() -> String {
    std::env::var("SERVICE_DB_URL" ).unwrap_or_else(|_| {
        ensure_required_env(
            "DATABASE_URL" ,
            Some("postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/datafusion" )
        )
    })
}

/// 确保服务的专用数据库 URL 配置正确
///
/// 检查顺序：
/// 1. `{SERVICE_NAME}_DB_URL`（服务专用，如 `USER_SERVICE_DB_URL`）
/// 2. `SERVICE_DB_URL`（通用）
/// 3. `DATABASE_URL`（全局回退）
#[must_use]
pub fn ensure_service_database_url(service_name: &str) -> String {
    ServiceDatabaseConfig::resolve_url(service_name)
}

/// 设置优雅关闭信号处理器
#[must_use]
pub fn setup_shutdown_handler() -> tokio::sync::broadcast::Receiver<()> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("收到 Ctrl+C 信号，正在关闭服务..." );
                let _ = shutdown_tx.send(());
            }
            Err(e) => {
                tracing::error!("监听信号失败: {e}" );
            }
        }
    });

    shutdown_rx
}

/// 初始化服务（简化版）
///
/// 初始化 tracing 和环境变量
pub fn init_service(service_name: &str) {
    crate::init::init_tracing(service_name);
    crate::init::init_env();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_default() {
        let config = HttpServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0" );
        assert!(config.enable_health_check);
    }

    #[test]
    fn test_http_config_listen_addr() {
        let config = HttpServerConfig {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_health_check: true,
        };
        assert_eq!(config.listen_addr(), "0.0.0.0:8080" );
    }

    #[test]
    fn test_grpc_config() {
        // gRPC 服务默认仅内网访问（127.0.0.1），除非设置 PUBLIC_GRPC=true
        let config = GrpcServerConfig::new(9090);
        assert_eq!(config.port, 9090);
        // 默认返回 127.0.0.1（内网访问）
        assert_eq!(config.listen_addr(), "127.0.0.1:9090" );
    }

    #[test]
    fn test_get_listen_addr() {
        assert_eq!(ServiceRunner::get_listen_addr(8080), "0.0.0.0:8080" );
    }
}
