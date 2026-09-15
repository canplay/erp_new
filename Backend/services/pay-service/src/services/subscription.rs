// 订阅服务
// Subscription service

use sqlx::PgPool;
use chrono::{Utc, Duration};

use common::AppError;
use common::AppResult;

/// 订阅计划定义
#[derive(Debug, Clone)]
pub struct PlanDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub currency: String,
    pub interval: String,    // "month", "year"
    pub interval_count: i32, // 1 = monthly, 12 = yearly
    pub trial_days: i32,
    pub active: bool,
}

impl PlanDefinition {
    /// 获取默认计划列表
    pub fn default_plans() -> Vec<Self> {
        vec![
            PlanDefinition {
                id: "free".to_string(),
                name: "Free".to_string(),
                description: "Free tier for small teams".to_string(),
                price_cents: 0,
                currency: "CNY".to_string(),
                interval: "month".to_string(),
                interval_count: 1,
                trial_days: 0,
                active: true,
            },
            PlanDefinition {
                id: "pro".to_string(),
                name: "Pro".to_string(),
                description: "Professional plan for growing teams".to_string(),
                price_cents: 9900, // 99 CNY/month
                currency: "CNY".to_string(),
                interval: "month".to_string(),
                interval_count: 1,
                trial_days: 14,
                active: true,
            },
            PlanDefinition {
                id: "enterprise".to_string(),
                name: "Enterprise".to_string(),
                description: "Enterprise plan with full features".to_string(),
                price_cents: 49900, // 499 CNY/month
                currency: "CNY".to_string(),
                interval: "month".to_string(),
                interval_count: 1,
                trial_days: 30,
                active: true,
            },
        ]
    }

    /// 根据ID获取计划
    pub fn get_plan(plan_id: &str) -> Option<Self> {
        Self::default_plans().into_iter().find(|p| p.id == plan_id)
    }
}

/// 订阅服务
pub struct SubscriptionService {
    pub pool: PgPool,
}

/// 订阅创建参数
#[derive(Debug)]
pub struct CreateSubscriptionParams {
    pub tenant_id: i64,
    pub plan_id: String,
    pub payment_method_id: Option<String>,
    pub trial_days: i32,
}

/// 订阅查询结果
#[derive(Debug, Clone)]
pub struct SubscriptionResult {
    pub id: i64,
    pub tenant_id: i64,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub cancel_at_period_end: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl SubscriptionService {
    /// 创建订阅服务实例
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建订阅
    pub async fn create_subscription(&self, params: &CreateSubscriptionParams) -> AppResult<SubscriptionResult> {
        let plan = PlanDefinition::get_plan(&params.plan_id)
            .ok_or_else(|| AppError::PayInternalError(format!("Unknown plan: {}", params.plan_id)))?;

        let now = Utc::now();
        let trial_days = if params.trial_days > 0 { params.trial_days } else { plan.trial_days };

        let (current_period_start, current_period_end, status) = if trial_days > 0 {
            let trial_end = now + Duration::days(trial_days as i64);
            let period_end = match plan.interval.as_str() {
                "year" => trial_end + Duration::days(365),
                _ => trial_end + Duration::days(30),
            };
            (now, period_end, "trialing")
        } else {
            let period_end = match plan.interval.as_str() {
                "year" => now + Duration::days(365 * plan.interval_count as i64),
                _ => now + Duration::days(30 * plan.interval_count as i64),
            };
            (now, period_end, "active")
        };

        // 检查是否已有活跃订阅
        let existing: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT id FROM subscriptions 
            WHERE tenant_id = $1 AND status IN ('active', 'trialing')
            LIMIT 1
            "#
        )
        .bind(params.tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        if existing.is_some() {
            return Err(AppError::PayInternalError("Tenant already has an active subscription".to_string()));
        }

        let row: (i64, i64, String, String, chrono::NaiveDateTime, chrono::NaiveDateTime, bool, chrono::NaiveDateTime, chrono::NaiveDateTime) = sqlx::query_as(
            r#"
            INSERT INTO subscriptions (tenant_id, plan_id, status, current_period_start, current_period_end, cancel_at_period_end, payment_method_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, false, $6, $7, $8)
            RETURNING id, tenant_id, plan_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at, updated_at
            "#
        )
        .bind(params.tenant_id)
        .bind(&params.plan_id)
        .bind(status)
        .bind(current_period_start.naive_utc())
        .bind(current_period_end.naive_utc())
        .bind(&params.payment_method_id)
        .bind(now.naive_utc())
        .bind(now.naive_utc())
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(SubscriptionResult {
            id: row.0,
            tenant_id: row.1,
            plan_id: row.2,
            status: row.3,
            current_period_start: row.4.and_utc().timestamp(),
            current_period_end: row.5.and_utc().timestamp(),
            cancel_at_period_end: row.6,
            created_at: row.7.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: row.8.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    /// 取消订阅
    pub async fn cancel_subscription(&self, subscription_id: i64, immediate: bool) -> AppResult<SubscriptionResult> {
        let row: (i64, i64, String, String, chrono::NaiveDateTime, chrono::NaiveDateTime, bool, chrono::NaiveDateTime, chrono::NaiveDateTime) = sqlx::query_as(
            r#"SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at, updated_at FROM subscriptions WHERE id = $1"#
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::PayInternalError("Subscription not found".to_string()))?;

        let now = Utc::now().naive_utc();

        if immediate {
            sqlx::query(
                r#"UPDATE subscriptions SET status = 'canceled', updated_at = $1 WHERE id = $2"#
            )
            .bind(now)
            .bind(subscription_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

            Ok(SubscriptionResult {
                id: row.0,
                tenant_id: row.1,
                plan_id: row.2,
                status: "canceled".to_string(),
                current_period_start: row.4.and_utc().timestamp(),
                current_period_end: row.5.and_utc().timestamp(),
                cancel_at_period_end: true,
                created_at: row.7.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        } else {
            // Cancel at period end
            sqlx::query(
                r#"UPDATE subscriptions SET cancel_at_period_end = true, updated_at = $1 WHERE id = $2"#
            )
            .bind(now)
            .bind(subscription_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

            Ok(SubscriptionResult {
                id: row.0,
                tenant_id: row.1,
                plan_id: row.2,
                status: row.3,
                current_period_start: row.4.and_utc().timestamp(),
                current_period_end: row.5.and_utc().timestamp(),
                cancel_at_period_end: true,
                created_at: row.7.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        }
    }

    /// 获取订阅详情
    pub async fn get_subscription(&self, subscription_id: i64) -> AppResult<SubscriptionResult> {
        let row: (i64, i64, String, String, chrono::NaiveDateTime, chrono::NaiveDateTime, bool, chrono::NaiveDateTime, chrono::NaiveDateTime) = sqlx::query_as(
            r#"SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at, updated_at FROM subscriptions WHERE id = $1"#
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::PayInternalError("Subscription not found".to_string()))?;

        Ok(SubscriptionResult {
            id: row.0,
            tenant_id: row.1,
            plan_id: row.2,
            status: row.3,
            current_period_start: row.4.and_utc().timestamp(),
            current_period_end: row.5.and_utc().timestamp(),
            cancel_at_period_end: row.6,
            created_at: row.7.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: row.8.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    /// 列出订阅
    pub async fn list_subscriptions(&self, tenant_id: i64, status: Option<&str>, page: i32, page_size: i32) -> AppResult<Vec<SubscriptionResult>> {
        let limit = page_size.max(1).min(100);
        let offset = (page.max(0)) * limit;

        let rows: Vec<(i64, i64, String, String, chrono::NaiveDateTime, chrono::NaiveDateTime, bool, chrono::NaiveDateTime, chrono::NaiveDateTime)> = if let Some(status_filter) = status {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at, updated_at 
                FROM subscriptions 
                WHERE tenant_id = $1 AND status = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#
            )
            .bind(tenant_id)
            .bind(status_filter)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end, cancel_at_period_end, created_at, updated_at 
                FROM subscriptions 
                WHERE tenant_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(tenant_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?
        };

        Ok(rows.into_iter().map(|row| SubscriptionResult {
            id: row.0,
            tenant_id: row.1,
            plan_id: row.2,
            status: row.3,
            current_period_start: row.4.and_utc().timestamp(),
            current_period_end: row.5.and_utc().timestamp(),
            cancel_at_period_end: row.6,
            created_at: row.7.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: row.8.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect())
    }

    /// 检查租户是否有有效订阅
    pub async fn has_active_subscription(&self, tenant_id: i64) -> AppResult<bool> {
        let now = Utc::now().naive_utc();

        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM subscriptions 
            WHERE tenant_id = $1 
              AND status IN ('active', 'trialing')
              AND current_period_end > $2
            "#
        )
        .bind(tenant_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(count.0 > 0)
    }
}

impl Default for SubscriptionService {
    fn default() -> Self {
        let pool = sqlx::PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
            .unwrap_or_else(|e| {
                tracing::error!("数据库连接失败: {e}");
                sqlx::PgPool::connect_lazy("postgres://localhost:5432/fallback").unwrap_or_else(|_| {
                    panic!("无法建立数据库连接: {e}")
                })
            });
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_plans() {
        let plans = PlanDefinition::default_plans();
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].id, "free");
        assert_eq!(plans[1].id, "pro");
        assert!(plans[1].trial_days > 0);
        assert_eq!(plans[2].id, "enterprise");
    }

    #[test]
    fn test_get_plan() {
        let plan = PlanDefinition::get_plan("pro");
        assert!(plan.is_some());
        assert_eq!(plan.unwrap().name, "Pro");

        let unknown = PlanDefinition::get_plan("nonexistent");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_plan_prices() {
        let plans = PlanDefinition::default_plans();
        assert_eq!(plans[0].price_cents, 0); // free
        assert!(plans[1].price_cents > 0);   // pro
        assert!(plans[2].price_cents > plans[1].price_cents); // enterprise
    }
}
