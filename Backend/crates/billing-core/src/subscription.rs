//! 订阅管理

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 订阅状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    /// 活跃
    Active,
    /// 已取消
    Cancelled,
    /// 已过期
    Expired,
    /// 试用中
    Trialing,
    /// 逾期未付
    PastDue,
    /// 未支付
    Unpaid,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active" ,
            Self::Cancelled => "cancelled" ,
            Self::Expired => "expired" ,
            Self::Trialing => "trialing" ,
            Self::PastDue => "past_due" ,
            Self::Unpaid => "unpaid" ,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Trialing)
    }
}

/// 订阅
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: Uuid,
    pub status: SubscriptionStatus,
    /// 当前周期开始时间
    pub current_period_start: DateTime<Utc>,
    /// 当前周期结束时间
    pub current_period_end: DateTime<Utc>,
    /// 是否在当前周期结束时取消
    pub cancel_at_period_end: bool,
    /// 取消时间
    pub canceled_at: Option<DateTime<Utc>>,
    /// 试用结束时间
    pub trial_end: Option<DateTime<Utc>>,
    /// 订阅数量（如席位数）
    pub quantity: i32,
    /// 单价
    pub unit_price: Decimal,
    /// 货币
    pub currency: String,
    /// 下次账单日期
    pub next_billing_date: Option<DateTime<Utc>>,
    /// 元数据
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subscription {
    /// 创建新订阅
    pub fn new(tenant_id: Uuid, plan_id: Uuid, unit_price: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            plan_id,
            status: SubscriptionStatus::Trialing,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(14),
            cancel_at_period_end: false,
            canceled_at: None,
            trial_end: Some(now + chrono::Duration::days(14)),
            quantity: 1,
            unit_price,
            currency: "CNY".to_string(),
            next_billing_date: Some(now + chrono::Duration::days(14)),
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    /// 检查是否处于活跃状态
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.current_period_end
    }

    /// 检查是否处于试用期
    pub fn is_trialing(&self) -> bool {
        self.status == SubscriptionStatus::Trialing
            && self.trial_end.map_or(false, |end| Utc::now() < end)
    }

    /// 获取剩余天数
    pub fn days_remaining(&self) -> i64 {
        let now = Utc::now();
        if now > self.current_period_end {
            0
        } else {
            (self.current_period_end - now).num_days()
        }
    }

    /// 取消订阅
    pub fn cancel(&mut self, at_period_end: bool) {
        if at_period_end {
            self.cancel_at_period_end = true;
        } else {
            self.status = SubscriptionStatus::Cancelled;
            self.canceled_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
    }

    /// 续订
    pub fn renew(&mut self, period_months: i64) {
        self.current_period_start = Utc::now();
        self.current_period_end = self.current_period_start
            + chrono::Duration::days(period_months * 30);
        self.status = SubscriptionStatus::Active;
        self.cancel_at_period_end = false;
        self.next_billing_date = Some(self.current_period_end);
        self.updated_at = Utc::now();
    }

    /// 切换到新计划
    pub fn change_plan(&mut self, new_plan_id: Uuid, new_price: Decimal) {
        self.plan_id = new_plan_id;
        self.unit_price = new_price;
        self.updated_at = Utc::now();
    }
}

/// 订阅变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionChange {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub change_type: SubscriptionChangeType,
    pub from_plan_id: Option<Uuid>,
    pub to_plan_id: Option<Uuid>,
    pub from_price: Option<Decimal>,
    pub to_price: Option<Decimal>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionChangeType {
    Created,
    Renewed,
    Upgraded,
    Downgraded,
    Cancelled,
    Expired,
}

impl SubscriptionChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created" ,
            Self::Renewed => "renewed" ,
            Self::Upgraded => "upgraded" ,
            Self::Downgraded => "downgraded" ,
            Self::Cancelled => "cancelled" ,
            Self::Expired => "expired" ,
        }
    }
}
