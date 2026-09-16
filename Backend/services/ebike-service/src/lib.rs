//! 电动自行车管理系统服务库
//!
//! 提供车辆管理、仓储管理、订单管理及用户认证功能。
//! 使用 `PostgreSQL` 存储数据，JWT 进行身份验证。
//!
//! # 注意
//! 本服务仅通过 gRPC 对外提供接口，HTTP 端口已废弃。
//! 所有 HTTP 路由代码保留为内部兼容，不再启动 axum server。

pub mod db;
pub mod ebike_server;
pub mod grpc_server;
pub mod model;
pub mod route;

use auth_core::JwtService;
use axum::Router;
use db::{CarRepository, OrderRepository, OptionsRepository, StorageRepository, UserRepository};
use route::{
    auth::{login, loginout},
    car::car,
    options::options,
    order::order,
    storage::storage,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Unified state for all routes
#[derive(Clone)]
pub struct AppState {
    pub car_repo: Arc<CarRepository>,
    pub storage_repo: Arc<StorageRepository>,
    pub order_repo: Arc<OrderRepository>,
    pub options_repo: Arc<OptionsRepository>,
    pub user_repo: Arc<UserRepository>,
    pub jwt_service: JwtService,
    /// 停放区缓存：避免每次新增车辆都查询所有停放区
    pub storage_cache: Arc<RwLock<Vec<crate::model::StorageInfo>>>,
}

impl AppState {
    #[must_use]
    pub fn new(pool: sqlx::PgPool, jwt_service: JwtService) -> Self {
        Self {
            car_repo: Arc::new(CarRepository::new(pool.clone())),
            storage_repo: Arc::new(StorageRepository::new(pool.clone())),
            order_repo: Arc::new(OrderRepository::new(pool.clone())),
            options_repo: Arc::new(OptionsRepository::new(pool.clone())),
            user_repo: Arc::new(UserRepository::new(pool)),
            jwt_service,
            storage_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 刷新停放区缓存（storage 增删后调用）
    pub async fn refresh_storage_cache(&self) {
        let mut cache = self.storage_cache.write().await;
        if let Ok(storages) = self.storage_repo.query("", "", -1).await {
            *cache = storages;
        }
    }
}

/// HTTP 路由已废弃，仅保留为内部兼容（所有流量经 api-gateway 转发至 gRPC）
#[deprecated(since = "0.1.0", note = "HTTP 路由已废弃，请使用 api-gateway 的 gRPC 调用")]
pub fn create_app(state: AppState) -> Router {
    Router::new()
        // All routes under /api/v1/ebike/ unified path
        .route("/api/v1/ebike/login", axum::routing::post(login))
        .route("/api/v1/ebike/info", axum::routing::get(route::auth::info))
        .route("/api/v1/ebike/loginout", axum::routing::get(loginout))
        .route("/api/v1/ebike/car", axum::routing::post(car))
        .route("/api/v1/ebike/storage", axum::routing::post(storage))
        .route("/api/v1/ebike/order", axum::routing::post(order))
        .route("/api/v1/ebike/options", axum::routing::post(options))
        // State
        .with_state(state)
}

// Connection pool configuration constants (可通过环境变量覆盖，避免耗尽数据库连接)
fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub async fn create_optimized_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let url = common::config::ServiceDatabaseConfig::resolve_url("ebike-service");
    let max_connections = env_u32("DB_POOL_MAX_CONNECTIONS", 15);
    let min_connections = env_u32("DB_POOL_MIN_CONNECTIONS", 1);
    let acquire_timeout = env_u64("DB_POOL_CONNECT_TIMEOUT", 30);
    let idle_timeout = env_u64("DB_POOL_IDLE_TIMEOUT", 900);
    let max_lifetime = env_u64("DB_POOL_MAX_LIFETIME", 3600);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(acquire_timeout))
        .idle_timeout(Duration::from_secs(idle_timeout))
        .max_lifetime(Duration::from_secs(max_lifetime))
        .connect(&url)
        .await
}

/// 创建 `AppState`
#[must_use]
pub fn create_state(pool: sqlx::PgPool) -> AppState {
    // 审计修复 (B6): 删除硬编码兜底密钥, 缺失即启动失败(fail-fast)
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET 环境变量必须配置");
    let jwt_issuer = std::env::var("JWT_ISSUER")
        .unwrap_or_else(|_| "myai".to_string()); // 审计修复 B7: 统一默认值
    let jwt_audience = std::env::var("JWT_AUDIENCE")
        .unwrap_or_else(|_| "myai-users".to_string());
    let access_expiry = std::env::var("JWT_ACCESS_EXPIRY_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(86400);
    let refresh_expiry = std::env::var("JWT_REFRESH_EXPIRY_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(604800);

    let jwt_service = JwtService::new(
        &jwt_secret,
        &jwt_issuer,
        &jwt_audience,
        access_expiry,
        refresh_expiry,
    );

    AppState::new(pool, jwt_service)
}
