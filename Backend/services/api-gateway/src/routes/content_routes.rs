//! 内容管理路由 — 公告 + 社交媒体运营
//!
//! 合并自 announcement_routes.rs + social_ops_routes.rs

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
    pub role: Option<String>,
}

// ==================== 公告管理 ====================

async fn list_announcements_handler(
    Query(q): Query<PageQuery>,
) -> Json<Value> {
    json_success(json!({"list": [], "total": 0, "page": q.page.unwrap_or(1), "page_size": q.page_size.unwrap_or(20)}))
}

async fn create_announcement_handler() -> Json<Value> {
    json_success(json!({"id": 0}))
}

async fn get_active_announcements_handler() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn pin_announcement_handler() -> Json<Value> { json_ok() }
async fn unpin_announcement_handler() -> Json<Value> { json_ok() }
async fn set_announcement_active_handler() -> Json<Value> { json_ok() }

async fn get_announcement_detail(Path(id): Path<i64>) -> Json<Value> {
    json_success(json!({"id": id}))
}

async fn update_announcement_detail() -> Json<Value> { json_ok() }
async fn delete_announcement_detail() -> Json<Value> { json_ok() }

// ==================== 社交媒体运营 ====================

macro_rules! sops_client {
    ($state:expr, $client:ident, $method:ident) => {
        $state.grpc_clients.read().await.$client().await
            .map_err(|e| json_error(&format!("social-ops 不可用: {e}" )))
    };
}

async fn list_social_accounts(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut c = match sops_client!(&state, social_ops_client, list_accounts) { Ok(c) => c, Err(r) => return r };
    match c.list_accounts().await {
        Ok(r) => json_success(json!(r.accounts)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn get_social_account(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let mut c = match sops_client!(&state, social_ops_client, list_accounts) { Ok(c) => c, Err(r) => return r };
    match c.get_account(&id).await {
        Ok(r) => json_success(json!(r)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn social_ops_health() -> Json<Value> { json_ok() }

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 公告
        .route("/api/admin/announcements", get(list_announcements_handler).post(create_announcement_handler))
        .route("/api/admin/announcements/{id}/pin", put(pin_announcement_handler).delete(unpin_announcement_handler))
        .route("/api/admin/announcements/{id}/active", put(set_announcement_active_handler))
        .route("/api/announcements/active", get(get_active_announcements_handler))
        .route("/api/admin/announcements/{id}", get(get_announcement_detail).put(update_announcement_detail).delete(delete_announcement_detail))
        // 社交媒体
        .route("/api/v1/social-ops/health", get(social_ops_health))
        .route("/api/v1/social-ops/accounts", get(list_social_accounts))
        .route("/api/v1/social-ops/accounts/{id}", get(get_social_account))
}
