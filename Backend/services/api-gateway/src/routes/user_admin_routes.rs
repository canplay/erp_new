//! 用户管理路由 — 用户 CRUD、搜索、批量操作

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
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

/// 获取 user-service gRPC 客户端
async fn get_user_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::UserGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("user-service 不可用: {e}")))
}

/// proto GetUserResponse → JSON
fn user_info_to_json(u: &grpc_proto::user::GetUserResponse) -> Value {
    json!({
        "id": u.id,
        "username": u.username,
        "nickname": u.nickname,
        "avatar": u.avatar,
        "phone": u.phone,
        "email": u.email,
        "gender": u.gender,
        "address": u.address,
        "role": u.role,
        "status": u.status,
        "created_at": u.created_at,
        "updated_at": u.updated_at,
    })
}

async fn list_users_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_users(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.keyword.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.users.iter().map(user_info_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_user_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_user(
        body["username"].as_str().unwrap_or("").to_string(),
        body["password"].as_str().unwrap_or("").to_string(),
        body["email"].as_str().unwrap_or("").to_string(),
        body["nickname"].as_str().unwrap_or("").to_string(),
        body["phone"].as_str().unwrap_or("").to_string(),
        body["gender"].as_i64().unwrap_or(0) as i32,
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id, "username": resp.username, "nickname": resp.nickname})),
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn get_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_user(id).await {
        Ok(resp) => json_success(user_info_to_json(&resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn update_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_user(
        id,
        body["nickname"].as_str().unwrap_or("").to_string(),
        body["avatar"].as_str().unwrap_or("").to_string(),
        body["gender"].as_i64().unwrap_or(0) as i32,
        body["address"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id, "username": resp.username, "nickname": resp.nickname})),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_user(id).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

async fn search_users_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_users(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.keyword.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.users.iter().map(user_info_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("搜索失败: {e}")),
    }
}

async fn update_user_status_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    let status = body["status"].as_i64().unwrap_or(0) as i32;
    let lock_hours = body["lock_hours"].as_i64().unwrap_or(0) as i32;
    match client.update_user_status(id, status, lock_hours).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn update_user_role_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_user_role(id, body["role"].as_str().unwrap_or("").to_string()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn reset_password_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.reset_password(id, body["new_password"].as_str().unwrap_or("").to_string()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("重置失败: {e}")),
    }
}

async fn batch_update_role_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let user_ids: Vec<i64> = body["user_ids"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
    let role = body["role"].as_str().unwrap_or("").to_string();
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.batch_update_user_role(user_ids, role).await {
        Ok(resp) => json_success(json!({"affected": resp.affected})),
        Err(e) => json_error(&format!("批量更新失败: {e}")),
    }
}

async fn batch_update_status_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let user_ids: Vec<i64> = body["user_ids"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
    let status = body["status"].as_i64().unwrap_or(0) as i32;
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.batch_update_user_status(user_ids, status).await {
        Ok(resp) => json_success(json!({"affected": resp.affected})),
        Err(e) => json_error(&format!("批量更新失败: {e}")),
    }
}

async fn batch_delete_users_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let user_ids: Vec<i64> = body["user_ids"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.batch_delete_users(user_ids).await {
        Ok(resp) => json_success(json!({"deleted": resp.deleted})),
        Err(e) => json_error(&format!("批量删除失败: {e}")),
    }
}

async fn not_implemented_handler() -> Json<Value> {
    json_not_implemented("导入导出待实现")
}

async fn import_users_handler() -> Json<Value> {
    json_not_implemented("导入待实现")
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/users", get(list_users_handler).post(create_user_handler))
        .route("/api/admin/users/search", get(search_users_handler))
        .route("/api/admin/users/{id}", get(get_user_handler).put(update_user_handler).delete(delete_user_handler))
        .route("/api/admin/users/{id}/status", put(update_user_status_handler))
        .route("/api/admin/users/{id}/role", put(update_user_role_handler))
        .route("/api/admin/users/{id}/reset-password", post(reset_password_handler))
        .route("/api/admin/users/batch-role", put(batch_update_role_handler))
        .route("/api/admin/users/batch-status", put(batch_update_status_handler))
        .route("/api/admin/users/batch-delete", post(batch_delete_users_handler))
        .route("/api/admin/users/export", get(not_implemented_handler))
        .route("/api/admin/users/import-template", get(not_implemented_handler))
        .route("/api/admin/users/import-template/download", get(not_implemented_handler))
        .route("/api/admin/users/import", post(import_users_handler))
}
