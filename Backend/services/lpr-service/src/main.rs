//! 车牌识别服务入口 (lpr-service) — gRPC + HTTP 健康检查
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9099 | 车牌识别回调
//! | HTTP | 8099 | 健康检查端点
//!

use std::net::SocketAddr;
use lpr_service::{{create_grpc_service, grpc_server}};
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
// 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("lpr-service" , 8099, 9099);

    let bootstrap = ServiceBootstrap::new(config);

    // 自定义初始化
    let grpc_service = create_grpc_service().await?;
    let _http_router = health_routes();

    let grpc_fn = move |addr: SocketAddr| {
        Box::pin(tokio::spawn(async move {
            if let Err(e) = grpc_server::start_grpc_server(addr, grpc_service).await {
                tracing::error!("gRPC error: {e}" );
            }
        }))
    };
    bootstrap.start_with_grpc_fn(grpc_fn).await?;

    Ok(())
}
