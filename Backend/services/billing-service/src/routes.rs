//! 计费服务路由

use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::BillingAppState;

/// 创建计费服务路由
pub fn billing_routes(state: Arc<BillingAppState>) -> Router {
    Router::new()
        // 计划管理
        .route("/api/v1/plans", post(handlers::create_plan).get(handlers::list_plans))
        .route("/api/v1/plans/:id", get(handlers::get_plan).put(handlers::update_plan))
        // 订阅管理
        .route("/api/v1/subscriptions", post(handlers::create_subscription).get(handlers::list_subscriptions))
        .route("/api/v1/subscriptions/:id", get(handlers::get_subscription).put(handlers::update_subscription))
        .route("/api/v1/tenants/:tenant_id/subscription", get(handlers::get_tenant_subscription))
        // 发票管理
        .route("/api/v1/invoices", post(handlers::create_invoice).get(handlers::list_invoices))
        .route("/api/v1/invoices/:id", get(handlers::get_invoice).put(handlers::update_invoice))
        .route("/api/v1/invoices/:id/pay", post(handlers::pay_invoice))
        // 用量管理
        .route("/api/v1/usage", post(handlers::record_usage).get(handlers::get_usage))
        .route("/api/v1/quotas/:tenant_id", get(handlers::check_quota))
        .with_state(state)
}

/// 健康检查路由
pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/ready", get(|| async { "Ready" }))
}
