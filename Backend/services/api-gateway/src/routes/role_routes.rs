//! 角色/权限管理路由 — 角色 CRUD、权限配置、角色用户

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

use crate::routes::helpers::*;

#[derive(Debug, Deserialize)]
pub struct RoleQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<i32>,
}

async fn get_user_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::UserGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("user-service 不可用: {e}")))
}

/// proto RoleInfo → JSON
fn role_info_to_json(r: &grpc_proto::user::RoleInfo) -> Value {
    json!({
        "id": r.id,
        "name": r.name,
        "description": r.description,
        "type": r.r#type,
        "status": r.status,
        "user_count": r.user_count,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
    })
}

async fn list_roles_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_roles(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.keyword.clone().unwrap_or_default(),
        q.r#type.clone().unwrap_or_default(),
        q.status.unwrap_or(-1),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.roles.iter().map(role_info_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_role_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_role(name).await {
        Ok(resp) => {
            if let Some(role) = resp.role {
                json_success(role_info_to_json(&role))
            } else {
                json_error("角色不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_role_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_role(
        body["name"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        body["type"].as_str().unwrap_or("custom").to_string(),
    ).await {
        Ok(resp) => {
            if let Some(role) = resp.role {
                json_success(role_info_to_json(&role))
            } else {
                json_ok()
            }
        }
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn update_role_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_role(
        name,
        body["description"].as_str().unwrap_or("").to_string(),
        body["type"].as_str().unwrap_or("custom").to_string(),
        body["status"].as_i64().unwrap_or(1) as i32,
    ).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_role_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_role(name).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

async fn get_role_permissions_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_role_permissions(name).await {
        Ok(resp) => json_success(json!(resp.permissions)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_role_permissions_admin_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_role_permissions(name).await {
        Ok(resp) => json_success(json!({ "permissions": resp.permissions })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn set_role_permissions_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let permissions: Vec<String> = body["permissions"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default();
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.set_role_permissions(name, permissions).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("设置失败: {e}")),
    }
}

async fn get_role_users_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(q): Query<PageQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };

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

    match client.get_role_users(name, q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await {
        Ok(resp) => json_success(json!({
            "list": resp.users.iter().map(user_info_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/roles", get(list_roles_handler).post(create_role_handler))
        .route("/api/roles/{name}", get(get_role_handler).put(update_role_handler).delete(delete_role_handler))
        .route("/api/roles/{name}/permissions", get(get_role_permissions_handler).put(set_role_permissions_handler))
        .route("/api/admin/roles/{role_name}/permissions", get(get_role_permissions_admin_handler))
        .route("/api/roles/{name}/users", get(get_role_users_handler))
}
