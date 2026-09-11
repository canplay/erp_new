//! Browser Service — 浏览器自动化管理微服务入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 8120 | 内部服务通信
//! | HTTP | 8120 | 健康检查端点
//!

use std::net::SocketAddr;
use std::sync::Arc;
use browser_service::browser_pool::BrowserPool;
use browser_service::grpc_server;
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};

const DEFAULT_POOL_SIZE: usize = 4;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("browser-service", 8120, 8120);

    let bootstrap = ServiceBootstrap::new(config);

    // 初始化浏览器池
    let pool_size = std::env::var("BROWSER_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_POOL_SIZE);

    log::info!("[BrowserService] initializing browser pool (size={})", pool_size);

    let pool = match BrowserPool::new(pool_size).await {
        Ok(p) => {
            log::info!("[BrowserService] browser pool ready with {} browsers", p.size());
            Arc::new(p)
        }
        Err(_) => {
            log::warn!("[BrowserService] browser pool init failed (Edge may not be installed)");
            log::warn!("[BrowserService] starting without browser — browser operations will fail");
            let empty_pool = BrowserPool::new(0).await.map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("无法创建空浏览器池: {e}。请检查 drission 依赖是否正确安装"),
                )
            })?;
            Arc::new(empty_pool)
        }
    };

    // 创建 gRPC 服务
    let grpc_service = move |addr: SocketAddr| {
        let pool = pool.clone();
        Box::pin(tokio::spawn(async move {
            if let Err(e) = grpc_server::start_grpc_server(addr, pool).await {
                tracing::error!("gRPC error: {e}");
            }
        }))
    };
    let _http_router = health_routes();

    bootstrap.start_with_grpc_fn(grpc_service).await?;

    #[cfg(feature = "telemetry")]
    common::shutdown_otel();

    Ok(())
}
