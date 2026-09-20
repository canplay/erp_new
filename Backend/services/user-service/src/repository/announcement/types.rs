//! 公告和系统配置仓储层

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 公告仓储错误类型
#[derive(Error, Debug)]
pub(crate) enum AnnouncementRepositoryError {
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("公告不存在" )]
    NotFound,
}

/// 公告信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Announcement {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub announcement_type: String,
    pub priority: i32,
    pub is_pinned: bool,
    pub is_active: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 公告列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AnnouncementListItem {
    pub id: i64,
    pub title: String,
    pub announcement_type: String,
    pub priority: i32,
    pub is_pinned: bool,
    pub is_active: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 系统配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SystemConfig {
    pub id: i64,
    pub category: String,
    pub config_key: String,
    pub config_value: Option<String>,
    pub value_type: String,
    pub label: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 登录日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoginLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub login_status: i32,
    pub fail_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 分页结果
pub(crate) struct PaginatedAnnouncements {
    pub announcements: Vec<AnnouncementListItem>,
    pub total: i64,
}

pub(crate) struct PaginatedLoginLogs {
    pub logs: Vec<LoginLog>,
    pub total: i64,
}

/// 数据字典类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DictionaryType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据字典项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DictionaryItem {
    pub id: i64,
    pub type_id: i64,
    pub label: String,
    pub value: String,
    pub sort: i32,
    pub status: i32,
    pub is_default: bool,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 字典类型分页
pub(crate) struct PaginatedDictionaryTypes {
    pub types: Vec<DictionaryType>,
    pub total: i64,
}

/// 字典项分页
pub(crate) struct PaginatedDictionaryItems {
    pub items: Vec<DictionaryItem>,
    pub total: i64,
}

/// 创建字典项参数
pub(crate) struct CreateDictionaryItemParams<'a> {
    pub type_id: i64,
    pub label: &'a str,
    pub value: &'a str,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub is_default: Option<bool>,
    pub remark: Option<&'a str>,
}

/// 更新字典项参数
pub(crate) struct UpdateDictionaryItemParams<'a> {
    pub id: i64,
    pub label: &'a str,
    pub value: &'a str,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub is_default: Option<bool>,
    pub remark: Option<&'a str>,
}

/// 创建公告参数
#[derive(Debug, Clone)]
pub(crate) struct CreateAnnouncementParams<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub announcement_type: &'a str,
    pub priority: i32,
    pub is_pinned: bool,
    pub is_active: bool,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<i64>,
}

/// 更新公告参数
#[derive(Debug, Clone)]
pub(crate) struct UpdateAnnouncementParams {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub announcement_type: Option<String>,
    pub priority: Option<i32>,
    pub is_pinned: Option<bool>,
    pub is_active: Option<bool>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}
