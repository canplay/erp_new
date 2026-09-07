//! XLT 信路通停车管理服务入口 — gRPC + HTTP 健康检查
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9096 | XLT 停车管理
//! | HTTP | 8097 | 健康检查端点
//!

use std::net::SocketAddr;
use xlt_service::{{create_state, grpc_server}};
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("xlt-service", 8097, 9096);

    let bootstrap = ServiceBootstrap::new(config);

    // 自定义初始化
    let state = create_state();
    let _http_router = health_routes();

    let grpc_service = move |addr: SocketAddr| {
        let state = state.clone();
        Box::pin(tokio::spawn(async move {
            if let Err(e) = grpc_server::start_grpc_server(addr, state).await {
                tracing::error!("gRPC error: {e}");
            }
        }))
    };
    bootstrap.start_with_grpc_fn(grpc_service).await?;

    Ok(())
}
