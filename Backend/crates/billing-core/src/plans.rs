//! 计费计划管理

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 计费计划类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingPlanType {
    /// 免费
    Free,
    /// 标准
    Standard,
    /// 企业
    Enterprise,
    /// 自定义
    Custom,
}

impl BillingPlanType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Standard => "standard",
            Self::Enterprise => "enterprise",
            Self::Custom => "custom",
        }
    }
}

/// 计费计划状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    /// 草稿
    Draft,
    /// 活跃（可订阅）
    Active,
    /// 已归档
    Archived,
}

/// 计费计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPlan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: BillingPlanType,
    pub status: PlanStatus,
    /// 月付价格
    pub price_monthly: Decimal,
    /// 年付价格
    pub price_yearly: Decimal,
    /// 货币
    pub currency: String,
    /// 功能特性列表
    pub features: Vec<PlanFeature>,
    /// 配额限制（键值对）
    pub quotas: HashMap<String, i64>,
    /// 排序权重
    pub sort_order: i32,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BillingPlan {
    /// 创建新的计费计划
    pub fn new(name: impl Into<String>, plan_type: BillingPlanType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            plan_type,
            status: PlanStatus::Draft,
            price_monthly: Decimal::ZERO,
            price_yearly: Decimal::ZERO,
            currency: "CNY".to_string(),
            features: Vec::new(),
            quotas: HashMap::new(),
            sort_order: 0,
            is_public: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 设置月付价格
    pub fn with_monthly_price(mut self, price: Decimal) -> Self {
        self.price_monthly = price;
        self
    }

    /// 设置年付价格
    pub fn with_yearly_price(mut self, price: Decimal) -> Self {
        self.price_yearly = price;
        self
    }

    /// 添加功能特性
    pub fn with_feature(mut self, feature: PlanFeature) -> Self {
        self.features.push(feature);
        self
    }

    /// 添加配额限制
    pub fn with_quota(mut self, key: impl Into<String>, value: i64) -> Self {
        self.quotas.insert(key.into(), value);
        self
    }

    /// 计算年付折扣百分比
    pub fn yearly_discount_percent(&self) -> Option<Decimal> {
        if self.price_monthly > Decimal::ZERO && self.price_yearly > Decimal::ZERO {
            let yearly_monthly = self.price_monthly * Decimal::from(12);
            if yearly_monthly > self.price_yearly {
                let discount = (yearly_monthly - self.price_yearly) / yearly_monthly
                    * Decimal::from(100);
                Some(discount.round_dp(2))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 获取指定周期的价格
    pub fn price_for_period(&self, is_yearly: bool) -> Decimal {
        if is_yearly {
            self.price_yearly
        } else {
            self.price_monthly
        }
    }
}

/// 计划功能特性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFeature {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    /// 特性值（如存储容量、用户数等）
    pub value: Option<String>,
}

/// 创建默认免费计划
pub fn create_free_plan() -> BillingPlan {
    BillingPlan::new("Free", BillingPlanType::Free)
        .with_monthly_price(Decimal::ZERO)
        .with_yearly_price(Decimal::ZERO)
        .with_feature(PlanFeature {
            name: "basic_chat".to_string(),
            description: Some("基础聊天功能".to_string()),
            enabled: true,
            value: None,
        })
        .with_feature(PlanFeature {
            name: "file_sharing".to_string(),
            description: Some("文件分享".to_string()),
            enabled: true,
            value: Some("10MB".to_string()),
        })
        .with_quota("max_users", 5)
        .with_quota("max_storage_gb", 1)
        .with_quota("monthly_api_calls", 10000)
}

/// 创建默认标准计划
pub fn create_standard_plan() -> BillingPlan {
    BillingPlan::new("Standard", BillingPlanType::Standard)
        .with_monthly_price(Decimal::from(99))
        .with_yearly_price(Decimal::from(999))
        .with_feature(PlanFeature {
            name: "advanced_chat".to_string(),
            description: Some("高级聊天功能".to_string()),
            enabled: true,
            value: None,
        })
        .with_feature(PlanFeature {
            name: "voice_video".to_string(),
            description: Some("语音视频通话".to_string()),
            enabled: true,
            value: None,
        })
        .with_feature(PlanFeature {
            name: "file_sharing".to_string(),
            description: Some("文件分享".to_string()),
            enabled: true,
            value: Some("100MB".to_string()),
        })
        .with_quota("max_users", 50)
        .with_quota("max_storage_gb", 10)
        .with_quota("monthly_api_calls", 100000)
}

/// 创建默认企业计划
pub fn create_enterprise_plan() -> BillingPlan {
    BillingPlan::new("Enterprise", BillingPlanType::Enterprise)
        .with_monthly_price(Decimal::from(499))
        .with_yearly_price(Decimal::from(4999))
        .with_feature(PlanFeature {
            name: "all_features".to_string(),
            description: Some("全部功能".to_string()),
            enabled: true,
            value: None,
        })
        .with_feature(PlanFeature {
            name: "priority_support".to_string(),
            description: Some("优先技术支持".to_string()),
            enabled: true,
            value: None,
        })
        .with_feature(PlanFeature {
            name: "custom_branding".to_string(),
            description: Some("自定义品牌".to_string()),
            enabled: true,
            value: None,
        })
        .with_quota("max_users", 500)
        .with_quota("max_storage_gb", 100)
        .with_quota("monthly_api_calls", 1000000)
}
