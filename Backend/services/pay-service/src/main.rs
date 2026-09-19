//! pay-service - service entry
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9093 | 内部服务通信
//! | HTTP | 8093 | 健康检查端点
//!

use std::net::SocketAddr;
use pay_service::{{AppState, grpc_server}};
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
    let config = ServiceConfig::from_env("pay-service" , 8093, 9093);

    let bootstrap = ServiceBootstrap::new(config);

    // 自定义初始化
    let state = AppState::new().expect("Failed to create AppState");
    let _http_router = health_routes();

    let grpc_service = move |addr: SocketAddr| {
        let state = state.clone();
        Box::pin(tokio::spawn(async move {
            if let Err(e) = grpc_server::start_grpc_server(addr, state).await {
                tracing::error!("gRPC error: {e}" );
            }
        }))
    };
    bootstrap.start_with_grpc_fn(grpc_service).await?;

    Ok(())
}
