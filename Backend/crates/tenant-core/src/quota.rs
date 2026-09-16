//! 配额管理
//!
//! 定义租户资源配额（用户数、存储、API调用次数等），
//! 并提供配额检查和用量跟踪功能。

use crate::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 配额类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuotaType {
    /// 最大用户数
    MaxUsers,
    /// 最大存储空间 (GB)
    MaxStorageGb,
    /// 每月 API 调用次数
    MonthlyApiCalls,
    /// 每月文件上传次数
    MonthlyFileUploads,
    /// 单文件大小限制 (MB)
    MaxFileSizeMb,
    /// 最大群组成员数
    MaxGroupMembers,
    /// 最大群组数
    MaxGroups,
    /// 最大消息保留天数
    MessageRetentionDays,
    /// 最大频道数
    MaxChannels,
    /// 最大角色数
    MaxRoles,
    /// 最大权限数
    MaxPermissions,
}

impl QuotaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MaxUsers => "max_users" ,
            Self::MaxStorageGb => "max_storage_gb" ,
            Self::MonthlyApiCalls => "monthly_api_calls" ,
            Self::MonthlyFileUploads => "monthly_file_uploads" ,
            Self::MaxFileSizeMb => "max_file_size_mb" ,
            Self::MaxGroupMembers => "max_group_members" ,
            Self::MaxGroups => "max_groups" ,
            Self::MessageRetentionDays => "message_retention_days" ,
            Self::MaxChannels => "max_channels" ,
            Self::MaxRoles => "max_roles" ,
            Self::MaxPermissions => "max_permissions" ,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "max_users" => Some(Self::MaxUsers),
            "max_storage_gb" => Some(Self::MaxStorageGb),
            "monthly_api_calls" => Some(Self::MonthlyApiCalls),
            "monthly_file_uploads" => Some(Self::MonthlyFileUploads),
            "max_file_size_mb" => Some(Self::MaxFileSizeMb),
            "max_group_members" => Some(Self::MaxGroupMembers),
            "max_groups" => Some(Self::MaxGroups),
            "message_retention_days" => Some(Self::MessageRetentionDays),
            "max_channels" => Some(Self::MaxChannels),
            "max_roles" => Some(Self::MaxRoles),
            "max_permissions" => Some(Self::MaxPermissions),
            _ => None,
        }
    }

    /// 获取默认单位
    pub fn unit(&self) -> &'static str {
        match self {
            Self::MaxUsers => "users" ,
            Self::MaxStorageGb => "GB" ,
            Self::MonthlyApiCalls => "calls/month" ,
            Self::MonthlyFileUploads => "uploads/month" ,
            Self::MaxFileSizeMb => "MB" ,
            Self::MaxGroupMembers => "members" ,
            Self::MaxGroups => "groups" ,
            Self::MessageRetentionDays => "days" ,
            Self::MaxChannels => "channels" ,
            Self::MaxRoles => "roles" ,
            Self::MaxPermissions => "permissions" ,
        }
    }
}

/// 配额限制
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuotaLimit {
    pub quota_type: QuotaType,
    pub limit: u64,
    pub used: u64,
    pub warning_threshold: f64, // 0.0-1.0, 达到此百分比时发出警告
}

impl QuotaLimit {
    /// 创建新的配额限制
    pub fn new(quota_type: QuotaType, limit: u64) -> Self {
        Self {
            quota_type,
            limit,
            used: 0,
            warning_threshold: 0.8,
        }
    }

    /// 设置警告阈值
    pub fn with_warning_threshold(mut self, threshold: f64) -> Self {
        self.warning_threshold = threshold;
        self
    }

    /// 检查是否超限
    pub fn is_exceeded(&self) -> bool {
        self.used >= self.limit
    }

    /// 检查是否接近限制（达到警告阈值）
    pub fn is_near_limit(&self) -> bool {
        if self.limit == 0 {
            return false;
        }
        let ratio = self.used as f64 / self.limit as f64;
        ratio >= self.warning_threshold && !self.is_exceeded()
    }

    /// 获取剩余配额
    pub fn remaining(&self) -> u64 {
        if self.used >= self.limit {
            0
        } else {
            self.limit - self.used
        }
    }

    /// 获取使用百分比
    pub fn usage_percentage(&self) -> f64 {
        if self.limit == 0 {
            0.0
        } else {
            (self.used as f64 / self.limit as f64) * 100.0
        }
    }

    /// 增加用量
    pub fn increment(&mut self, amount: u64) -> bool {
        if self.used + amount > self.limit {
            false
        } else {
            self.used += amount;
            true
        }
    }

    /// 减少用量
    pub fn decrement(&mut self, amount: u64) {
        self.used = self.used.saturating_sub(amount);
    }
}

/// 配额集合
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuotaSet {
    pub tenant_id: TenantId,
    pub quotas: HashMap<String, QuotaLimit>,
    pub updated_at: DateTime<Utc>,
}

impl QuotaSet {
    /// 创建新的配额集合
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            quotas: HashMap::new(),
            updated_at: Utc::now(),
        }
    }

    /// 添加配额
    pub fn add_quota(&mut self, quota: QuotaLimit) {
        self.quotas.insert(quota.quota_type.as_str().to_string(), quota);
        self.updated_at = Utc::now();
    }

    /// 获取配额
    pub fn get(&self, quota_type: QuotaType) -> Option<&QuotaLimit> {
        self.quotas.get(quota_type.as_str())
    }

    /// 获取可变配额
    pub fn get_mut(&mut self, quota_type: QuotaType) -> Option<&mut QuotaLimit> {
        self.quotas.get_mut(quota_type.as_str())
    }

    /// 检查配额是否足够
    pub fn check(&self, quota_type: QuotaType, requested: u64) -> QuotaCheckResult {
        match self.get(quota_type) {
            Some(quota) => {
                if quota.remaining() >= requested {
                    QuotaCheckResult::Available
                } else {
                    QuotaCheckResult::Exceeded {
                        requested,
                        available: quota.remaining(),
                    }
                }
            }
            None => QuotaCheckResult::Available, // 未设置配额 = 无限
        }
    }

    /// 使用配额
    pub fn consume(&mut self, quota_type: QuotaType, amount: u64) -> bool {
        match self.get_mut(quota_type) {
            Some(quota) => quota.increment(amount),
            None => true, // 未设置配额 = 无限
        }
    }

    /// 释放配额
    pub fn release(&mut self, quota_type: QuotaType, amount: u64) {
        if let Some(quota) = self.get_mut(quota_type) {
            quota.decrement(amount);
        }
    }

    /// 获取所有接近限制的配额
    pub fn get_near_limit(&self) -> Vec<&QuotaLimit> {
        self.quotas
            .values()
            .filter(|q| q.is_near_limit())
            .collect()
    }

    /// 获取所有超限的配额
    pub fn get_exceeded(&self) -> Vec<&QuotaLimit> {
        self.quotas
            .values()
            .filter(|q| q.is_exceeded())
            .collect()
    }
}

/// 配额检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaCheckResult {
    /// 可用
    Available,
    /// 超限
    Exceeded {
        /// 请求量
        requested: u64,
        /// 可用量
        available: u64,
    },
}

/// 配额预设（按套餐）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaPreset {
    pub name: String,
    pub quotas: Vec<QuotaLimit>,
}

impl QuotaPreset {
    /// 免费套餐配额
    pub fn free() -> Self {
        Self {
            name: "Free".to_string(),
            quotas: vec![
                QuotaLimit::new(QuotaType::MaxUsers, 5),
                QuotaLimit::new(QuotaType::MaxStorageGb, 1),
                QuotaLimit::new(QuotaType::MonthlyApiCalls, 10000),
                QuotaLimit::new(QuotaType::MonthlyFileUploads, 100),
                QuotaLimit::new(QuotaType::MaxFileSizeMb, 10),
                QuotaLimit::new(QuotaType::MaxGroupMembers, 100),
                QuotaLimit::new(QuotaType::MaxGroups, 5),
                QuotaLimit::new(QuotaType::MessageRetentionDays, 30),
            ],
        }
    }

    /// 标准套餐配额
    pub fn standard() -> Self {
        Self {
            name: "Standard".to_string(),
            quotas: vec![
                QuotaLimit::new(QuotaType::MaxUsers, 50),
                QuotaLimit::new(QuotaType::MaxStorageGb, 10),
                QuotaLimit::new(QuotaType::MonthlyApiCalls, 100000),
                QuotaLimit::new(QuotaType::MonthlyFileUploads, 1000),
                QuotaLimit::new(QuotaType::MaxFileSizeMb, 50),
                QuotaLimit::new(QuotaType::MaxGroupMembers, 500),
                QuotaLimit::new(QuotaType::MaxGroups, 20),
                QuotaLimit::new(QuotaType::MessageRetentionDays, 90),
            ],
        }
    }

    /// 企业套餐配额
    pub fn enterprise() -> Self {
        Self {
            name: "Enterprise".to_string(),
            quotas: vec![
                QuotaLimit::new(QuotaType::MaxUsers, 500),
                QuotaLimit::new(QuotaType::MaxStorageGb, 100),
                QuotaLimit::new(QuotaType::MonthlyApiCalls, 1000000),
                QuotaLimit::new(QuotaType::MonthlyFileUploads, 10000),
                QuotaLimit::new(QuotaType::MaxFileSizeMb, 200),
                QuotaLimit::new(QuotaType::MaxGroupMembers, 5000),
                QuotaLimit::new(QuotaType::MaxGroups, 100),
                QuotaLimit::new(QuotaType::MessageRetentionDays, 365),
            ],
        }
    }
}

/// 用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub tenant_id: TenantId,
    pub metric: String,
    pub quantity: f64,
    pub recorded_at: DateTime<Utc>,
}

/// 用量聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAggregation {
    pub tenant_id: TenantId,
    pub metric: String,
    pub period: String, // daily, monthly, yearly
    pub total: f64,
    pub average: f64,
    pub peak: f64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}
