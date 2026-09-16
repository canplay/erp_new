//! 计费服务
//!
//! 根据停车时长和计费规则计算停车费用。

use common::AppError;
use common::AppResult;
use crate::models::{BillingRequest, BillingResult, BillingRule};

/// 计费服务
pub struct BillingService;

impl BillingService {
    pub async fn calculate(&self, req: &BillingRequest) -> AppResult<BillingResult> {
        let entry = chrono::DateTime::parse_from_rfc3339(&req.entry_time)
            .map_err(|e| AppError::BillingCalculationError(format!("入场时间格式错误: {e}")))?;
        let exit = chrono::DateTime::parse_from_rfc3339(&req.exit_time)
            .map_err(|e| AppError::BillingCalculationError(format!("出场时间格式错误: {e}")))?;

        if exit <= entry {
            return Err(AppError::BillingCalculationError("出场时间必须晚于入场时间".to_string()));
        }

        let duration_minutes = (exit - entry).num_minutes();
        let rule = self.get_rule(&req.park_code);
        let total_amount = self.apply_rule(&rule, duration_minutes);

        Ok(BillingResult {
            plate_no: req.plate_no.clone(),
            park_code: req.park_code.clone(),
            entry_time: req.entry_time.clone(),
            exit_time: req.exit_time.clone(),
            duration_minutes,
            total_amount,
            rule: format!("首{}分钟免费，首小时{}元，之后{}元/小时",
                rule.free_minutes,
                rule.first_hour_fee / 100,
                rule.hourly_fee / 100),
        })
    }

    fn get_rule(&self, park_code: &str) -> BillingRule {
        BillingRule {
            park_code: park_code.to_string(),
            free_minutes: 15,
            first_hour_fee: 500,
            hourly_fee: 200,
            daily_cap: 3000,
            night_fee: 500,
            night_start: "22:00".to_string(),
            night_end: "07:00".to_string(),
        }
    }

    fn apply_rule(&self, rule: &BillingRule, duration_minutes: i64) -> i64 {
        if duration_minutes <= i64::from(rule.free_minutes) {
            return 0;
        }

        let billable = duration_minutes - i64::from(rule.free_minutes);
        let hours = (billable + 59) / 60;

        let total = if hours <= 1 {
            rule.first_hour_fee
        } else {
            rule.first_hour_fee + (hours - 1) * rule.hourly_fee
        };

        total.min(rule.daily_cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BillingRule;

    #[test]
    fn test_apply_rule_free_period() {
        let rule = BillingRule {
            park_code: "test".to_string(),
            free_minutes: 15,
            first_hour_fee: 500,
            hourly_fee: 200,
            daily_cap: 3000,
            night_fee: 500,
            night_start: "22:00".to_string(),
            night_end: "07:00".to_string(),
        };
        let service = BillingService;
        assert_eq!(service.apply_rule(&rule, 10), 0);
        assert_eq!(service.apply_rule(&rule, 15), 0);
    }

    #[test]
    fn test_apply_rule_first_hour() {
        let rule = BillingRule {
            park_code: "test".to_string(),
            free_minutes: 15,
            first_hour_fee: 500,
            hourly_fee: 200,
            daily_cap: 3000,
            night_fee: 500,
            night_start: "22:00".to_string(),
            night_end: "07:00".to_string(),
        };
        let service = BillingService;
        // After free period, up to 1 billable hour = first_hour_fee
        assert_eq!(service.apply_rule(&rule, 16), 500);  // 1 min billable
        assert_eq!(service.apply_rule(&rule, 30), 500);  // 15 min billable
        assert_eq!(service.apply_rule(&rule, 60), 500);  // 45 min billable
    }

    #[test]
    fn test_apply_rule_multi_hour() {
        let rule = BillingRule {
            park_code: "test".to_string(),
            free_minutes: 15,
            first_hour_fee: 500,
            hourly_fee: 200,
            daily_cap: 3000,
            night_fee: 500,
            night_start: "22:00".to_string(),
            night_end: "07:00".to_string(),
        };
        let service = BillingService;
        // 75 min = 15 free + 60 billable = 1 hour = first_hour_fee
        assert_eq!(service.apply_rule(&rule, 75), 500);
        // 90 min = 15 free + 75 billable = ceil(75/60)=2 hours
        // first hour: 500, second hour: 200 = 700
        assert_eq!(service.apply_rule(&rule, 90), 700);
        // 120 min = 15 free + 105 billable = ceil(105/60)=2 hours
        assert_eq!(service.apply_rule(&rule, 120), 700);
    }

    #[test]
    fn test_apply_rule_daily_cap() {
        let rule = BillingRule {
            park_code: "test".to_string(),
            free_minutes: 15,
            first_hour_fee: 500,
            hourly_fee: 200,
            daily_cap: 3000,
            night_fee: 500,
            night_start: "22:00".to_string(),
            night_end: "07:00".to_string(),
        };
        let service = BillingService;
        // Very long duration should be capped at daily_cap
        assert_eq!(service.apply_rule(&rule, 24 * 60), 3000);  // 24 hours
        assert_eq!(service.apply_rule(&rule, 48 * 60), 3000);  // 48 hours
    }

    #[test]
    fn test_apply_rule_zero_duration() {
        let rule = BillingRule {
            park_code: "test".to_string(),
            free_minutes: 15,
            first_hour_fee: 500,
            hourly_fee: 200,
            daily_cap: 3000,
            night_fee: 500,
            night_start: "22:00".to_string(),
            night_end: "07:00".to_string(),
        };
        let service = BillingService;
        assert_eq!(service.apply_rule(&rule, 0), 0);
    }

    #[tokio::test]
    async fn test_calculate_valid_times() {
        let service = BillingService;
        let req = BillingRequest {
            plate_no: "粤A12345".to_string(),
            park_code: "P001".to_string(),
            entry_time: "2026-06-18T08:00:00Z".to_string(),
            exit_time: "2026-06-18T10:30:00Z".to_string(),
        };

        let result = service.calculate(&req).await;
        assert!(result.is_ok());
        let result = result.expect("billing result should exist");
        assert_eq!(result.plate_no, "粤A12345");
        assert_eq!(result.park_code, "P001");
        assert_eq!(result.duration_minutes, 150); // 2.5 hours
        // 15 free + 135 billable = ceil(135/60)=3 hours
        // first_hour: 500 + (3-1)*200 = 900
        assert_eq!(result.total_amount, 900);
    }

    #[tokio::test]
    async fn test_calculate_exit_before_entry_error() {
        let service = BillingService;
        let req = BillingRequest {
            plate_no: "粤A12345".to_string(),
            park_code: "P001".to_string(),
            entry_time: "2026-06-18T10:00:00Z".to_string(),
            exit_time: "2026-06-18T08:00:00Z".to_string(),
        };

        let result = service.calculate(&req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("出场时间必须晚于入场时间"));
    }

    #[tokio::test]
    async fn test_calculate_invalid_entry_time() {
        let service = BillingService;
        let req = BillingRequest {
            plate_no: "粤A12345".to_string(),
            park_code: "P001".to_string(),
            entry_time: "invalid-date".to_string(),
            exit_time: "2026-06-18T10:00:00Z".to_string(),
        };

        let result = service.calculate(&req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("入场时间格式错误"));
    }

    #[tokio::test]
    async fn test_calculate_invalid_exit_time() {
        let service = BillingService;
        let req = BillingRequest {
            plate_no: "粤A12345".to_string(),
            park_code: "P001".to_string(),
            entry_time: "2026-06-18T08:00:00Z".to_string(),
            exit_time: "invalid-date".to_string(),
        };

        let result = service.calculate(&req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("出场时间格式错误"));
    }

    #[test]
    fn test_get_rule_returns_default() {
        let service = BillingService;
        let rule = service.get_rule("P001");
        assert_eq!(rule.park_code, "P001");
        assert_eq!(rule.free_minutes, 15);
        assert_eq!(rule.first_hour_fee, 500);
        assert_eq!(rule.hourly_fee, 200);
        assert_eq!(rule.daily_cap, 3000);
    }
}
