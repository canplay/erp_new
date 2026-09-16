//! 统一的服务启动器 - Bootstrap 模块
//!
//! 提供标准化的服务启动、优雅关闭和健康检查功能。
//! 所有微服务都应使用此模块来启动服务，减少重复代码。
//!
//! # 支持的服务类型
//!
//! - 仅 HTTP 服务
//! - gRPC + HTTP 双协议服务
//! - API Gateway（特殊处理）
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use common::bootstrap::{Bootstrap, ServerType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     Bootstrap::new("user-service" , ServerType::GrpcHttp { grpc_port: 9090, http_port: 8080 })
//!         .with_database()
//!         .run(|state| create_router(state))
//!         .await
//! }
//! ```

use crate::config::{AppConfig, DatabaseConfig, ServiceDatabaseConfig, create_db_pool, create_service_db_pool};
use axum::Router;
use std::sync::Arc;
use tokio::signal;

/// 服务类型
#[derive(Debug, Clone)]
pub enum ServerType {
    /// 仅 HTTP 服务
    HttpOnly { port: u16 },
    /// gRPC + HTTP 双协议服务
    GrpcHttp { grpc_port: u16, http_port: u16 },
    /// API Gateway（特殊处理）
    ApiGateway { port: u16 },
}

impl ServerType {
    /// 获取 HTTP 端口
    #[must_use]
    pub const fn http_port(&self) -> u16 {
        match self {
            Self::HttpOnly { port } => *port,
            Self::GrpcHttp { http_port, .. } => *http_port,
            Self::ApiGateway { port } => *port,
        }
    }

    /// 获取 gRPC 端口
    #[must_use]
    pub const fn grpc_port(&self) -> Option<u16> {
        match self {
            Self::HttpOnly { .. } => None,
            Self::GrpcHttp { grpc_port, .. } => Some(*grpc_port),
            Self::ApiGateway { .. } => None,
        }
    }
}

/// 数据库连接池包装器
pub struct DbPool(pub sqlx::PgPool);

impl DbPool {
    /// 获取内部池引用
    #[must_use]
    pub const fn pool(&self) -> &sqlx::PgPool {
        &self.0
    }

    /// 异步关闭连接池
    pub async fn close(&self) {
        self.0.close().await;
    }
}

/// 统一的服务启动器
pub struct Bootstrap {
    /// 服务名称
    service_name: String,
    /// 服务类型
    server_type: ServerType,
    /// 是否启用数据库
    with_db: bool,
    /// 数据库配置
    db_config: Option<DatabaseConfig>,
    /// 是否使用服务专用数据库配置（替代全局 `DATABASE_URL`）
    use_service_db: bool,
    /// 服务专用数据库配置
    service_db_config: Option<ServiceDatabaseConfig>,
}

impl Bootstrap {
    /// 创建新的 Bootstrap 实例
    #[must_use]
    pub fn new(service_name: &str, server_type: ServerType) -> Self {
        Self {
            service_name: service_name.to_string(),
            server_type,
            with_db: false,
            db_config: None,
            use_service_db: false,
            service_db_config: None,
        }
    }

    /// 启用数据库连接
    #[must_use]
    pub const fn with_database(mut self) -> Self {
        self.with_db = true;
        self
    }

    /// 设置自定义数据库配置
    #[must_use]
    pub fn with_db_config(mut self, config: DatabaseConfig) -> Self {
        self.with_db = true;
        self.db_config = Some(config);
        self
    }

    /// 启用服务专用数据库配置（支持 `SERVICE_DB_URL` 和 schema 隔离）
    #[must_use]
    pub fn with_service_database(mut self) -> Self {
        self.with_db = true;
        self.use_service_db = true;
        self.service_db_config = Some(ServiceDatabaseConfig::for_service(&self.service_name));
        self
    }

    /// 设置服务专用数据库配置（自定义）
    #[must_use]
    pub fn with_service_db_config(mut self, config: ServiceDatabaseConfig) -> Self {
        self.with_db = true;
        self.use_service_db = true;
        self.service_db_config = Some(config);
        self
    }

    /// 运行 HTTP 服务
    pub async fn run_http<F>(self, router_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(Option<Arc<DbPool>>) -> Router + Send + 'static,
    {
        // 1. 初始化环境与统一日志（传入服务名确保 JSON 日志包含 service 字段）
        crate::init::init_env();
        crate::init::init_tracing(&self.service_name);

        tracing::info!("=== {} 启动中 ===" , self.service_name);

        // 2. 加载配置
        let config = AppConfig::load().unwrap_or_default();

        // 3. 创建数据库连接池（如果需要）
        let pool = if self.with_db {
            if self.use_service_db {
                let sd_config = self.service_db_config.unwrap_or_else(|| {
                    ServiceDatabaseConfig::for_service(&self.service_name)
                });
                tracing::info!(
                    "服务数据库连接: service={}, url={}" ,
                    self.service_name,
                    sd_config.connection_url().split('@').next_back().unwrap_or("***" )
                );
                if let Some(ref schema) = sd_config.schema {
                    tracing::info!("Schema 隔离启用: {schema}" );
                }
                let p = create_service_db_pool(&sd_config).await?;
                tracing::info!("数据库连接池创建成功 (服务专用)" );
                Some(Arc::new(DbPool(p)))
            } else {
                let db_config = self.db_config.unwrap_or_else(|| config.database.clone());
                tracing::info!(
                    "连接数据库: {}" ,
                    db_config.url.split('@').next_back().unwrap_or("***" )
                );
                let p = create_db_pool(&db_config).await?;
                tracing::info!("数据库连接池创建成功" );
                Some(Arc::new(DbPool(p)))
            }
        } else {
            None
        };

        // 4. 获取端口
        let http_port = self.server_type.http_port();
        let http_addr = format!("0.0.0.0:{http_port}" );

        tracing::info!("HTTP 服务地址: {http_addr}" );
        if let Some(grpc_port) = self.server_type.grpc_port() {
            tracing::info!("gRPC 服务地址: 0.0.0.0:{grpc_port}" );
        }

        // 5. 创建 HTTP 路由
        let router = router_fn(pool.clone());
        let router = add_health_routes(router);

        // 6. 创建 shutdown 信号处理器
        let mut shutdown_rx = Self::setup_shutdown_handler();

        // 7. 启动 HTTP 服务
        tracing::info!("HTTP 服务启动..." );
        let listener = tokio::net::TcpListener::bind(&http_addr).await?;

        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
            })
            .await?;

        // 8. 关闭数据库连接池
        if let Some(pool) = &pool {
            tracing::info!("关闭数据库连接池..." );
            pool.close().await;
        }

        tracing::info!("=== {} 已优雅关闭 ===" , self.service_name);
        Ok(())
    }

    /// 设置优雅关闭信号处理器
    fn setup_shutdown_handler() -> tokio::sync::broadcast::Receiver<()> {
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
}

/// 添加健康检查路由
fn add_health_routes(router: Router) -> Router {
    router.route("/health" , axum::routing::get(health_handler))
}

async fn health_handler() -> &'static str {
    "OK"
}

/// 帮助函数：创建标准的服务运行器
///
/// 适用于简单的 HTTP 服务，只需提供数据库连接池和路由创建函数
pub async fn run_http_service(
    service_name: &str,
    port: u16,
    pool: Option<sqlx::PgPool>,
    router_fn: impl FnOnce(Option<Arc<DbPool>>) -> Router,
) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化
    crate::init::init_env();
    let _ = crate::init::init_logging();

    tracing::info!("=== {service_name} 启动中 ===" );

    // 创建池包装
    let wrapped_pool = pool.map(|p| Arc::new(DbPool(p)));

    if wrapped_pool.is_some() {
        tracing::info!("数据库连接池已准备" );
    }

    // 获取地址
    let addr = format!("0.0.0.0:{port}" );
    tracing::info!("监听地址: {addr}" );

    // 创建路由
    let router = router_fn(wrapped_pool.clone());
    let router = add_health_routes(router);

    // 设置关闭信号
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    tokio::spawn(async move {
        if matches!(signal::ctrl_c().await, Ok(())) {
            tracing::info!("收到 Ctrl+C 信号，正在关闭服务..." );
            let _ = shutdown_tx.send(());
        }
    });

    // 启动服务
    tracing::info!("HTTP 服务启动..." );
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await?;

    // 关闭连接池
    if let Some(pool) = &wrapped_pool {
        tracing::info!("关闭数据库连接池..." );
        pool.close().await;
    }

    tracing::info!("=== {service_name} 已优雅关闭 ===" );
    Ok(())
}

/// 运行双协议服务（gRPC + HTTP）
pub async fn run_dual_service(
    service_name: &str,
    grpc_port: u16,
    http_port: u16,
    pool: sqlx::PgPool,
    http_router_fn: impl FnOnce(Arc<DbPool>) -> Router + Send + 'static,
    grpc_factory: impl FnOnce(Arc<DbPool>) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化
    crate::init::init_env();
    let _ = crate::init::init_logging();

    tracing::info!("=== {service_name} 启动中 (gRPC + HTTP) ===" );

    let wrapped_pool = Arc::new(DbPool(pool));
    tracing::info!("数据库连接池已准备" );

    // 如果配置了 schema 隔离，设置 search_path
    let sd_config = ServiceDatabaseConfig::for_service(service_name);
    if let Some(ref schema) = sd_config.schema {
        tracing::info!("Schema 隔离已启用: service={service_name}, schema={schema}" );
    }

    // gRPC 地址
    let grpc_addr = format!("0.0.0.0:{grpc_port}" );
    let http_addr = format!("0.0.0.0:{http_port}" );
    tracing::info!("gRPC 监听: {grpc_addr}" );
    tracing::info!("HTTP 监听: {http_addr}" );

    // 创建 shutdown 信号
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    tokio::spawn(async move {
        if matches!(signal::ctrl_c().await, Ok(())) {
            tracing::info!("收到 Ctrl+C 信号，正在关闭服务..." );
            let _ = shutdown_tx.send(());
        }
    });

    // 创建 HTTP 路由
    let http_state = wrapped_pool.clone();
    let router = http_router_fn(http_state);
    let router = add_health_routes(router);

    // 并行启动两个服务
    let http_handle = tokio::spawn(async move {
        tracing::info!("HTTP 服务启动..." );
        let listener = tokio::net::TcpListener::bind(&http_addr)
            .await
            .expect("HTTP 监听端口绑定失败 — 检查端口是否被占用" );
        axum::serve(listener, router)
            .await
            .expect("HTTP 服务运行失败" );
    });

    // 启动 gRPC 服务
    let grpc_state = wrapped_pool.clone();
    if let Err(e) = grpc_factory(grpc_state) {
        tracing::error!("gRPC 服务启动失败: {e}" );
    }

    // 等待任一服务结束
    tokio::select! {
        result = http_handle => {
            if let Err(e) = result {
                tracing::error!("HTTP 服务出错: {e}" );
            }
        }
        _ = shutdown_rx.recv() => {
            tracing::info!("收到关闭信号..." );
        }
    }

    // 关闭连接池
    tracing::info!("关闭数据库连接池..." );
    wrapped_pool.close().await;

    tracing::info!("=== {service_name} 已优雅关闭 ===" );
    Ok(())
}

/// 获取端口的辅助函数
#[must_use]
pub fn get_port_from_env(default_port: u16) -> u16 {
    std::env::var("HTTP_PORT" )
        .unwrap_or_else(|_| default_port.to_string())
        .parse()
        .unwrap_or(default_port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_type_http_port() {
        let st = ServerType::HttpOnly { port: 8080 };
        assert_eq!(st.http_port(), 8080);
        assert_eq!(st.grpc_port(), None);

        let st = ServerType::GrpcHttp {
            grpc_port: 9090,
            http_port: 8080,
        };
        assert_eq!(st.http_port(), 8080);
        assert_eq!(st.grpc_port(), Some(9090));
    }

    #[test]
    fn test_get_port_from_env() {
        // 默认值测试
        unsafe { std::env::remove_var("HTTP_PORT" ) };
        assert_eq!(get_port_from_env(8080), 8080);
    }
}
