//! 车牌识别服务库 (lpr-service)
//!
//! 提供车牌识别相机回调处理、通行记录管理、车辆授权查询及开闸联动。
//!
//! # 业务流
//!
//! ```text
//! [Vz车牌相机] ──HTTP POST──→ [api-gateway] ──gRPC──→ [lpr-service]
//!                                                       ├─ 记录通行记录到数据库
//!                                                       ├─ 查询 hik-service 车辆授权
//!                                                       ├─ 入场授权车辆 → Signo开闸
//!                                                       └─ 出场计算时长 → 开闸放行
//! ```
//!
//! # 环境变量
//!
//! | 变量 | 默认值 | 说明 |
//! |------|--------|------|
//! | `DATABASE_URL` | `postgres://postgres:***@localhost:5432/myai` | PG 连接串 |
//! | `HIK_SERVICE_URL` | `http://localhost:8092` | hik-service HTTP 地址 |
//! | `REDIS_URL` | `redis://127.0.0.1:6379` | Redis 地址 |

pub use common::AppError;
pub use common::AppResult;

pub mod grpc;
pub mod grpc_server;
pub mod pass_record;
pub mod pass_service;

pub use pass_record::*;
pub use pass_service::PassService;

use axum::{Router, routing::get};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;

/// gRPC 应用状态
#[derive(Clone)]
pub struct GrpcAppState {
    pub pass_service: Arc<PassService>,
}

/// HTTP 应用状态
#[derive(Clone)]
pub struct AppState;

/// 创建 HTTP 应用（仅健康检查）
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health" , get(|| async { "OK" }))
        .with_state(state)
}

/// 创建 HTTP 服务状态
#[must_use]
pub fn create_state() -> AppState {
    AppState
}

/// 创建数据库连接池
pub async fn create_db_pool() -> Result<PgPool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL" ).unwrap_or_else(|_| {
            "postgres://postgres:***@localhost:5432/myai".to_string()
        });
    PgPool::connect(&database_url).await
}

/// 创建 Redis 连接（可选）
pub async fn create_redis_conn() -> Option<ConnectionManager> {
    let redis_url =
        std::env::var("REDIS_URL" ).unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    match redis::Client::open(redis_url.as_str()) {
        Ok(client) => match client.get_connection_manager().await {
            Ok(conn) => {
                tracing::info!("Redis 连接成功" );
                Some(conn)
            }
            Err(e) => {
                tracing::warn!("Redis 连接失败，将以无缓存模式运行: {e}" );
                None
            }
        },
        Err(e) => {
            tracing::warn!("Redis URL 无效，将以无缓存模式运行: {e}" );
            None
        }
    }
}

/// 创建通行服务实例
pub async fn create_pass_service() -> Result<Arc<PassService>, sqlx::Error> {
    let db_pool = create_db_pool().await?;
    let redis_conn = create_redis_conn().await;
    Ok(Arc::new(PassService::new(db_pool, redis_conn)))
}

/// 创建 gRPC 服务实现
pub async fn create_grpc_service() -> Result<crate::grpc::LprGrpcService, sqlx::Error> {
    let pass_service = create_pass_service().await?;
    Ok(crate::grpc::LprGrpcService::new(pass_service))
}
