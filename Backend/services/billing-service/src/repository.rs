//! 计费服务仓储层

use sqlx::PgPool;
use uuid::Uuid;

use common::AppError;

/// 计费仓储
#[derive(Clone)]
pub struct BillingRepository {
    pool: PgPool,
}

impl BillingRepository {
    /// 创建新的仓储
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============ 计划管理 ============

    /// 创建计划
    pub async fn create_plan(
        &self,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        price_monthly: f64,
        price_yearly: f64,
        currency: &str,
        features: &serde_json::Value,
        quotas: &serde_json::Value,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO plans (id, name, description, plan_type, status, price_monthly, price_yearly, currency, features, quotas, is_public)
            VALUES ($1, $2, $3, $4, 'active', $5, $6, $7, $8, $9, true)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(plan_type)
        .bind(price_monthly)
        .bind(price_yearly)
        .bind(currency)
        .bind(features)
        .bind(quotas)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// 获取计划
    pub async fn get_plan(&self, id: Uuid) -> Result<Option<PlanRow>, sqlx::Error> {
        sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT id, name, description, plan_type, status, price_monthly, price_yearly, 
                   currency, features, quotas, sort_order, is_public, created_at, updated_at
            FROM plans WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 列出计划
    pub async fn list_plans(&self, include_archived: bool) -> Result<Vec<PlanRow>, sqlx::Error> {
        if include_archived {
            sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, name, description, plan_type, status, price_monthly, price_yearly, 
                       currency, features, quotas, sort_order, is_public, created_at, updated_at
                FROM plans ORDER BY sort_order
                "#,
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, name, description, plan_type, status, price_monthly, price_yearly, 
                       currency, features, quotas, sort_order, is_public, created_at, updated_at
                FROM plans WHERE status = 'active' AND is_public = true ORDER BY sort_order
                "#,
            )
            .fetch_all(&self.pool)
            .await
        }
    }

    // ============ 订阅管理 ============

    /// 创建订阅
    pub async fn create_subscription(
        &self,
        tenant_id: Uuid,
        plan_id: Uuid,
        status: &str,
        current_period_start: chrono::DateTime<chrono::Utc>,
        current_period_end: chrono::DateTime<chrono::Utc>,
        trial_end: Option<chrono::DateTime<chrono::Utc>>,
        unit_price: f64,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO subscriptions (id, tenant_id, plan_id, status, current_period_start, current_period_end, trial_end, unit_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(plan_id)
        .bind(status)
        .bind(current_period_start)
        .bind(current_period_end)
        .bind(trial_end)
        .bind(unit_price)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// 获取订阅
    pub async fn get_subscription(&self, id: Uuid) -> Result<Option<SubscriptionRow>, sqlx::Error> {
        sqlx::query_as::<_, SubscriptionRow>(
            r#"
            SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end,
                   cancel_at_period_end, canceled_at, trial_end, quantity, unit_price, currency,
                   next_billing_date, metadata, created_at, updated_at
            FROM subscriptions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 获取租户的订阅
    pub async fn get_tenant_subscription(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<SubscriptionRow>, sqlx::Error> {
        sqlx::query_as::<_, SubscriptionRow>(
            r#"
            SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end,
                   cancel_at_period_end, canceled_at, trial_end, quantity, unit_price, currency,
                   next_billing_date, metadata, created_at, updated_at
            FROM subscriptions WHERE tenant_id = $1 AND status IN ('active', 'trialing')
            ORDER BY created_at DESC LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
    }

    // ============ 发票管理 ============

    /// 创建发票
    pub async fn create_invoice(
        &self,
        invoice_number: &str,
        subscription_id: Option<Uuid>,
        tenant_id: Uuid,
        subtotal: f64,
        tax_amount: f64,
        total: f64,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
        due_at: chrono::DateTime<chrono::Utc>,
        line_items: &serde_json::Value,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO invoices (id, invoice_number, subscription_id, tenant_id, status, subtotal, tax_amount, total, period_start, period_end, due_at, line_items)
            VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(id)
        .bind(invoice_number)
        .bind(subscription_id)
        .bind(tenant_id)
        .bind(subtotal)
        .bind(tax_amount)
        .bind(total)
        .bind(period_start)
        .bind(period_end)
        .bind(due_at)
        .bind(line_items)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// 获取发票
    pub async fn get_invoice(&self, id: Uuid) -> Result<Option<InvoiceRow>, sqlx::Error> {
        sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT id, invoice_number, subscription_id, tenant_id, status, subtotal, tax_amount, total,
                   currency, period_start, period_end, issued_at, due_at, paid_at, line_items, notes, created_at, updated_at
            FROM invoices WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 列出发票
    pub async fn list_invoices(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<InvoiceRow>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let status_filter = status.map(|s| format!("AND status = '{}'" , s)).unwrap_or_default();

        let invoices = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT id, invoice_number, subscription_id, tenant_id, status, subtotal, tax_amount, total,
                   currency, period_start, period_end, issued_at, due_at, paid_at, line_items, notes, created_at, updated_at
            FROM invoices WHERE tenant_id = $1
            ORDER BY created_at DESC LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count FROM invoices WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((invoices, total))
    }

    // ============ 用量管理 ============

    /// 记录用量
    pub async fn record_usage(
        &self,
        tenant_id: Uuid,
        metric: &str,
        quantity: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO usage_records (tenant_id, metric, quantity)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(tenant_id)
        .bind(metric)
        .bind(quantity)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取用量统计
    pub async fn get_usage(
        &self,
        tenant_id: Uuid,
        metric: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<f64, sqlx::Error> {
        let total: f64 = sqlx::query_scalar::<_, f64>(
            r#"
            SELECT COALESCE(SUM(quantity), 0.0) as total
            FROM usage_records
            WHERE tenant_id = $1 AND metric = $2 AND recorded_at >= $3 AND recorded_at <= $4
            "#,
        )
        .bind(tenant_id)
        .bind(metric)
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&self.pool)
        .await?;

        Ok(total)
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct PlanRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub status: String,
    pub price_monthly: rust_decimal::Decimal,
    pub price_yearly: rust_decimal::Decimal,
    pub currency: String,
    pub features: serde_json::Value,
    pub quotas: serde_json::Value,
    pub sort_order: i32,
    pub is_public: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: Uuid,
    pub status: String,
    pub current_period_start: chrono::DateTime<chrono::Utc>,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub trial_end: Option<chrono::DateTime<chrono::Utc>>,
    pub quantity: i32,
    pub unit_price: rust_decimal::Decimal,
    pub currency: String,
    pub next_billing_date: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct InvoiceRow {
    pub id: Uuid,
    pub invoice_number: String,
    pub subscription_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub status: String,
    pub subtotal: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub total: rust_decimal::Decimal,
    pub currency: String,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub due_at: chrono::DateTime<chrono::Utc>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub line_items: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
