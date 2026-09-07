//! 微服务通用启动器
//!
//! 提供统一的 gRPC + HTTP 双协议服务启动逻辑。
//! 所有微服务都应使用此模块来启动服务，减少重复代码。
//!
//! # 使用方式
//!
//! 每个服务只需要在 `main.rs` 中配置：
//! - 服务名称
//! - HTTP 端口（用于健康检查）
//! - gRPC 端口（用于内部服务通信）
//! - 自定义的 gRPC 服务实现
//!
//! 启动器会自动处理：
//! - 端口解析（支持环境变量覆盖）
//! - 数据库连接池创建
//! - HTTP 健康检查端点
//! - gRPC 服务器启动
//! - HTTP 服务器启动
//! - 优雅关闭信号处理
//!
//! # 示例
//!
//! ```rust,ignore
//! use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ServiceConfig {
//!         service_name: "my-service",
//!         http_port: 8080,
//!         grpc_port: 9080,
//!         grpc_addr: "0.0.0.0:9080".to_string(),
//!     };
//!
//!     let bootstrap = ServiceBootstrap::new(config);
//!     bootstrap.start(
//!         || async { create_db_pool().await },
//!         |pool| create_grpc_service(pool),
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```

use axum::Router;
use sqlx::PgPool;
use std::net::SocketAddr;

/// gRPC 服务构建器 trait
///
/// 所有服务的 gRPC 服务实现都需要实现此 trait。
/// 这允许每个服务自定义其 gRPC 服务的创建方式。
pub trait GrpcServiceBuilder {
    /// 构建 gRPC 服务
    ///
    /// # 参数
    /// - `grpc_addr`: gRPC 监听地址
    ///
    /// # 返回
    /// gRPC 服务器的 Future，用于 `tokio::select!` 中等待
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>>;
}

/// HTTP 服务构建器 trait
///
/// 所有服务的 HTTP 服务实现都需要实现此 trait。
/// 这允许每个服务自定义其 HTTP 路由的创建方式。
pub trait HttpRouterBuilder {
    /// 构建 HTTP 路由
    ///
    /// # 返回
    /// HTTP 路由，通常包含健康检查端点和业务 API
    fn build_http_router(&self) -> Router;
}

/// 服务配置
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// 服务名称（用于日志和配置）
    pub service_name: &'static str,
    /// HTTP 端口（用于健康检查端点）
    pub http_port: u16,
    /// gRPC 端口（用于内部服务通信）
    pub grpc_port: u16,
    /// gRPC 监听地址
    pub grpc_addr: String,
    /// HTTP 监听地址
    pub http_addr: String,
}

impl ServiceConfig {
    /// 从环境变量创建服务配置
    ///
    /// 支持以下环境变量：
    /// - `SERVICE_PORT`: HTTP 端口（覆盖默认值）
    /// - `GRPC_PORT`: gRPC 端口（覆盖默认值）
    /// - `INTERNAL_ONLY`: 如果为 "true"，HTTP 绑定到 127.0.0.1
    /// - `PUBLIC_GRPC`: 如果为 "true"，gRPC 绑定到 0.0.0.0
    pub fn from_env(service_name: &'static str, default_http_port: u16, default_grpc_port: u16) -> Self {
        let http_port = Self::get_http_port_from_env(default_http_port);
        let grpc_port = Self::get_grpc_port_from_env(default_grpc_port);

        let http_host = if std::env::var("INTERNAL_ONLY")
            .unwrap_or_default()
            .to_lowercase()
            == "true"
        {
            tracing::info!("服务 {} HTTP 配置为仅内网访问，绑定 127.0.0.1", service_name);
            "127.0.0.1"
        } else {
            "0.0.0.0"
        };

        let grpc_host = if std::env::var("PUBLIC_GRPC")
            .unwrap_or_default()
            .to_lowercase()
            == "true"
        {
            tracing::info!("服务 {} gRPC 配置为公网访问，绑定 0.0.0.0", service_name);
            "0.0.0.0"
        } else {
            tracing::info!("服务 {} gRPC 配置为仅内网访问，绑定 127.0.0.1", service_name);
            "127.0.0.1"
        };

        Self {
            service_name,
            http_port,
            grpc_port,
            grpc_addr: format!("{}:{}", grpc_host, grpc_port),
            http_addr: format!("{}:{}", http_host, http_port),
        }
    }

    /// 从环境变量获取 HTTP 端口
    fn get_http_port_from_env(default_port: u16) -> u16 {
        std::env::var("HTTP_PORT")
            .unwrap_or_else(|_| default_port.to_string())
            .parse()
            .unwrap_or(default_port)
    }

    /// 从环境变量获取 gRPC 端口
    fn get_grpc_port_from_env(default_port: u16) -> u16 {
        std::env::var("GRPC_PORT")
            .unwrap_or_else(|_| default_port.to_string())
            .parse()
            .unwrap_or(default_port)
    }

    /// 解析 gRPC 地址为 SocketAddr
    pub fn parse_addr(&self) -> SocketAddr {
        self.grpc_addr.parse().expect("invalid grpc_addr in ServiceConfig")
    }
}

/// 服务启动器
pub struct ServiceBootstrap {
    config: ServiceConfig,
}

impl ServiceBootstrap {
    /// 创建新的服务启动器
    #[must_use]
    pub fn new(config: ServiceConfig) -> Self {
        Self { config }
    }

    /// 从环境变量创建服务启动器
    #[must_use]
    pub fn from_env(
        service_name: &'static str,
        default_http_port: u16,
        default_grpc_port: u16,
    ) -> Self {
        Self {
            config: ServiceConfig::from_env(service_name, default_http_port, default_grpc_port),
        }
    }

    /// 启动服务
    ///
    /// # 参数
    /// - `db_pool_factory`: 创建数据库连接池的工厂函数
    /// - `grpc_service_factory`: 创建 gRPC 服务的工厂函数
    /// - `http_router_factory`: 创建 HTTP 路由的工厂函数
    ///
    /// # 返回
    /// 当 HTTP 或 gRPC 服务器关闭时返回
    pub async fn start_with_fns<F, G, H>(
        &self,
        db_pool_factory: F,
        grpc_service_factory: G,
        http_router_factory: H,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<PgPool, Box<dyn std::error::Error>>,
        G: FnOnce(PgPool) -> Result<Box<dyn GrpcServiceBuilder>, Box<dyn std::error::Error>>,
        H: FnOnce() -> Result<Router, Box<dyn std::error::Error>>,
    {
        self.start_impl(db_pool_factory, grpc_service_factory, http_router_factory).await
    }

    /// 启动服务（使用 trait）
    ///
    /// # 参数
    /// - `grpc_service_builder`: gRPC 服务构建器
    ///
    /// # 返回
    /// 当 HTTP 或 gRPC 服务器关闭时返回
    pub async fn start_with_trait<G>(
        &self,
        grpc_service_builder: G,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        G: GrpcServiceBuilder,
    {
        let grpc_addr = self.config.grpc_addr.clone();
        let http_addr = self.config.http_addr.clone();
        let service_name = self.config.service_name;

        tracing::info!("{} 启动中", service_name);
        tracing::info!("  - gRPC 监听: {}", grpc_addr);
        tracing::info!("  - HTTP 监听: {}", http_addr);

        // 创建 HTTP 路由（默认只包含健康检查）
        let http_router = Self::default_http_router();

        // 启动 HTTP 服务器
        let http_listener = tokio::net::TcpListener::bind(&http_addr).await?;
        let mut shutdown_rx = Self::setup_shutdown_handler();

        let http_server = axum::serve(http_listener, http_router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
            });

        // 启动 gRPC 服务器
        let grpc_handle = match grpc_service_builder.build_grpc_server(&grpc_addr) {
            Ok(h) => h,
            Err(e) => return Err(e as Box<dyn std::error::Error + Send + Sync>),
        };

        // 同时启动 HTTP 和 gRPC 服务器
        tracing::info!("{} 已启动", service_name);
        tokio::select! {
            _ = http_server => {
                tracing::info!("HTTP 服务器已关闭");
            }
            _ = grpc_handle => {
                tracing::info!("gRPC 服务器已关闭");
            }
        }

        tracing::info!("{} 已关闭", service_name);
        Ok(())
    }

    /// 启动服务（使用 gRPC 函数，而非 trait）
    ///
    /// # 参数
    /// - `grpc_service_fn`: gRPC 服务启动函数，接收 `addr`（state 由调用方在闭包中捕获）
    ///
    /// # 返回
    /// 当 HTTP 或 gRPC 服务器关闭时返回
    pub async fn start_with_grpc_fn<F>(
        &self,
        grpc_service_fn: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(SocketAddr) -> std::pin::Pin<Box<tokio::task::JoinHandle<()>>> + Send + 'static,
    {
        let config = self.config.clone();
        tracing::info!("{} 启动中", config.service_name);

        // 创建 HTTP 路由（默认只包含健康检查）
        let http_router = Self::default_http_router();

        // 启动 HTTP 服务器（带优雅关闭）
        let http_listener = tokio::net::TcpListener::bind(&config.http_addr).await?;
        let mut shutdown_rx = Self::setup_shutdown_handler();

        let http_server = axum::serve(http_listener, http_router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
            });

        // 启动 gRPC 服务器（与 HTTP 并行运行）
        let config_for_grpc = config.clone();
        let _ = grpc_service_fn(config_for_grpc.parse_addr());

        // 等待 HTTP 服务器关闭（gRPC 服务器在后台运行）
        tokio::select! {
            _ = http_server => {
                tracing::info!("HTTP 服务器已关闭");
            }
        }

        tracing::info!("{} 已关闭", config.service_name);
        Ok(())
    }

    /// 内部实现
    async fn start_impl<F, G, H>(
        &self,
        db_pool_factory: F,
        grpc_service_factory: G,
        http_router_factory: H,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<PgPool, Box<dyn std::error::Error>>,
        G: FnOnce(PgPool) -> Result<Box<dyn GrpcServiceBuilder>, Box<dyn std::error::Error>>,
        H: FnOnce() -> Result<Router, Box<dyn std::error::Error>>,
    {
        let service_name = self.config.service_name;
        let grpc_addr = self.config.grpc_addr.clone();
        let http_addr = self.config.http_addr.clone();

        tracing::info!("{} 启动中", service_name);
        tracing::info!("  - gRPC 监听: {}", grpc_addr);
        tracing::info!("  - HTTP 监听: {}", http_addr);

        // 创建数据库连接池
        let pool = db_pool_factory()?;

        // 创建 gRPC 服务
        let grpc_service_builder = grpc_service_factory(pool)?;

        // 创建 HTTP 路由
        let http_router = http_router_factory()?;

        // 启动 HTTP 服务器
        let http_listener = tokio::net::TcpListener::bind(&http_addr).await?;
        let mut shutdown_rx = Self::setup_shutdown_handler();

        let http_server = axum::serve(http_listener, http_router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
            });

        // 启动 gRPC 服务器
        let grpc_handle = match grpc_service_builder.build_grpc_server(&grpc_addr) {
            Ok(h) => h,
            Err(e) => return Err(e as Box<dyn std::error::Error + Send + Sync>),
        };

        // 同时启动 HTTP 和 gRPC 服务器
        tracing::info!("{} 已启动", service_name);
        tokio::select! {
            _ = http_server => {
                tracing::info!("HTTP 服务器已关闭");
            }
            _ = grpc_handle => {
                tracing::info!("gRPC 服务器已关闭");
            }
        }

        tracing::info!("{} 已关闭", service_name);
        Ok(())
    }

    /// 默认 HTTP 路由（仅健康检查）
    fn default_http_router() -> Router {
        Router::new().route(
            "/health",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({
                    "status": "ok",
                    "service": "myai",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }),
        )
    }

    /// 设置优雅关闭信号处理器
    fn setup_shutdown_handler() -> tokio::sync::broadcast::Receiver<()> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

        tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    tracing::info!("收到 Ctrl+C 信号，正在关闭服务...");
                    let _ = shutdown_tx.send(());
                }
                Err(e) => {
                    tracing::error!("监听信号失败: {}", e);
                }
            }
        });

        shutdown_rx
    }

    /// 创建关闭信号 Future
    pub fn create_shutdown_future(&self) -> impl Future<Output = ()> + 'static {
        let mut shutdown_rx = Self::setup_shutdown_handler();
        async move {
            let _ = shutdown_rx.recv().await;
        }
    }
}
