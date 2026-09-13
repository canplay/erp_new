//! 租户初始化与配置管理
//!
//! 负责租户创建后的初始化流程，包括数据库 schema 创建、
//! 默认数据填充、管理员账户创建等。

use crate::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 租户初始化状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProvisioningState {
    /// 待初始化
    #[default]
    Pending,
    /// 初始化中
    InProgress,
    /// 数据库创建中
    CreatingDatabase,
    /// 迁移执行中
    RunningMigrations,
    /// 默认数据填充中
    SeedingData,
    /// 完成
    Completed,
    /// 失败
    Failed,
}

impl ProvisioningState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::CreatingDatabase => "creating_database",
            Self::RunningMigrations => "running_migrations",
            Self::SeedingData => "seeding_data",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// 租户初始化步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningStep {
    pub name: String,
    pub status: ProvisioningStepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisioningStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// 租户初始化请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub admin_email: String,
    pub admin_username: String,
    pub plan_id: String,
    pub metadata: HashMap<String, String>,
}

/// 租户初始化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningResult {
    pub tenant_id: TenantId,
    pub state: ProvisioningState,
    pub steps: Vec<ProvisioningStep>,
    pub admin_user_id: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// 租户数据库迁移状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub tenant_id: TenantId,
    pub current_version: String,
    pub target_version: String,
    pub pending_migrations: Vec<String>,
    pub applied_migrations: Vec<String>,
    pub last_migration_at: Option<DateTime<Utc>>,
}

/// 租户数据库迁移记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub version: String,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// 租户默认数据种子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedData {
    pub category: String,
    pub items: Vec<SeedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedItem {
    pub name: String,
    pub data: serde_json::Value,
    pub dependencies: Vec<String>,
}

/// 租户初始化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningConfig {
    /// 是否自动创建数据库
    pub auto_create_database: bool,
    /// 是否执行迁移
    pub run_migrations: bool,
    /// 是否填充默认数据
    pub seed_default_data: bool,
    /// 默认数据类别
    pub seed_categories: Vec<String>,
    /// 是否创建管理员账户
    pub create_admin: bool,
}

impl Default for ProvisioningConfig {
    fn default() -> Self {
        Self {
            auto_create_database: true,
            run_migrations: true,
            seed_default_data: true,
            seed_categories: vec![
                "roles".to_string(),
                "permissions".to_string(),
                "settings".to_string(),
            ],
            create_admin: true,
        }
    }
}

/// 租户销毁请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprovisioningRequest {
    pub tenant_id: TenantId,
    pub reason: String,
    pub backup_before_delete: bool,
    pub delete_database: bool,
}

/// 租户销毁结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprovisioningResult {
    pub tenant_id: TenantId,
    pub success: bool,
    pub backup_path: Option<String>,
    pub error: Option<String>,
    pub completed_at: DateTime<Utc>,
}

/// 租户克隆请求（用于创建测试租户）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneTenantRequest {
    pub source_tenant_id: TenantId,
    pub target_tenant_name: String,
    pub target_admin_email: String,
    pub copy_data: bool,
    pub copy_users: bool,
    pub copy_files: bool,
}

/// 租户克隆结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneTenantResult {
    pub source_tenant_id: TenantId,
    pub target_tenant_id: TenantId,
    pub success: bool,
    pub error: Option<String>,
    pub completed_at: DateTime<Utc>,
}
