//! 角色仓储模块
//!
//! 从 role_repository.rs (951行) 按业务域拆分

mod data_perm_ops;
mod field_perm_ops;
mod inherit_ops;
mod role_ops;
mod types;

pub(crate) use types::RoleRepositoryError;

use sqlx::PgPool;

/// 角色仓储
#[derive(Clone)]
pub(crate) struct RoleRepository {
    pub(crate) pool: PgPool,
}

impl RoleRepository {
    /// 创建新的角色仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
