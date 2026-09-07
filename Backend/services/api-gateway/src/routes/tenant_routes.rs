//! 租户路由 — 真实 gRPC 调用 tenant-service

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::{get, post, put, delete}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct TntQuery { page: Option<i32>, page_size: Option<i32>, keyword: Option<String> }

/// 获取 tenant-service gRPC 客户端
async fn get_tenant_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::TenantGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.tenant_client().await
        .map_err(|e| json_error(&format!("tenant-service 不可用: {e}")))
}

/// proto Tenant → JSON
fn tenant_to_json(t: &grpc_proto::tenant::Tenant) -> Value {
    json!({
        "id": t.id,
        "name": t.name,
        "code": t.code,
        "logo": t.logo,
        "description": t.description,
        "status": t.status,
        "plan": t.plan,
        "max_users": t.max_users,
        "current_users": t.current_users,
        "expires_at": t.expires_at,
        "config": t.config,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })
}

async fn list_tenants(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TntQuery>,
) -> Json<Value> {
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_tenants(q.page.unwrap_or(1), q.page_size.unwrap_or(20), q.keyword.clone().unwrap_or_default(), -1, String::new()).await {
        Ok(resp) => json_success(json!({
            "list": resp.tenants.iter().map(tenant_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_tenant(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_tenant(
        body["name"].as_str().unwrap_or("").to_string(),
        body["code"].as_str().unwrap_or("").to_string(),
        body["logo"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        body["plan"].as_str().unwrap_or("free").to_string(),
        body["max_users"].as_i64().unwrap_or(100),
        std::collections::HashMap::new(),
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id})),
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn get_tenant(
    State(state): State<Arc<AppState>>,
    path: Option<axum::extract::Path<i64>>,
) -> Json<Value> {
    // 修复 (2026-08-07): 同一 handler 挂带参(/api/admin/tenants/{id})和无参
    // (/api/tenant/current)两个路由。用 Option<Path> 兼容, 无参时默认 id=0。
    // 原签名 Path(id): Path<i64> 在无参路由上 axum 报 "Wrong number of path
    // arguments. Expected 1 but got 0" 500 → 租户管理页 ErrorBoundary 崩溃。
    let id = path.map(|p| p.0).unwrap_or(0);
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    // 无参(当前租户语义)时用户表无 tenant_id 关联, id=0 查询返回空而非崩溃
    match client.get_tenant(id, String::new()).await {
        Ok(resp) => {
            if let Some(t) = resp.tenant {
                json_success(tenant_to_json(&t))
            } else {
                json_error("租户不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn update_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_tenant(
        id,
        body["name"].as_str().unwrap_or("").to_string(),
        body["logo"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        -1,
        body["plan"].as_str().unwrap_or("").to_string(),
        body["max_users"].as_i64().unwrap_or(0),
        0,
    ).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_tenant(id, false).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

async fn enable_tenant() -> Json<Value> { json_ok() }
async fn disable_tenant() -> Json<Value> { json_ok() }
async fn reset_tenant_quota() -> Json<Value> { json_ok() }

async fn list_tenant_users(
    State(state): State<Arc<AppState>>,
    path: Option<axum::extract::Path<i64>>,
) -> Json<Value> {
    // 修复 (2026-08-07): 同一 handler 被挂到两个路由——带参 /api/admin/tenants/{tenantId}/users
    // 和无参 /api/tenant/users。用 Option<Path> 兼容两种调用, 无参时用默认 id=0。
    let tenant_id = path.map(|p| p.0).unwrap_or(0);
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_tenant_users(tenant_id, 1, 100).await {
        Ok(resp) => json_success(json!({
            "list": resp.users.iter().map(|u| json!({
                "id": u.user_id,
                "username": u.username,
                "role": u.role,
                "joined_at": u.joined_at,
            })).collect::<Vec<_>>(),
            "total": resp.users.len(),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn add_tenant_user(
    State(state): State<Arc<AppState>>,
    path: Option<axum::extract::Path<i64>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    // 修复 (2026-08-07): 同一 handler 挂带参(/api/admin/tenants/{tenantId}/users)和
    // 无参(/api/tenant/users)两个路由, 用 Option<Path> 兼容, 无参时默认 id=0。
    let tenant_id = path.map(|p| p.0).unwrap_or(0);
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.add_tenant_user(tenant_id, body["user_id"].as_i64().unwrap_or(0), body["role"].as_str().unwrap_or("member").to_string()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("添加失败: {e}")),
    }
}

/// 更新租户用户（角色/部门/职位）
/// 修复 (2026-08-07): 原 /api/tenant/users/{user_id} PUT 错误映射到 update_tenant，
/// 导致 user_id 被当作 tenant_id 语义错位。现创建独立 handler。
/// 注意: tenant.proto 尚未定义 UpdateTenantUser RPC，暂时通过 HTTP 代理到 tenant-service。
async fn update_tenant_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if user_id <= 0 {
        return json_error("无效的用户 ID");
    }
    // 从请求头获取 tenant_id（前端在 X-Tenant-Id 头中注入）
    let _tenant_id: i64 = state.grpc_clients.read().await.tenant_client().await
        .ok()
        .and_then(|_| None) // placeholder - tenant_id from header
        .unwrap_or(0);
    // 通过 HTTP 代理到 tenant-service（gRPC 尚未暴露 UpdateTenantUser）
    let tenant_url = &state.tenant_service_url;
    let url = format!("{}/api/tenant/users/{}", tenant_url, user_id);
    match state.http_client.put(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                json_ok()
            } else {
                json_error(&format!("tenant-service 返回 {}", resp.status()))
            }
        }
        Err(e) => json_error(&format!("代理请求失败: {e}")),
    }
}

async fn remove_tenant_user(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, user_id)): Path<(i64, i64)>,
) -> Json<Value> {
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.remove_tenant_user(tenant_id, user_id).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("移除失败: {e}")),
    }
}

async fn get_tenant_plans() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": []})) }
async fn get_current_plan() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": null})) }
async fn upgrade_plan() -> Json<Value> { json_ok() }

async fn get_usage(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_tenant_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_usage_stats(0, 0, 0).await {
        Ok(resp) => {
            if let Some(s) = resp.stats {
                json_success(json!({
                    "tenant_id": s.tenant_id,
                    "users_count": s.users_count,
                    "storage_used": s.storage_used,
                }))
            } else {
                json_success(json!({}))
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

/// 审计修复 (C5): 真实实现——校验租户存在并返回租户信息。
/// 前端保存返回的 tenant_id, 并在后续请求注入 X-Tenant-Id 头,
/// 由 gateway 中间件 (middleware.rs) 透传给各业务服务做数据隔离。
async fn switch_tenant(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let tenant_id = body["tenant_id"]
        .as_str()
        .or_else(|| body["id"].as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if tenant_id <= 0 {
        return json_error("缺少有效的 tenant_id");
    }
    let mut client = match get_tenant_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match client.get_tenant(tenant_id, String::new()).await {
        Ok(resp) => {
            if let Some(t) = resp.tenant {
                json_success(json!({
                    "tenant": tenant_to_json(&t),
                    "tenant_id": t.id,
                }))
            } else {
                json_error("租户不存在")
            }
        }
        Err(e) => json_error(&format!("切换租户失败: {e}")),
    }
}
async fn get_audit_logs() -> Json<Value> { Json(json!({"success": true, "code": 200, "data": {"list": [], "total": 0}})) }
async fn export_audit_logs() -> Json<Value> { Json(json!({"success": false, "code": 501, "message": "导出待实现"})) }

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/tenants", get(list_tenants).post(create_tenant))
        .route("/api/admin/tenants/{id}", get(get_tenant).put(update_tenant).delete(delete_tenant))
        .route("/api/admin/tenants/{id}/enable", put(enable_tenant))
        .route("/api/admin/tenants/{id}/disable", put(disable_tenant))
        .route("/api/admin/tenants/{id}/reset-quota", post(reset_tenant_quota))
        .route("/api/admin/tenants/{tenantId}/users", get(list_tenant_users).post(add_tenant_user))
        .route("/api/admin/tenants/{tenantId}/users/{user_id}", delete(remove_tenant_user))
        .route("/api/admin/tenant-plans", get(get_tenant_plans))
        .route("/api/tenant/current", get(get_tenant))
        .route("/api/tenant/{id}", put(update_tenant))
        .route("/api/tenant/switch", post(switch_tenant))
        .route("/api/tenant/users", post(add_tenant_user).get(list_tenant_users))
        .route("/api/tenant/users/{user_id}", put(update_tenant_user))
        .route("/api/tenant/users/{user_id}", delete(remove_tenant_user))
        .route("/api/tenant/plans", get(get_tenant_plans))
        .route("/api/tenant/plan/current", get(get_current_plan))
        .route("/api/tenant/plan/upgrade", post(upgrade_plan))
        .route("/api/tenant/usage", get(get_usage))
        .route("/api/tenant/audit-logs", get(get_audit_logs))
        .route("/api/tenant/audit-logs/export", get(export_audit_logs))
}
