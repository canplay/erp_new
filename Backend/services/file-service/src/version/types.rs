//! 文件版本管理模块
//!
//! 实现文件版本历史、版本对比、版本回退功能
//! 支持多版本存储、增量存储、版本标签管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 文件版本状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VersionState {
    /// 草稿版本
    Draft,
    /// 正式版本
    #[default]
    Active,
    /// 已归档
    Archived,
    /// 已删除
    Deleted,
}

/// 文件版本元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub version_id: String,
    pub file_id: String,
    pub version_number: u32,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub content_hash: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub state: VersionState,
    pub description: Option<String>,
    pub tags: Vec<String>,
    /// 增量大小（相对于前一版本）
    pub delta_size: Option<u64>,
    /// 是否为初始版本
    pub is_initial: bool,
}

/// 版本差异信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: String,
    pub to_version: String,
    pub diff_type: DiffType,
    pub added_bytes: u64,
    pub removed_bytes: u64,
    pub net_change: i64,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// 完全新增
    Added,
    /// 完全删除
    Removed,
    /// 部分修改
    Modified,
    /// 无变化
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub change_type: String,
    pub position: Option<u64>,
    pub content: Option<String>,
}

/// 版本历史查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistoryQuery {
    pub file_id: String,
    pub from_version: Option<u32>,
    pub to_version: Option<u32>,
    pub state: Option<VersionState>,
    pub created_by: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 版本比较配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareConfig {
    /// 允许比较的最大版本跨度
    pub max_version_span: u32,
    /// 是否启用增量比较
    pub enable_delta_compare: bool,
    /// 最大缓存版本数
    pub max_cached_versions: usize,
    /// 版本保留策略（天数）
    pub retention_days: u64,
}

impl Default for CompareConfig {
    fn default() -> Self {
        Self {
            max_version_span: 100,
            enable_delta_compare: true,
            max_cached_versions: 50,
            retention_days: 365,
        }
    }
}

/// 版本管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    /// 启用版本管理
    pub enabled: bool,
    /// 自动创建版本间隔（秒）
    pub auto_save_interval: u64,
    /// 最大版本数（0 = 无限制）
    pub max_versions: usize,
    /// 启用增量存储
    pub enable_delta_storage: bool,
    /// 版本比较配置
    pub compare: CompareConfig,
    /// 清理过期版本
    pub auto_cleanup: bool,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_save_interval: 300,
            max_versions: 0,
            enable_delta_storage: true,
            compare: CompareConfig::default(),
            auto_cleanup: true,
        }
    }
}

/// 版本回退请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub file_id: String,
    pub target_version: u32,
    pub reason: Option<String>,
    pub create_backup: bool,
}

/// 版本回退结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub new_version_id: String,
    pub new_version_number: u32,
    pub backup_version_id: Option<String>,
    pub message: String,
}

/// 文件版本管理器
pub struct VersionManager {
    /// `版本存储（file_id` -> Vec<FileVersion>）
    versions: Arc<RwLock<HashMap<String, Vec<FileVersion>>>>,
    /// 当前版本快照
    snapshots: Arc<RwLock<HashMap<String, FileVersion>>>,
    /// 配置
    config: VersionConfig,
}

/// 创建新版本参数
#[derive(Debug, Clone)]
pub struct CreateVersionParams {
    pub file_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub content_hash: String,
    pub storage_path: String,
    pub created_by: String,
    pub description: Option<String>,
}


/// 版本统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionStats {
    pub total_files: usize,
    pub total_versions: usize,
    pub total_size: u64,
    pub active_count: usize,
    pub archived_count: usize,
}
