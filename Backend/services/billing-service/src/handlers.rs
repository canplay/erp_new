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

    let tax_amount = request.amount * Decimal::try_from(0.06).unwrap_or(Decimal::ZERO);
    let total = request.amount + tax_amount;

    sqlx::query(
        r#"
        INSERT INTO invoices (id, invoice_number, subscription_id, tenant_id, status, subtotal, tax_amount, total, period_start, period_end, due_at)
        VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(id)
    .bind(&invoice_number)
    .bind(request.subscription_id)
    .bind(request.tenant_id)
    .bind(request.amount)
    .bind(tax_amount)
    .bind(total)
    .bind(now)
    .bind(now + chrono::Duration::days(30))
    .bind(now + chrono::Duration::days(7))
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(id))))
}

// ============ 用量管理 ============

#[derive(Debug, Deserialize)]
pub struct RecordUsageRequest {
    pub tenant_id: Uuid,
    pub metric: String,
    pub quantity: f64,
}

pub async fn record_usage(
    State(state): State<Arc<BillingAppState>>,
    Json(request): Json<RecordUsageRequest>,
) -> AppResult<impl IntoResponse> {
    sqlx::query(
        "INSERT INTO usage_records (tenant_id, metric, quantity) VALUES ($1, $2, $3)",
    )
    .bind(request.tenant_id)
    .bind(&request.metric)
    .bind(request.quantity)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(ApiResponse::success("Usage recorded")))
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