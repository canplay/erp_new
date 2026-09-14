use axum::{routing::get, Router, extract::State, Json};
use std::sync::Arc;
use serde_json::json;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/billing/plans", get(list_plans))
        .route("/api/v1/billing/subscriptions", get(list_subscriptions))
        .route("/api/v1/billing/invoices", get(list_invoices))
}

async fn list_plans(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({"plans": []}))
}

async fn list_subscriptions(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({"subscriptions": []}))
}

async fn list_invoices(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({"invoices": []}))
}
