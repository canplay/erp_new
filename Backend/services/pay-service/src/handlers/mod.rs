//! HTTP 请求处理器

#![allow(dead_code)]

mod pay_service;

pub use pay_service::*;

use std::sync::Arc;

use sqlx::PgPool;
use crate::services::pay::PayService;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub pool: PgPool,
    /// 支付服务
    pub pay_service: Arc<PayService>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        let pool = PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
            .unwrap_or_else(|e| {
                tracing::error!("数据库连接失败: {e}");
                sqlx::PgPool::connect_lazy("postgres://localhost:5432/fallback").unwrap_or_else(|_| {
                    panic!("无法建立数据库连接: {e}")
                })
            });
        Self {
            pool: pool.clone(),
            pay_service: Arc::new(PayService::new(pool, std::env::var("REDIS_URL").unwrap_or_default())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
