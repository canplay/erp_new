//! tow-service - 拖车服务入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9086 | 内部服务通信
//! | HTTP | 8094 | 健康检查端点
//!

use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use tow_service::grpc::TowGrpcService;
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
    let config = ServiceConfig::from_env("tow-service" , 8094, 9086);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL" )
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let grpc_service = TowGrpcService::new(pool.clone());
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}
