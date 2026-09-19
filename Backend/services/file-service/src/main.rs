//! file-service — gRPC + HTTP 健康检查
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9084 | 内部服务通信
//! | HTTP | 8086 | 健康检查端点
//!

use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use file_service::grpc_handlers::{{FileAppState, FileGrpcService}};
use file_service::FileRepository;
use std::sync::Arc;
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
    let config = ServiceConfig::from_env("file-service" , 8086, 9084);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL" )
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let repository = FileRepository::new(pool.clone());
    let grpc_app_state = Arc::new(FileAppState { repository });
    let grpc_service = FileGrpcService::new(grpc_app_state);
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}
