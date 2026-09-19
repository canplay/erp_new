//! 计费服务 HTTP 处理器

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use billing_core::{
    BillingPlan, BillingPlanType, Invoice, InvoiceLineItem, InvoiceStatus,
    PlanFeature, Subscription, SubscriptionStatus, UsageRecord, UsageType,
};
use common::{ApiResponse, AppError, AppResult};

use crate::models::{SubscriptionStateMachine, QuotaCheckResult, UsageSummary};

/// 应用状态
#[derive(Clone)]
pub struct BillingAppState {
    pub pool: PgPool,
}

impl BillingAppState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ============ 计划管理 ============

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub price_monthly: Decimal,
    pub price_yearly: Decimal,
    pub currency: Option<String>,
    pub features: Vec<PlanFeatureRequest>,
    pub quotas: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanFeatureRequest {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub value: Option<String>,
}

pub async fn create_plan(
    State(state): State<Arc<BillingAppState>>,
    Json(request): Json<CreatePlanRequest>,
) -> AppResult<impl IntoResponse> {
    let plan_type = match request.plan_type.as_str() {
        "free" => BillingPlanType::Free,
        "standard" => BillingPlanType::Standard,
        "enterprise" => BillingPlanType::Enterprise,
        _ => BillingPlanType::Custom,
    };

    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO plans (id, name, description, plan_type, status, price_monthly, price_yearly, currency, features, quotas, is_public)
        VALUES ($1, $2, $3, $4, 'active', $5, $6, $7, $8, $9, true)
        "#,
    )
    .bind(id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(plan_type.as_str())
    .bind(request.price_monthly)
    .bind(request.price_yearly)
    .bind(request.currency.as_deref().unwrap_or("CNY"))
    .bind(serde_json::to_value(&request.features).unwrap_or_default())
    .bind(serde_json::to_value(&request.quotas).unwrap_or_default())
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(id))))
}

pub async fn list_plans(
    State(state): State<Arc<BillingAppState>>,
) -> AppResult<impl IntoResponse> {
    let plans: Vec<serde_json::Value> = sqlx::query_as::<_, PlanRow>(
        r#"
        SELECT id, name, description, plan_type, status, price_monthly, price_yearly, 
               currency, features, quotas, sort_order, is_public, created_at, updated_at
        FROM plans WHERE is_public = true ORDER BY sort_order
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "id": row.id,
            "name": row.name,
            "description": row.description,
            "plan_type": row.plan_type,
            "status": row.status,
            "price_monthly": row.price_monthly,
            "price_yearly": row.price_yearly,
            "currency": row.currency,
            "features": row.features,
            "quotas": row.quotas,
        })
    })
    .collect();

    Ok(Json(ApiResponse::success(plans)))
}

// ============ 订阅管理 ============

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub tenant_id: Uuid,
    pub plan_id: Uuid,
}

pub async fn create_subscription(
    State(state): State<Arc<BillingAppState>>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> AppResult<impl IntoResponse> {
    // 获取计划价格
    let plan: PlanRow = sqlx::query_as::<_, PlanRow>(
        "SELECT id, name, description, plan_type, status, price_monthly, price_yearly, \
         currency, features, quotas, sort_order, is_public, created_at, updated_at \
         FROM plans WHERE id = $1",
    )
    .bind(request.plan_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::NotFound("Plan not found".to_string()))?;

    let now = chrono::Utc::now();
    let trial_end = now + chrono::Duration::days(14);
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO subscriptions (id, tenant_id, plan_id, status, current_period_start, current_period_end, trial_end, unit_price)
        VALUES ($1, $2, $3, 'trialing', $4, $5, $6, $7)
        "#,
    )
    .bind(id)
    .bind(request.tenant_id)
    .bind(request.plan_id)
    .bind(now)
    .bind(trial_end)
    .bind(trial_end)
    .bind(plan.price_monthly)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(id))))
}

// ============ 状态机流转 ============

#[derive(Debug, Deserialize)]
pub struct SubscriptionTransitionRequest {
    pub subscription_id: Uuid,
    pub action: String, // "activate", "mark_past_due", "reactivate", "cancel", "expire"
}

#[derive(Debug, Serialize)]
pub struct SubscriptionTransitionResponse {
    pub subscription_id: Uuid,
    pub previous_status: String,
    pub new_status: String,
    pub success: bool,
}

pub async fn transition_subscription(
    State(state): State<Arc<BillingAppState>>,
    Json(request): Json<SubscriptionTransitionRequest>,
) -> AppResult<impl IntoResponse> {
    let mut subscription: SubscriptionRow = sqlx::query_as::<_, SubscriptionRow>(
        r#"
        SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end,
               cancel_at_period_end, canceled_at, trial_end, quantity, unit_price, currency,
               next_billing_date, metadata, created_at, updated_at
        FROM subscriptions WHERE id = $1
        "#,
    )
    .bind(request.subscription_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::NotFound("Subscription not found".to_string()))?;

    let previous_status = subscription.status.clone();
    let new_status = match request.action.as_str() {
        "activate" => "active",
        "mark_past_due" => "past_due",
        "reactivate" => "active",
        "cancel" => "cancelled",
        "expire" => "expired",
        _ => return Err(AppError::InvalidParam(format!("Unknown action: {}", request.action))),
    };

    sqlx::query(
        r#"
        UPDATE subscriptions SET status = $1, updated_at = $2 WHERE id = $3
        "#,
    )
    .bind(new_status)
    .bind(chrono::Utc::now())
    .bind(request.subscription_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let response = SubscriptionTransitionResponse {
        subscription_id: request.subscription_id,
        previous_status,
        new_status: new_status.to_string(),
        success: true,
    };

    Ok(Json(ApiResponse::success(response)))
}

// ============ 发票管理 ============

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub subscription_id: Uuid,
    pub tenant_id: Uuid,
    pub amount: Decimal,
}

pub async fn create_invoice(
    State(state): State<Arc<BillingAppState>>,
    Json(request): Json<CreateInvoiceRequest>,
) -> AppResult<impl IntoResponse> {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();
    let invoice_number = format!("INV-{}", now.format("%Y%m%d%H%M%S"));

    // 获取订阅信息以生成发票项目
    let subscription: SubscriptionRow = sqlx::query_as::<_, SubscriptionRow>(
        r#"
        SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end,
               cancel_at_period_end, canceled_at, trial_end, quantity, unit_price, currency,
               next_billing_date, metadata, created_at, updated_at
        FROM subscriptions WHERE id = $1
        "#,
    )
    .bind(request.subscription_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::NotFound("Subscription not found".to_string()))?;

    // 获取计划信息
    let plan: PlanRow = sqlx::query_as::<_, PlanRow>(
        "SELECT id, name, description, plan_type, status, price_monthly, price_yearly, \
         currency, features, quotas, sort_order, is_public, created_at, updated_at \
         FROM plans WHERE id = $1",
    )
    .bind(subscription.plan_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::NotFound("Plan not found".to_string()))?;

    // 获取当前周期的用量统计
    let usage_rows: Vec<(String, f64)> = sqlx::query_as::<_, (String, f64)>(
        r#"
        SELECT metric, COALESCE(SUM(quantity), 0.0) as total
        FROM usage_records
        WHERE tenant_id = $1 AND recorded_at >= $2 AND recorded_at <= $3
        GROUP BY metric
        "#,
    )
    .bind(request.tenant_id)
    .bind(subscription.current_period_start)
    .bind(subscription.current_period_end)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // 生成发票行项目
    let mut line_items: Vec<serde_json::Value> = Vec::new();
    let mut subtotal = Decimal::ZERO;

    // 添加订阅费行项目
    let subscription_fee = subscription.unit_price * Decimal::from(subscription.quantity);
    line_items.push(serde_json::json!({
        "description": format!("{} Plan Subscription ({} seat(s))", plan.name, subscription.quantity),
        "quantity": subscription.quantity,
        "unit_price": subscription.unit_price,
        "amount": subscription_fee,
    }));
    subtotal += subscription_fee;

    // 添加用量超额行项目（基于计划配额）
    let plan_quotas: std::collections::HashMap<String, i64> =
        serde_json::from_value(plan.quotas.clone()).unwrap_or_default();

    for (metric, total_usage) in &usage_rows {
        if let Some(&quota_limit) = plan_quotas.get(metric) {
            let usage_f64 = *total_usage;
            if usage_f64 > quota_limit as f64 {
                let overage = usage_f64 - quota_limit as f64;
                // 计算超额费用（每单位 0.01）
                let overage_fee = Decimal::try_from(overage * 0.01)
                    .unwrap_or(Decimal::ZERO)
                    .round_dp(2);
                line_items.push(serde_json::json!({
                    "description": format!("Overage for {}: {:.1} units over quota", metric, overage),
                    "quantity": overage as i64,
                    "unit_price": 0.01,
                    "amount": overage_fee,
                }));
                subtotal += overage_fee;
            }
        }
    }

    let tax_amount = (subtotal * Decimal::try_from(0.06).unwrap_or(Decimal::ZERO)).round_dp(2);
    let total = subtotal + tax_amount;

    let line_items_json = serde_json::to_value(&line_items).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO invoices (id, invoice_number, subscription_id, tenant_id, status, subtotal, tax_amount, total, period_start, period_end, due_at, line_items)
        VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(id)
    .bind(&invoice_number)
    .bind(request.subscription_id)
    .bind(request.tenant_id)
    .bind(subtotal)
    .bind(tax_amount)
    .bind(total)
    .bind(subscription.current_period_start)
    .bind(subscription.current_period_end)
    .bind(now + chrono::Duration::days(7))
    .bind(&line_items_json)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let result = serde_json::json!({
        "id": id,
        "invoice_number": invoice_number,
        "subtotal": subtotal,
        "tax_amount": tax_amount,
        "total": total,
        "line_items": line_items,
    });

    Ok((StatusCode::CREATED, Json(ApiResponse::success(result))))
}

// ============ 用量管理 ============

#[derive(Debug, Deserialize)]
pub struct RecordUsageRequest {
    pub tenant_id: Uuid,
    pub metric: String,
    pub quantity: f64,
}

#[derive(Debug, Serialize)]
pub struct UsageRecordResponse {
    pub tenant_id: Uuid,
    pub metric: String,
    pub quantity: f64,
    pub period_usage: f64,
    pub quota_check: Option<QuotaCheckResponse>,
}

#[derive(Debug, Serialize)]
pub struct QuotaCheckResponse {
    pub quota_limit: i64,
    pub exceeded: bool,
    pub overage: f64,
}

pub async fn record_usage(
    State(state): State<Arc<BillingAppState>>,
    Json(request): Json<RecordUsageRequest>,
) -> AppResult<impl IntoResponse> {
    // 记录用量
    sqlx::query(
        "INSERT INTO usage_records (tenant_id, metric, quantity) VALUES ($1, $2, $3)",
    )
    .bind(request.tenant_id)
    .bind(&request.metric)
    .bind(request.quantity)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // 获取租户的活跃订阅
    let subscription: Option<SubscriptionRow> = sqlx::query_as::<_, SubscriptionRow>(
        r#"
        SELECT id, tenant_id, plan_id, status, current_period_start, current_period_end,
               cancel_at_period_end, canceled_at, trial_end, quantity, unit_price, currency,
               next_billing_date, metadata, created_at, updated_at
        FROM subscriptions WHERE tenant_id = $1 AND status IN ('active', 'trialing')
        ORDER BY created_at DESC LIMIT 1
        "#,
    )
    .bind(request.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut quota_check_response = None;

    if let Some(sub) = &subscription {
        // 获取计划配额
        let plan: Option<PlanRow> = sqlx::query_as::<_, PlanRow>(
            "SELECT id, name, description, plan_type, status, price_monthly, price_yearly, \
             currency, features, quotas, sort_order, is_public, created_at, updated_at \
             FROM plans WHERE id = $1",
        )
        .bind(sub.plan_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some(plan_row) = plan {
            // 获取当前周期的用量汇总
            let period_total: f64 = sqlx::query_scalar::<_, f64>(
                r#"
                SELECT COALESCE(SUM(quantity), 0.0) as total
                FROM usage_records
                WHERE tenant_id = $1 AND metric = $2 AND recorded_at >= $3 AND recorded_at <= $4
                "#,
            )
            .bind(request.tenant_id)
            .bind(&request.metric)
            .bind(sub.current_period_start)
            .bind(sub.current_period_end)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            // 检查配额限制
            let plan_quotas: std::collections::HashMap<String, i64> =
                serde_json::from_value(plan_row.quotas.clone()).unwrap_or_default();

            if let Some(&quota_limit) = plan_quotas.get(&request.metric) {
                let exceeded = period_total > quota_limit as f64;
                let overage = if exceeded {
                    period_total - quota_limit as f64
                } else {
                    0.0
                };
                quota_check_response = Some(QuotaCheckResponse {
                    quota_limit,
                    exceeded,
                    overage,
                });
            }

            let response = UsageRecordResponse {
                tenant_id: request.tenant_id,
                metric: request.metric.clone(),
                quantity: request.quantity,
                period_usage: period_total,
                quota_check: quota_check_response,
            };

            return Ok(Json(ApiResponse::success(response)));
        }
    }

    // 无订阅或计划时返回基础响应
    let response = UsageRecordResponse {
        tenant_id: request.tenant_id,
        metric: request.metric,
        quantity: request.quantity,
        period_usage: request.quantity,
        quota_check: None,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 获取用量汇总
pub async fn get_usage_summary(
    State(state): State<Arc<BillingAppState>>,
    Path(tenant_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<impl IntoResponse> {
    let start_time = params
        .get("start")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));

    let end_time = params
        .get("end")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now());

    let summaries: Vec<UsageSummaryRow> = sqlx::query_as::<_, UsageSummaryRow>(
        r#"
        SELECT metric, COALESCE(SUM(quantity), 0.0) as total_quantity,
               MIN(recorded_at) as period_start, MAX(recorded_at) as period_end
        FROM usage_records
        WHERE tenant_id = $1 AND recorded_at >= $2 AND recorded_at <= $3
        GROUP BY metric
        "#,
    )
    .bind(tenant_id)
    .bind(start_time)
    .bind(end_time)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let result: Vec<serde_json::Value> = summaries
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "metric": row.metric,
                "total_quantity": row.total_quantity,
                "period_start": row.period_start,
                "period_end": row.period_end,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(result)))
}

// ============ 错误类型 ============

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    plan_type: String,
    status: String,
    price_monthly: rust_decimal::Decimal,
    price_yearly: rust_decimal::Decimal,
    currency: String,
    features: serde_json::Value,
    quotas: serde_json::Value,
    sort_order: i32,
    is_public: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    id: Uuid,
    tenant_id: Uuid,
    plan_id: Uuid,
    status: String,
    current_period_start: chrono::DateTime<chrono::Utc>,
    current_period_end: chrono::DateTime<chrono::Utc>,
    cancel_at_period_end: bool,
    canceled_at: Option<chrono::DateTime<chrono::Utc>>,
    trial_end: Option<chrono::DateTime<chrono::Utc>>,
    quantity: i32,
    unit_price: rust_decimal::Decimal,
    currency: String,
    next_billing_date: Option<chrono::DateTime<chrono::Utc>>,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct UsageSummaryRow {
    metric: String,
    total_quantity: f64,
    period_start: chrono::DateTime<chrono::Utc>,
    period_end: chrono::DateTime<chrono::Utc>,
}
