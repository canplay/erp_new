//! 公告/系统配置/登录日志/数据字典仓储模块
//!
//! 从 announcement_repository.rs (1008行) 按业务域拆分

mod announcement_ops;
mod config_ops;
mod dictionary_item_ops;
mod dictionary_type_ops;
mod log_ops;
mod types;

pub(crate) use types::{
    CreateAnnouncementParams,
    CreateDictionaryItemParams,
    UpdateAnnouncementParams, UpdateDictionaryItemParams,
};

use sqlx::PgPool;

/// 公告和配置仓储
#[derive(Clone)]
pub(crate) struct AnnouncementRepository {
    pub(crate) pool: PgPool,
}

impl AnnouncementRepository {
    /// 创建新的仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
