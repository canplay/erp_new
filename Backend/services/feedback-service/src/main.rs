//! feedback-service — gRPC + HTTP 健康检查
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9085 | 内部服务通信
//! | HTTP | 8085 | 健康检查端点
//!

use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use feedback_service::grpc_handlers::{{FeedbackAppState, FeedbackGrpcService}};
use feedback_service::repository::FeedbackRepository;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("feedback-service", 8085, 9085);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let repository = FeedbackRepository::new(pool.clone());
    let grpc_app_state = Arc::new(FeedbackAppState::new(repository));
    let grpc_service = FeedbackGrpcService::new(grpc_app_state);
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service) as Box<dyn common::service_bootstrap::GrpcServiceBuilder>),
        || Ok(http_router),
    ).await?;

    Ok(())
}
