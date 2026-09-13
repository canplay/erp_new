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
    let state = Arc::new(BillingAppState::new(pool.clone()));
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(state.clone())),
        || Ok(http_router),
    ).await?;

    Ok(())
}
