//! 计费服务单元测试
//!
//! 覆盖状态机流转、用量汇总、配额检查等核心业务逻辑。

#[cfg(test)]
mod subscription_state_machine_tests {
    use billing_core::{Subscription, SubscriptionStatus};
    use billing_service::SubscriptionStateMachine;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn create_subscription(status: SubscriptionStatus) -> Subscription {
        let now = chrono::Utc::now();
        Subscription {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            status,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(30),
            cancel_at_period_end: false,
            canceled_at: None,
            trial_end: Some(now + chrono::Duration::days(14)),
            quantity: 1,
            unit_price: Decimal::from(99),
            currency: "CNY".to_string(),
            next_billing_date: Some(now + chrono::Duration::days(14)),
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    // ==================== 状态机流转测试 ====================

    #[test]
    fn test_activate_trialing_to_active() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        assert_eq!(sub.status, SubscriptionStatus::Trialing);

        let result = SubscriptionStateMachine::activate(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }

    #[test]
    fn test_activate_non_trialing_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Active);
        let result = SubscriptionStateMachine::activate(&mut sub);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Only trialing subscriptions can be activated"
        );
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }

    #[test]
    fn test_activate_past_due_fails() {
        let mut sub = create_subscription(SubscriptionStatus::PastDue);
        let result = SubscriptionStateMachine::activate(&mut sub);
        assert!(result.is_err());
        assert_eq!(sub.status, SubscriptionStatus::PastDue);
    }

    #[test]
    fn test_activate_cancelled_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Cancelled);
        let result = SubscriptionStateMachine::activate(&mut sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_activate_expired_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Expired);
        let result = SubscriptionStateMachine::activate(&mut sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_mark_past_due_from_active() {
        let mut sub = create_subscription(SubscriptionStatus::Active);
        let result = SubscriptionStateMachine::mark_past_due(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::PastDue);
    }

    #[test]
    fn test_mark_past_due_from_trialing() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        let result = SubscriptionStateMachine::mark_past_due(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::PastDue);
    }

    #[test]
    fn test_mark_past_due_from_past_due_fails() {
        let mut sub = create_subscription(SubscriptionStatus::PastDue);
        let result = SubscriptionStateMachine::mark_past_due(&mut sub);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Only active or trialing subscriptions can be marked past due"
        );
    }

    #[test]
    fn test_mark_past_due_from_cancelled_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Cancelled);
        let result = SubscriptionStateMachine::mark_past_due(&mut sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_reactivate_from_past_due() {
        let mut sub = create_subscription(SubscriptionStatus::PastDue);
        let result = SubscriptionStateMachine::reactivate(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }

    #[test]
    fn test_reactivate_from_active_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Active);
        let result = SubscriptionStateMachine::reactivate(&mut sub);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Only past due subscriptions can be reactivated"
        );
    }

    #[test]
    fn test_reactivate_from_trialing_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        let result = SubscriptionStateMachine::reactivate(&mut sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_reactivate_from_cancelled_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Cancelled);
        let result = SubscriptionStateMachine::reactivate(&mut sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_from_active() {
        let mut sub = create_subscription(SubscriptionStatus::Active);
        let original_updated_at = sub.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1));
        let result = SubscriptionStateMachine::cancel(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
        assert!(sub.canceled_at.is_some());
        assert!(sub.updated_at > original_updated_at);
    }

    #[test]
    fn test_cancel_from_trialing() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        let result = SubscriptionStateMachine::cancel(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
    }

    #[test]
    fn test_cancel_from_past_due() {
        let mut sub = create_subscription(SubscriptionStatus::PastDue);
        let result = SubscriptionStateMachine::cancel(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
    }

    #[test]
    fn test_cancel_already_cancelled_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Cancelled);
        let result = SubscriptionStateMachine::cancel(&mut sub);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Subscription is already cancelled");
    }

    #[test]
    fn test_cancel_expired_allowed() {
        let mut sub = create_subscription(SubscriptionStatus::Expired);
        let result = SubscriptionStateMachine::cancel(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
        assert!(sub.canceled_at.is_some());
    }

    #[test]
    fn test_expire_from_active() {
        let mut sub = create_subscription(SubscriptionStatus::Active);
        let result = SubscriptionStateMachine::expire(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Expired);
    }

    #[test]
    fn test_expire_from_trialing() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        let result = SubscriptionStateMachine::expire(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Expired);
    }

    #[test]
    fn test_expire_from_past_due() {
        let mut sub = create_subscription(SubscriptionStatus::PastDue);
        let result = SubscriptionStateMachine::expire(&mut sub);
        assert!(result.is_ok());
        assert_eq!(sub.status, SubscriptionStatus::Expired);
    }

    #[test]
    fn test_expire_from_expired_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Expired);
        let result = SubscriptionStateMachine::expire(&mut sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_expire_from_cancelled_fails() {
        let mut sub = create_subscription(SubscriptionStatus::Cancelled);
        let result = SubscriptionStateMachine::expire(&mut sub);
        assert!(result.is_err());
    }

    // ==================== 完整状态机流转测试 ====================

    #[test]
    fn test_full_lifecycle_trialing_active() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        SubscriptionStateMachine::activate(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        SubscriptionStateMachine::cancel(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
    }

    #[test]
    fn test_full_lifecycle_trialing_past_due_active() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        SubscriptionStateMachine::mark_past_due(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::PastDue);
        SubscriptionStateMachine::reactivate(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        SubscriptionStateMachine::expire(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Expired);
    }

    #[test]
    fn test_full_lifecycle_trialing_past_due_expired() {
        let mut sub = create_subscription(SubscriptionStatus::Trialing);
        SubscriptionStateMachine::mark_past_due(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::PastDue);
        SubscriptionStateMachine::expire(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Expired);
    }

    #[test]
    fn test_full_lifecycle_active_past_due_cancel() {
        let mut sub = create_subscription(SubscriptionStatus::Active);
        SubscriptionStateMachine::mark_past_due(&mut sub).unwrap();
        SubscriptionStateMachine::cancel(&mut sub).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
    }
}

#[cfg(test)]
mod usage_summary_tests {
    use billing_service::UsageSummary;

    #[test]
    fn test_usage_summary_creation() {
        let now = chrono::Utc::now();
        let summary = UsageSummary {
            metric: "api_calls".to_string(),
            total_quantity: 1500.0,
            period_start: now - chrono::Duration::days(7),
            period_end: now,
        };
        assert_eq!(summary.metric, "api_calls");
        assert_eq!(summary.total_quantity, 1500.0);
    }

    #[test]
    fn test_usage_summary_total_quantity() {
        let summaries = vec![
            UsageSummary {
                metric: "api_calls".to_string(),
                total_quantity: 100.0,
                period_start: chrono::Utc::now(),
                period_end: chrono::Utc::now(),
            },
            UsageSummary {
                metric: "api_calls".to_string(),
                total_quantity: 200.0,
                period_start: chrono::Utc::now(),
                period_end: chrono::Utc::now(),
            },
            UsageSummary {
                metric: "api_calls".to_string(),
                total_quantity: 300.0,
                period_start: chrono::Utc::now(),
                period_end: chrono::Utc::now(),
            },
        ];
        let total: f64 = summaries.iter().map(|s| s.total_quantity).sum();
        assert_eq!(total, 600.0);
    }

    #[test]
    fn test_usage_summary_zero_usage() {
        let summary = UsageSummary {
            metric: "storage_gb".to_string(),
            total_quantity: 0.0,
            period_start: chrono::Utc::now(),
            period_end: chrono::Utc::now(),
        };
        assert_eq!(summary.total_quantity, 0.0);
    }

    #[test]
    fn test_usage_summary_fractional_usage() {
        let summary = UsageSummary {
            metric: "storage_gb".to_string(),
            total_quantity: 2.5,
            period_start: chrono::Utc::now(),
            period_end: chrono::Utc::now(),
        };
        assert_eq!(summary.total_quantity, 2.5);
    }

    #[test]
    fn test_usage_summary_large_usage() {
        let summary = UsageSummary {
            metric: "messages".to_string(),
            total_quantity: 1_000_000.0,
            period_start: chrono::Utc::now(),
            period_end: chrono::Utc::now(),
        };
        assert_eq!(summary.total_quantity, 1_000_000.0);
    }
}

#[cfg(test)]
mod quota_check_tests {
    use billing_service::QuotaCheckResult;

    #[test]
    fn test_quota_not_exceeded() {
        let result = QuotaCheckResult::new("api_calls", 500.0, 1000);
        assert!(!result.exceeded);
        assert_eq!(result.overage, 0.0);
        assert_eq!(result.current_usage, 500.0);
        assert_eq!(result.quota_limit, 1000);
    }

    #[test]
    fn test_quota_exactly_at_limit() {
        let result = QuotaCheckResult::new("api_calls", 1000.0, 1000);
        assert!(!result.exceeded);
        assert_eq!(result.overage, 0.0);
    }

    #[test]
    fn test_quota_exceeded() {
        let result = QuotaCheckResult::new("api_calls", 1500.0, 1000);
        assert!(result.exceeded);
        assert_eq!(result.overage, 500.0);
    }

    #[test]
    fn test_quota_exceeded_fractional() {
        let result = QuotaCheckResult::new("storage_gb", 2.5, 2);
        assert!(result.exceeded);
        assert_eq!(result.overage, 0.5);
    }

    #[test]
    fn test_quota_zero_usage() {
        let result = QuotaCheckResult::new("api_calls", 0.0, 1000);
        assert!(!result.exceeded);
        assert_eq!(result.overage, 0.0);
    }

    #[test]
    fn test_quota_zero_limit() {
        let result = QuotaCheckResult::new("api_calls", 1.0, 0);
        assert!(result.exceeded);
        assert_eq!(result.overage, 1.0);
    }

    #[test]
    fn test_quota_multiple_metrics() {
        let metrics = vec![
            ("api_calls", 500.0, 1000),
            ("storage_gb", 5.0, 10),
            ("messages", 2000.0, 1000),
        ];
        let results: Vec<QuotaCheckResult> = metrics
            .iter()
            .map(|(name, usage, limit)| QuotaCheckResult::new(*name, *usage, *limit))
            .collect();

        assert_eq!(results.len(), 3);
        assert!(!results[0].exceeded);
        assert!(!results[1].exceeded);
        assert!(results[2].exceeded);
        assert_eq!(results[2].overage, 1000.0);
    }

    #[test]
    fn test_quota_display_metric() {
        let result = QuotaCheckResult::new("active_users", 50.0, 100);
        assert_eq!(result.metric, "active_users");
    }
}

#[cfg(test)]
mod subscription_helper_tests {
    use billing_core::{Subscription, SubscriptionStatus};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[test]
    fn test_subscription_is_active_when_active() {
        let now = chrono::Utc::now();
        let sub = Subscription {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            status: SubscriptionStatus::Active,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(30),
            cancel_at_period_end: false,
            canceled_at: None,
            trial_end: None,
            quantity: 1,
            unit_price: Decimal::from(99),
            currency: "CNY".to_string(),
            next_billing_date: Some(now + chrono::Duration::days(30)),
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        };
        assert!(sub.is_active());
    }

    #[test]
    fn test_subscription_is_active_when_trialing() {
        let now = chrono::Utc::now();
        let sub = Subscription {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            status: SubscriptionStatus::Trialing,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(30),
            cancel_at_period_end: false,
            canceled_at: None,
            trial_end: Some(now + chrono::Duration::days(14)),
            quantity: 1,
            unit_price: Decimal::from(99),
            currency: "CNY".to_string(),
            next_billing_date: Some(now + chrono::Duration::days(14)),
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        };
        assert!(sub.is_active());
    }

    #[test]
    fn test_subscription_is_not_active_when_cancelled() {
        let now = chrono::Utc::now();
        let sub = Subscription {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            status: SubscriptionStatus::Cancelled,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(30),
            cancel_at_period_end: false,
            canceled_at: Some(now),
            trial_end: None,
            quantity: 1,
            unit_price: Decimal::from(99),
            currency: "CNY".to_string(),
            next_billing_date: None,
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        };
        assert!(!sub.is_active());
    }

    #[test]
    fn test_subscription_days_remaining() {
        let now = chrono::Utc::now();
        let sub = Subscription {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            status: SubscriptionStatus::Active,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(15),
            cancel_at_period_end: false,
            canceled_at: None,
            trial_end: None,
            quantity: 1,
            unit_price: Decimal::from(99),
            currency: "CNY".to_string(),
            next_billing_date: Some(now + chrono::Duration::days(15)),
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        };
        let remaining = sub.days_remaining();
        assert!(remaining >= 14 && remaining <= 15);
    }

    #[test]
    fn test_subscription_days_remaining_expired() {
        let now = chrono::Utc::now();
        let sub = Subscription {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            status: SubscriptionStatus::Expired,
            current_period_start: now - chrono::Duration::days(30),
            current_period_end: now - chrono::Duration::days(1),
            cancel_at_period_end: false,
            canceled_at: None,
            trial_end: None,
            quantity: 1,
            unit_price: Decimal::from(99),
            currency: "CNY".to_string(),
            next_billing_date: None,
            metadata: serde_json::Value::Null,
            created_at: now - chrono::Duration::days(60),
            updated_at: now - chrono::Duration::days(30),
        };
        assert_eq!(sub.days_remaining(), 0);
    }

    #[test]
    fn test_subscription_new_creates_trialing() {
        let sub = Subscription::new(Uuid::new_v4(), Uuid::new_v4(), Decimal::from(99));
        assert_eq!(sub.status, SubscriptionStatus::Trialing);
        assert_eq!(sub.quantity, 1);
        assert_eq!(sub.unit_price, Decimal::from(99));
    }

    #[test]
    fn test_subscription_cancel_at_period_end() {
        let mut sub = Subscription::new(Uuid::new_v4(), Uuid::new_v4(), Decimal::from(99));
        sub.cancel(true);
        assert!(sub.cancel_at_period_end);
        assert_eq!(sub.status, SubscriptionStatus::Trialing); // status unchanged
    }

    #[test]
    fn test_subscription_cancel_immediately() {
        let mut sub = Subscription::new(Uuid::new_v4(), Uuid::new_v4(), Decimal::from(99));
        sub.cancel(false);
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
        assert!(sub.canceled_at.is_some());
    }

    #[test]
    fn test_subscription_renew() {
        let mut sub = Subscription::new(Uuid::new_v4(), Uuid::new_v4(), Decimal::from(99));
        sub.renew(1);
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert!(!sub.cancel_at_period_end);
        assert!(sub.next_billing_date.is_some());
    }

    #[test]
    fn test_subscription_change_plan() {
        let mut sub = Subscription::new(Uuid::new_v4(), Uuid::new_v4(), Decimal::from(99));
        let new_plan_id = Uuid::new_v4();
        let new_price = Decimal::from(199);
        sub.change_plan(new_plan_id, new_price);
        assert_eq!(sub.plan_id, new_plan_id);
        assert_eq!(sub.unit_price, new_price);
    }
}

#[cfg(test)]
mod billing_plan_tests {
    use billing_core::{BillingPlan, BillingPlanType, PlanStatus};
    use rust_decimal::Decimal;

    #[test]
    fn test_billing_plan_new() {
        let plan = BillingPlan::new("Test Plan", BillingPlanType::Standard);
        assert_eq!(plan.name, "Test Plan");
        assert_eq!(plan.plan_type, BillingPlanType::Standard);
        assert_eq!(plan.status, PlanStatus::Draft);
        assert_eq!(plan.price_monthly, Decimal::ZERO);
        assert_eq!(plan.price_yearly, Decimal::ZERO);
    }

    #[test]
    fn test_billing_plan_with_prices() {
        let plan = BillingPlan::new("Test Plan", BillingPlanType::Standard)
            .with_monthly_price(Decimal::from(99))
            .with_yearly_price(Decimal::from(999));
        assert_eq!(plan.price_monthly, Decimal::from(99));
        assert_eq!(plan.price_yearly, Decimal::from(999));
    }

    #[test]
    fn test_billing_plan_yearly_discount() {
        let plan = BillingPlan::new("Test Plan", BillingPlanType::Standard)
            .with_monthly_price(Decimal::from(100))
            .with_yearly_price(Decimal::from(1000));
        // 100*12=1200, 1200-1000=200, 200/1200*100=16.67%
        let discount = plan.yearly_discount_percent();
        assert!(discount.is_some());
        assert_eq!(discount.unwrap().round_dp(2), Decimal::try_from(16.67).unwrap());
    }

    #[test]
    fn test_billing_plan_no_discount() {
        let plan = BillingPlan::new("Test Plan", BillingPlanType::Standard)
            .with_monthly_price(Decimal::from(100))
            .with_yearly_price(Decimal::from(1200));
        // 100*12=1200, 1200-1200=0, no discount
        let discount = plan.yearly_discount_percent();
        assert!(discount.is_none());
    }

    #[test]
    fn test_billing_plan_price_for_period() {
        let plan = BillingPlan::new("Test Plan", BillingPlanType::Standard)
            .with_monthly_price(Decimal::from(99))
            .with_yearly_price(Decimal::from(999));
        assert_eq!(plan.price_for_period(false), Decimal::from(99));
        assert_eq!(plan.price_for_period(true), Decimal::from(999));
    }

    #[test]
    fn test_billing_plan_with_quota() {
        let plan =
            BillingPlan::new("Test Plan", BillingPlanType::Standard).with_quota("max_users", 100);
        assert!(plan.quotas.contains_key("max_users"));
        assert_eq!(plan.quotas["max_users"], 100);
    }

    #[test]
    fn test_create_free_plan() {
        let plan = billing_core::create_free_plan();
        assert_eq!(plan.name, "Free");
        assert_eq!(plan.plan_type, BillingPlanType::Free);
        assert_eq!(plan.price_monthly, Decimal::ZERO);
        assert_eq!(plan.price_yearly, Decimal::ZERO);
        assert!(plan.quotas.contains_key("max_users"));
        assert_eq!(plan.quotas["max_users"], 5);
    }

    #[test]
    fn test_create_standard_plan() {
        let plan = billing_core::create_standard_plan();
        assert_eq!(plan.name, "Standard");
        assert_eq!(plan.plan_type, BillingPlanType::Standard);
        assert_eq!(plan.price_monthly, Decimal::from(99));
        assert_eq!(plan.price_yearly, Decimal::from(999));
        assert!(plan.quotas.contains_key("monthly_api_calls"));
        assert_eq!(plan.quotas["monthly_api_calls"], 100000);
    }

    #[test]
    fn test_create_enterprise_plan() {
        let plan = billing_core::create_enterprise_plan();
        assert_eq!(plan.name, "Enterprise");
        assert_eq!(plan.plan_type, BillingPlanType::Enterprise);
        assert_eq!(plan.price_monthly, Decimal::from(499));
        assert_eq!(plan.price_yearly, Decimal::from(4999));
        assert!(plan.quotas.contains_key("max_users"));
        assert_eq!(plan.quotas["max_users"], 500);
    }
}

#[cfg(test)]
mod usage_type_tests {
    use billing_core::UsageType;

    #[test]
    fn test_usage_type_as_str() {
        assert_eq!(UsageType::ApiCalls.as_str(), "api_calls");
        assert_eq!(UsageType::FileUploads.as_str(), "file_uploads");
        assert_eq!(UsageType::StorageGb.as_str(), "storage_gb");
        assert_eq!(UsageType::ActiveUsers.as_str(), "active_users");
        assert_eq!(UsageType::Messages.as_str(), "messages");
        assert_eq!(UsageType::VoiceMinutes.as_str(), "voice_minutes");
        assert_eq!(UsageType::VideoMinutes.as_str(), "video_minutes");
    }

    #[test]
    fn test_usage_type_equality() {
        assert_eq!(UsageType::ApiCalls, UsageType::ApiCalls);
        assert_ne!(UsageType::ApiCalls, UsageType::Messages);
    }

    #[test]
    fn test_usage_type_clone() {
        let ut = UsageType::ApiCalls;
        let cloned = ut;
        assert_eq!(ut, cloned);
    }
}
