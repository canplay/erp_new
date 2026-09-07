//! Social Ops Service — 社交账号自动运维微服务
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9110 | 内部服务通信
//! | HTTP | 8110 | 健康检查端点
//!

use std::sync::Arc;
use social_ops_service::adapters::browser_client::BrowserClient;
use social_ops_service::services::{{
    account_service::AccountService,
    content_service::ContentService,
    crawl_service::CrawlService,
    publish_service::PublishService,
    llm_service::LlmService,
    rewrite_service::RewriteService,
}};
use social_ops_service::grpc_handlers::AppState;
use social_ops_service::grpc_server;
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("social-ops-service", 8110, 9110);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 自定义初始化
    let account_svc = AccountService::new(pool.clone());
    let content_svc = ContentService::new(pool.clone());
    let browser_client = Arc::new(BrowserClient::new());
    let crawl_svc = CrawlService::new(pool.clone(), browser_client);
    let publish_svc = PublishService::new(pool.clone());
    let llm_svc = LlmService::new(pool.clone());
    let rewrite_svc = RewriteService::new(pool.clone());

    let state = AppState::new(
        pool.clone(),
        account_svc, crawl_svc, content_svc,
        publish_svc, llm_svc, rewrite_svc,
    );
    let _http_router = health_routes();

    // 定义 gRPC 服务启动函数
    let grpc_service = |addr| {
        let handle = tokio::spawn(async move {
            let _ = grpc_server::start_grpc_server(addr, state.clone()).await;
        });
        Box::pin(handle)
    };
    bootstrap.start_with_grpc_fn(grpc_service).await?;

    Ok(())
}
