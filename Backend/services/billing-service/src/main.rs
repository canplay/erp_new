//! 计费服务 - gRPC + HTTP 健康检查
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9101 | 内部服务通信 |
//! | HTTP | 8088 | 健康检查端点 |

use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use std::sync::Arc;
use billing_service::BillingAppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("billing-service", 8088, 9101);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:***@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建应用状态
    let _state = Arc::new(BillingAppState::new(pool.clone()));
    let _http_router = health_routes();

    // billing-service 目前只有 HTTP 端点，gRPC 使用 no-op
    bootstrap.start_with_grpc_fn(move |_addr| {
        Box::pin(tokio::spawn(async move {
            tracing::info!("billing-service gRPC no-op (HTTP only)");
        }))
    }).await?;

    Ok(())
}
