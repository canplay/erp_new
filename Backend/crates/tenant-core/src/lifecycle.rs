//! 租户生命周期管理
//!
//! 管理租户从创建到销毁的完整生命周期：
//! 创建 → 活跃 → 暂停 → 过期 → 续期 → 删除

use crate::TenantId;
use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};

use std::fmt;

/// 租户生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LifecycleState {
    /// 新建（待初始化）
    #[default]
    Provisioning,
    /// 活跃（正常使用）
    Active,
    /// 试用中
    Trial,
    /// 暂停（欠费/违规）
    Suspended,
    /// 宽限期（即将过期）
    GracePeriod,
    /// 已过期
    Expired,
    /// 已删除
    Deleted,
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}" , self.as_str())
    }
}

impl LifecycleState {
    /// 获取状态的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Provisioning => "provisioning" ,
            Self::Active => "active" ,
            Self::Trial => "trial" ,
            Self::Suspended => "suspended" ,
            Self::GracePeriod => "grace_period" ,
            Self::Expired => "expired" ,
            Self::Deleted => "deleted" ,
        }
    }

    /// 从字符串解析状态
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "provisioning" => Some(Self::Provisioning),
            "active" => Some(Self::Active),
            "trial" => Some(Self::Trial),
            "suspended" => Some(Self::Suspended),
            "grace_period" => Some(Self::GracePeriod),
            "expired" => Some(Self::Expired),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }

    /// 是否为活跃状态（可正常使用）
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Active | Self::Trial)
    }

    /// 是否可以转换到目标状态
    pub fn can_transition_to(&self, target: &Self) -> bool {
        match (self, target) {
            // 新建 → 活跃/试用/删除
            (Self::Provisioning, Self::Active) => true,
            (Self::Provisioning, Self::Trial) => true,
            (Self::Provisioning, Self::Deleted) => true,
            // 活跃 → 暂停/过期/删除
            (Self::Active, Self::Suspended) => true,
            (Self::Active, Self::GracePeriod) => true,
            (Self::Active, Self::Expired) => true,
            (Self::Active, Self::Deleted) => true,
            // 试用 → 活跃/过期/删除
            (Self::Trial, Self::Active) => true,
            (Self::Trial, Self::Expired) => true,
            (Self::Trial, Self::Deleted) => true,
            // 暂停 → 活跃/删除
            (Self::Suspended, Self::Active) => true,
            (Self::Suspended, Self::Deleted) => true,
            // 宽限期 → 活跃/过期
            (Self::GracePeriod, Self::Active) => true,
            (Self::GracePeriod, Self::Expired) => true,
            // 过期 → 活跃（续期）/删除
            (Self::Expired, Self::Active) => true,
            (Self::Expired, Self::Deleted) => true,
            // 不允许其他转换
            _ => false,
        }
    }
}

/// 租户生命周期管理器
#[derive(Debug, Clone)]
pub struct LifecycleManager {
    /// 试用天数
    pub trial_days: i64,
    /// 宽限期天数
    pub grace_period_days: i64,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self {
            trial_days: 14,
            grace_period_days: 7,
        }
    }
}

impl LifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new(trial_days: i64, grace_period_days: i64) -> Self {
        Self {
            trial_days,
            grace_period_days,
        }
    }

    /// 计算试用到期时间
    pub fn trial_expiry(&self) -> DateTime<Utc> {
        Utc::now() + Duration::days(self.trial_days)
    }

    /// 计算宽限期结束时间
    pub fn grace_period_expiry(&self, expiry: DateTime<Utc>) -> DateTime<Utc> {
        expiry + Duration::days(self.grace_period_days)
    }

    /// 检查是否处于宽限期
    pub fn is_in_grace_period(&self, expiry: DateTime<Utc>) -> bool {
        let now = Utc::now();
        now > expiry && now <= self.grace_period_expiry(expiry)
    }

    /// 检查是否已过期
    pub fn is_expired(&self, expiry: DateTime<Utc>) -> bool {
        Utc::now() > self.grace_period_expiry(expiry)
    }

    /// 获取下一个计费周期结束时间
    pub fn next_billing_cycle(&self, from: DateTime<Utc>, interval_months: u32) -> DateTime<Utc> {
        let mut year = from.year();
        let mut month = from.month() + interval_months;
        
        while month > 12 {
            month -= 12;
            year += 1;
        }
        
        chrono::NaiveDate::from_ymd_opt(year, month, from.day().min(28))
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default())
            .and_hms_opt(from.hour().min(23), from.minute().min(59), from.second().min(59))
            .unwrap_or_else(|| {
                chrono::NaiveDate::from_ymd_opt(year, month, 1)
                    .unwrap_or_default()
                    .and_hms_opt(0, 0, 0)
                    .unwrap_or_default()
            })
            .and_utc()
    }
}

/// 生命周期转换请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleTransition {
    pub tenant_id: TenantId,
    pub from_state: LifecycleState,
    pub to_state: LifecycleState,
    pub reason: Option<String>,
    pub operator_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
}

impl LifecycleTransition {
    /// 创建新的生命周期转换
    pub fn new(
        tenant_id: TenantId,
        from_state: LifecycleState,
        to_state: LifecycleState,
    ) -> Self {
        Self {
            tenant_id,
            from_state,
            to_state,
            reason: None,
            operator_id: None,
            timestamp: Utc::now(),
        }
    }

    /// 设置原因
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// 设置操作人
    pub fn with_operator(mut self, operator_id: i64) -> Self {
        self.operator_id = Some(operator_id);
        self
    }
}

/// 生命周期事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// 租户创建
    TenantCreated {
        tenant_id: TenantId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    /// 租户激活
    TenantActivated {
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
    /// 租户暂停
    TenantSuspended {
        tenant_id: TenantId,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// 租户即将过期
    TenantNearingExpiry {
        tenant_id: TenantId,
        expiry: DateTime<Utc>,
        days_remaining: i64,
        timestamp: DateTime<Utc>,
    },
    /// 租户进入宽限期
    TenantEnteredGracePeriod {
        tenant_id: TenantId,
        expiry: DateTime<Utc>,
        grace_period_end: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    /// 租户过期
    TenantExpired {
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
    /// 租户续期
    TenantRenewed {
        tenant_id: TenantId,
        new_expiry: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    /// 租户删除
    TenantDeleted {
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_state_str() {
        assert_eq!(LifecycleState::Active.as_str(), "active" );
        assert_eq!(LifecycleState::from_str("active" ), Some(LifecycleState::Active));
        assert_eq!(LifecycleState::from_str("invalid" ), None);
    }

    #[test]
    fn test_state_transitions() {
        assert!(LifecycleState::Provisioning.can_transition_to(&LifecycleState::Active));
        assert!(LifecycleState::Active.can_transition_to(&LifecycleState::Suspended));
        assert!(LifecycleState::Suspended.can_transition_to(&LifecycleState::Active));
        assert!(!LifecycleState::Deleted.can_transition_to(&LifecycleState::Active));
    }

    #[test]
    fn test_operational_states() {
        assert!(LifecycleState::Active.is_operational());
        assert!(LifecycleState::Trial.is_operational());
        assert!(!LifecycleState::Suspended.is_operational());
        assert!(!LifecycleState::Expired.is_operational());
    }

    #[test]
    fn test_lifecycle_manager() {
        let manager = LifecycleManager::new(14, 7);
        let trial_expiry = manager.trial_expiry();
        assert!(trial_expiry > Utc::now());
        
        let grace_end = manager.grace_period_expiry(trial_expiry);
        assert!(grace_end > trial_expiry);
    }
}
