//! 角色仓储层

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 角色仓储错误类型
#[derive(Error, Debug)]
pub(crate) enum RoleRepositoryError {
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("角色已存在" )]
    AlreadyExists,

    #[error("角色有关联用户，无法删除" )]
    HasAssociatedUsers,

    #[error("角色有子角色，无法删除" )]
    HasChildRoles,
}

/// 角色信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Role {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub role_type: String,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub status: i32,
    pub is_default: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// 角色列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RoleListItem {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub role_type: String,
    pub level: i32,
    pub status: i32,
    pub is_default: bool,
    pub user_count: i64,
    pub created_at: chrono::DateTime<Utc>,
}

/// 权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Permission {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub permission_type: String,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i32,
}

/// 分页结果
pub(crate) struct PaginatedRoles {
    pub roles: Vec<RoleListItem>,
    pub total: i64,
}
