//! clean-service - 服务入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9087 | 内部服务通信
//! | HTTP | 8095 | 健康检查端点
//!

use clean_service::handlers::AppState;
use clean_service::services::grpc_impl::CleanGrpcService;
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("clean-service", 8095, 9087);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let app_state = AppState::new(pool.clone());
    let grpc_service = CleanGrpcService::new(Arc::new(app_state));
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}
