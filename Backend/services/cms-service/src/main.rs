//! cms-service - CMS 内容管理服务入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9082 | 内部服务通信
//! | HTTP | 8082 | 健康检查端点
//!

use cms_service::{CmsAppState, CmsGrpcServer};
use common::cache::create_multi_level_cache;
use common::config::{MeilisearchConfig, RedisConfig};
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use search_core::{setup::setup_cms_indexes, SearchClient};
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
    let config = ServiceConfig::from_env("cms-service", 8082, 9082);

    let bootstrap = ServiceBootstrap::new(config);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/erp_new".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 初始化 Redis 缓存
    let redis_config = RedisConfig::default();
    let cache = create_multi_level_cache(&redis_config, 1000, 3600).await?;

    // 初始化 Meilisearch 搜索客户端
    let meili_config = MeilisearchConfig::default();
    let search = SearchClient::new(&meili_config.url, meili_config.api_key.as_deref())?;

    // 设置 CMS 搜索索引
    if let Err(e) = setup_cms_indexes(&search, &meili_config.index_prefix).await {
        tracing::warn!("Failed to setup CMS search indexes: {}", e);
    }

    // 创建 gRPC 服务
    let state = Arc::new(CmsAppState::new(pool.clone(), cache, search));
    let grpc_service = CmsGrpcServer::new(state);
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(grpc_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}