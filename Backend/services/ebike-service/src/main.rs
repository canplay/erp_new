//! ebike-service - 电动自行车管理系统
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9100 | 内部服务通信
//! | HTTP | 8098 | 健康检查端点
//!

use std::sync::Arc;
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use ebike_service::{{create_state, grpc_server}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("ebike-service", 8098, 9100);

    let bootstrap = ServiceBootstrap::new(config);

    // 自定义初始化
    let pool = ebike_service::create_optimized_pool().await?;
    let state = create_state(pool);
    let _http_router = health_routes();

    let grpc_builder = grpc_server::GrpcServerBuilder { state: Arc::new(state) };
    bootstrap.start_with_trait(grpc_builder).await?;

    Ok(())
}
