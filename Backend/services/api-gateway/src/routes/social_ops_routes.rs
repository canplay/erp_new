//! Social Ops 路由 — gRPC 调用

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Path}, routing::get};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

macro_rules! sops_client {
    ($state:expr, $client:ident, $method:ident) => {
        $state.grpc_clients.write().await.$client().await
            .map_err(|e| json_error(&format!("social-ops 不可用: {e}")))
    };
}

async fn list_accounts(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut c = match sops_client!(&state, social_ops_client, list_accounts) { Ok(c) => c, Err(r) => return r };
    match c.list_accounts().await {
        Ok(r) => json_success(json!(r.accounts)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_account(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let mut c = match sops_client!(&state, social_ops_client, list_accounts) { Ok(c) => c, Err(r) => return r };
    match c.get_account(&id).await {
        Ok(r) => json_success(json!(r)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/social-ops/health", get(|| async { Json(json!({"status":"ok"})) }))
        .route("/api/v1/social-ops/accounts", get(list_accounts))
        .route("/api/v1/social-ops/accounts/{id}", get(get_account))
}
