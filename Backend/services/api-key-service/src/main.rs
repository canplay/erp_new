//! api-key-service - gRPC + HTTP 健康检查入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9094 | 内部服务通信
//! | HTTP | 8091 | 健康检查端点
//!

use api_key_service::{{ApiKeyAppState, ApiKeyGrpcServer, PostgresApiKeyRepository}};
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("api-key-service" , 8091, 9094);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL" )
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let repository = Arc::new(PostgresApiKeyRepository::new(pool.clone()));
    let state = Arc::new(ApiKeyAppState::new(repository));
    let grpc_service = ApiKeyGrpcServer::new(state);
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}
