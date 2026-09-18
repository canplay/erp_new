//! HTTP 请求处理器

mod pay_service;

pub use pay_service::*;

use std::sync::Arc;

use sqlx::PgPool;
use common::AppError;
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
    pub fn new() -> Result<Self, AppError> {
        let pool = PgPool::connect_lazy(&std::env::var("DATABASE_URL" ).unwrap_or_default())
            .map_err(AppError::Database)?;
        Ok(Self {
            pool: pool.clone(),
            pay_service: Arc::new(PayService::new(pool, std::env::var("REDIS_URL" ).unwrap_or_default())),
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("Failed to create AppState" )
    }
}
