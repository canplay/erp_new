//! 数据库仓库通用接口

use sqlx::postgres::PgPool;

/// 通用数据仓库 trait
pub trait GenericRepository: Send + Sync {
    type Model;
    fn pool(&self) -> &PgPool;
}
