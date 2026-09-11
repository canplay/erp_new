//! audit-service — gRPC 审计日志服务 + HTTP 健康检查
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9010 | 内部服务通信
//! | HTTP | 8089 | 健康检查端点
//!

use std::sync::Arc;
use audit_service::grpc_handlers::{AuditAppState, AuditGrpcService};
use audit_service::AuditRepository;
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("audit-service", 8089, 9010);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let state = Arc::new(AuditAppState { repository: AuditRepository::new(pool.clone()) });
    let grpc_service = AuditGrpcService::new(state);
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}
