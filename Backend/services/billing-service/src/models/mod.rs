//! 计费模型 - 包含状态机流转和配额检查

// Re-export billing-core types
pub use billing_core::{
    BillingPlan, BillingPlanType, Invoice, InvoiceLineItem, InvoiceStatus,
    PlanFeature, PlanStatus, Subscription, SubscriptionStatus, UsageRecord, UsageType,
};

/// 订阅状态机
pub struct SubscriptionStateMachine;

impl SubscriptionStateMachine {
    /// 试用 -> 活跃（支付成功）
    pub fn activate(subscription: &mut Subscription) -> Result<(), String> {
        if subscription.status == SubscriptionStatus::Trialing {
            subscription.status = SubscriptionStatus::Active;
            subscription.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Only trialing subscriptions can be activated".to_string())
        }
    }

    /// 试用/活跃 -> 逾期
    pub fn mark_past_due(subscription: &mut Subscription) -> Result<(), String> {
        if matches!(
            subscription.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trialing
        ) {
            subscription.status = SubscriptionStatus::PastDue;
            subscription.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Only active or trialing subscriptions can be marked past due".to_string())
        }
    }

    /// 逾期 -> 活跃（支付成功）
    pub fn reactivate(subscription: &mut Subscription) -> Result<(), String> {
        if subscription.status == SubscriptionStatus::PastDue {
            subscription.status = SubscriptionStatus::Active;
            subscription.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Only past due subscriptions can be reactivated".to_string())
        }
    }

    /// 任何状态 -> 已取消
    pub fn cancel(subscription: &mut Subscription) -> Result<(), String> {
        if subscription.status != SubscriptionStatus::Cancelled {
            subscription.status = SubscriptionStatus::Cancelled;
            subscription.canceled_at = Some(chrono::Utc::now());
            subscription.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Subscription is already cancelled".to_string())
        }
    }

    /// 活跃/逾期 -> 已过期
    pub fn expire(subscription: &mut Subscription) -> Result<(), String> {
        if matches!(
            subscription.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trialing | SubscriptionStatus::PastDue
        ) {
            subscription.status = SubscriptionStatus::Expired;
            subscription.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Only active, trialing, or past due subscriptions can expire".to_string())
        }
    }
}

/// 配额检查结果
#[derive(Debug, Clone)]
pub struct QuotaCheckResult {
    pub metric: String,
    pub current_usage: f64,
    pub quota_limit: i64,
    pub exceeded: bool,
    pub overage: f64,
}

impl QuotaCheckResult {
    pub fn new(metric: impl Into<String>, current_usage: f64, quota_limit: i64) -> Self {
        let exceeded = current_usage > quota_limit as f64;
        let overage = if exceeded {
            current_usage - quota_limit as f64
        } else {
            0.0
        };
        Self {
            metric: metric.into(),
            current_usage,
            quota_limit,
            exceeded,
            overage,
        }
    }
}

/// 用量聚合
#[derive(Debug, Clone)]
pub struct UsageSummary {
    pub metric: String,
    pub total_quantity: f64,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}
