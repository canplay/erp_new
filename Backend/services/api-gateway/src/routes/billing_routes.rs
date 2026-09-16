use axum::{routing::get, Router, extract::State, Json};
use std::sync::Arc;
use serde_json::Value;
use crate::AppState;
use super::helpers::json_success;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/billing/plans" , get(list_plans))
        .route("/api/v1/billing/subscriptions" , get(list_subscriptions))
        .route("/api/v1/billing/invoices" , get(list_invoices))
}

async fn list_plans(State(_state): State<Arc<AppState>>) -> Json<Value> {
    json_success(serde_json::json!({"plans": []}))
}

async fn list_subscriptions(State(_state): State<Arc<AppState>>) -> Json<Value> {
    json_success(serde_json::json!({"subscriptions": []}))
}

async fn list_invoices(State(_state): State<Arc<AppState>>) -> Json<Value> {
    json_success(serde_json::json!({"invoices": []}))
}
