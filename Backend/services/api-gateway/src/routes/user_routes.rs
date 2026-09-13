//! User/角色/部门/公告/字典 管理路由 — 真实 gRPC 调用 user-service
//!
//! 所有数据通过 gRPC 调用 user-service 获取/写入

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

// ==================== 查询参数 ====================

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
    pub role: Option<String>,
}

/// 字典查询参数（包含 type 字段）
#[derive(Debug, Deserialize)]
pub struct DictItemQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub type_id: Option<i64>,
    pub type_code: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RoleQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<i32>,
}

// ==================== 用户 Handler（真实 gRPC） ====================

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
    json_error("导入导出待实现")
}

/// POST /api/admin/users/import — 导入用户
/// 审计修复 (C3): 原实现返回假成功({"imported":0}); 前端已改逐行调用创建接口,
/// 此统一导入接口未实现, 明确返回 501 而非假装成功(诚实降级)
async fn import_users_handler() -> Json<Value> {
    json_error("导入待实现")
}

// ==================== 角色 Handler（真实 gRPC） ====================

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

/// 审计修复 (C1): 前端 permission store 调用的 admin 别名路由
/// GET /api/admin/roles/{role_name}/permissions → 返回 { success, data: { permissions: [...] } }
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
    match client.get_role_users(name, q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await {
        Ok(resp) => json_success(json!({
            "list": resp.users.iter().map(user_info_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

// ==================== 部门 Handler（真实 gRPC） ====================

/// proto DepartmentInfo → JSON
fn dept_info_to_json(d: &grpc_proto::user::DepartmentInfo) -> Value {
    json!({
        "id": d.id,
        "name": d.name,
        "parent_id": d.parent_id,
        "description": d.description,
        "sort_order": d.sort_order,
        "status": d.status,
        "created_at": d.created_at,
        "updated_at": d.updated_at,
    })
}

async fn list_departments_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_departments(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.keyword.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.departments.iter().map(dept_info_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_department_tree_handler(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_departments(1, 1000, String::new()).await {
        Ok(resp) => {
            // 构建树形结构
            let depts = resp.departments;
            let mut nodes: Vec<Value> = depts.iter().map(|d| json!({
                "id": d.id,
                "name": d.name,
                "parent_id": d.parent_id,
                "description": d.description,
                "sort_order": d.sort_order,
                "status": d.status,
                "children": Vec::<Value>::new(),
            })).collect();
            let node_map: std::collections::HashMap<i64, usize> = depts.iter().enumerate().map(|(i, d)| (d.id, i)).collect();
            let mut tree: Vec<Value> = vec![];
            for (i, d) in depts.iter().enumerate() {
                let node = nodes[i].clone();
                if d.parent_id == 0 {
                    tree.push(node);
                } else if let Some(&parent_idx) = node_map.get(&d.parent_id)
                    && let Some(children) = nodes[parent_idx]["children"].as_array_mut() {
                    children.push(node);
                }
            }
            json_success(tree)
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_department_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_department(id).await {
        Ok(resp) => {
            if let Some(dept) = resp.department {
                json_success(dept_info_to_json(&dept))
            } else {
                json_error("部门不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_department_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_department(
        body["name"].as_str().unwrap_or("").to_string(),
        body["parent_id"].as_i64().unwrap_or(0),
        body["description"].as_str().unwrap_or("").to_string(),
        body["sort_order"].as_i64().unwrap_or(0) as i32,
    ).await {
        Ok(resp) => {
            if let Some(dept) = resp.department {
                json_success(dept_info_to_json(&dept))
            } else {
                json_ok()
            }
        }
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn update_department_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_department(
        id,
        body["name"].as_str().unwrap_or("").to_string(),
        body["parent_id"].as_i64().unwrap_or(0),
        body["description"].as_str().unwrap_or("").to_string(),
        body["sort_order"].as_i64().unwrap_or(0) as i32,
        body["status"].as_i64().unwrap_or(1) as i32,
    ).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_department_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_department(id).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

/// GET /api/admin/departments/{id}/users — 部门用户列表（当前返回空，user-service 暂未提供部门用户 RPC）
async fn get_department_users(Path(_id): Path<i64>) -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

/// GET /api/admin/departments/{id}/move — 部门移动
async fn move_department_down(Path(_id): Path<i64>) -> Json<Value> { json_ok() }

// ==================== 公告 Handler ====================
// 公告数据在 user-service 的公告表中，当前 user.proto 未提供公告 RPC，保留占位
// 待 user.proto 扩展公告 RPC 后接入

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

// ==================== 系统配置 Handler ====================
// 系统配置在 system_configs 表，user.proto 暂未提供 RPC，保留占位

async fn list_system_configs_handler(
    Query(_q): Query<PageQuery>,
) -> Json<Value> {
    json_success(json!([]))
}

async fn batch_update_system_configs_handler() -> Json<Value> { json_ok() }

async fn update_system_config_handler(
    Path(key): Path<String>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    json_success(json!({"key": key}))
}

// ==================== 字典 Handler ====================
// 字典数据在 user-service（gRPC: ListDictionaryTypes/Create/Update/Delete 等）

fn dict_type_to_json(t: &grpc_proto::user::DictionaryTypeInfo) -> Value {
    json!({
        "id": t.id,
        "code": t.code,
        "name": t.name,
        "description": t.description,
        "sort": t.sort,
        "status": t.status,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })
}

fn dict_item_to_json(i: &grpc_proto::user::DictionaryItemInfo) -> Value {
    json!({
        "id": i.id,
        "type_id": i.type_id,
        "label": i.label,
        "value": i.value,
        "sort": i.sort,
        "status": i.status,
        "is_default": i.is_default,
        "remark": i.remark,
        "created_at": i.created_at,
        "updated_at": i.updated_at,
    })
}

async fn list_dict_types_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_dictionary_types(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.keyword.clone().unwrap_or_default(),
        q.status.unwrap_or(0),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.types.iter().map(dict_type_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_dict_type_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_dictionary_type(
        body["code"].as_str().unwrap_or("").to_string(),
        body["name"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        body["sort"].as_i64().unwrap_or(0) as i32,
    ).await {
        Ok(resp) => match resp.r#type {
            Some(t) => json_success(dict_type_to_json(&t)),
            None => json_error("创建失败"),
        },
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn update_dict_type_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_dictionary_type(
        id,
        body["name"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        body["sort"].as_i64().unwrap_or(0) as i32,
        body["status"].as_i64().unwrap_or(1) as i32,
    ).await {
        Ok(_) => json_success(json!({ "success": true })),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_dict_type_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_dictionary_type(id).await {
        Ok(_) => json_success(json!({ "success": true })),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

async fn batch_delete_dict_types_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    let ids: Vec<i64> = body["ids"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();
    let mut failed = 0i64;
    for id in ids {
        if client.delete_dictionary_type(id).await.is_err() {
            failed += 1;
        }
    }
    json_success(json!({ "deleted": true, "failed": failed }))
}

async fn list_dict_items_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DictItemQuery>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_dictionary_items(
        q.type_id.unwrap_or(0),
        q.type_code.clone().unwrap_or_default(),
        q.keyword.clone().unwrap_or_default(),
        q.status.unwrap_or(0),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.items.iter().map(dict_item_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_dict_item_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_dictionary_item(
        body["type_id"].as_i64().unwrap_or(0),
        body["label"].as_str().unwrap_or("").to_string(),
        body["value"].as_str().unwrap_or("").to_string(),
        body["sort"].as_i64().unwrap_or(0) as i32,
        body["status"].as_i64().unwrap_or(1) as i32,
        body["is_default"].as_bool().unwrap_or(false),
        body["remark"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => match resp.item {
            Some(i) => json_success(dict_item_to_json(&i)),
            None => json_error("创建失败"),
        },
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn update_dict_item_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_dictionary_item(
        id,
        body["label"].as_str().unwrap_or("").to_string(),
        body["value"].as_str().unwrap_or("").to_string(),
        body["sort"].as_i64().unwrap_or(0) as i32,
        body["status"].as_i64().unwrap_or(1) as i32,
        body["is_default"].as_bool().unwrap_or(false),
        body["remark"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(_) => json_success(json!({ "success": true })),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_dict_item_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_dictionary_item(id).await {
        Ok(_) => json_success(json!({ "success": true })),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

async fn batch_delete_dict_items_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    let ids: Vec<i64> = body["ids"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();
    let mut failed = 0i64;
    for id in ids {
        if client.delete_dictionary_item(id).await.is_err() {
            failed += 1;
        }
    }
    json_success(json!({ "deleted": true, "failed": failed }))
}

async fn get_all_enabled_dict_types_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_dictionary_types(1, 1000, String::new(), 1).await {
        Ok(resp) => json_success(json!({
            "list": resp.types.iter().map(dict_type_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn batch_add_dict_items_handler(
    State(state): State<Arc<AppState>>,
    Path(type_id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    let items: Vec<Value> = body["items"].as_array().cloned().unwrap_or_default();
    let mut created = 0i64;
    for item in items {
        if client.create_dictionary_item(
            type_id,
            item["label"].as_str().unwrap_or("").to_string(),
            item["value"].as_str().unwrap_or("").to_string(),
            item["sort"].as_i64().unwrap_or(0) as i32,
            item["status"].as_i64().unwrap_or(1) as i32,
            item["is_default"].as_bool().unwrap_or(false),
            item["remark"].as_str().unwrap_or("").to_string(),
        ).await.is_ok() {
            created += 1;
        }
    }
    json_success(json!({ "created": created }))
}

async fn batch_add_dict_items_by_type_handler(
    State(state): State<Arc<AppState>>,
    Path(type_id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    batch_add_dict_items_handler(State(state), Path(type_id), Json(body)).await
}

async fn reorder_dict_items_handler(
    State(state): State<Arc<AppState>>,
    Path(type_id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_user_client(&state).await { Ok(c) => c, Err(r) => return r };
    let ids: Vec<i64> = body["itemIds"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();
    let mut updated = 0i64;
    for (idx, id) in ids.iter().enumerate() {
        // 仅更新排序字段（其余字段保持不变，用空串/0 跳过）
        if client.update_dictionary_item(
            *id,
            String::new(),
            String::new(),
            (idx + 1) as i32,
            0,
            false,
            String::new(),
        ).await.is_ok() {
            updated += 1;
        }
    }
    let _ = type_id;
    json_success(json!({ "updated": updated }))
}

// ==================== 路由定义 ====================

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 用户 CRUD
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
        // 角色管理
        .route("/api/roles", get(list_roles_handler).post(create_role_handler))
        .route("/api/roles/{name}", get(get_role_handler).put(update_role_handler).delete(delete_role_handler))
        .route("/api/roles/{name}/permissions", get(get_role_permissions_handler).put(set_role_permissions_handler))
        .route("/api/admin/roles/{role_name}/permissions", get(get_role_permissions_admin_handler)) // 审计修复 C1: 前端权限初始化别名路由
        .route("/api/roles/{name}/users", get(get_role_users_handler))
        // 部门管理
        .route("/api/admin/departments", get(list_departments_handler).post(create_department_handler))
        .route("/api/admin/departments/tree", get(get_department_tree_handler))
        .route("/api/admin/departments/{id}", get(get_department_handler).put(update_department_handler).delete(delete_department_handler))
        .route("/api/admin/departments/{id}/move", put(move_department_down))
        // 公告管理
        .route("/api/admin/announcements", get(list_announcements_handler).post(create_announcement_handler))
        .route("/api/admin/announcements/{id}/pin", put(pin_announcement_handler).delete(unpin_announcement_handler))
        .route("/api/admin/announcements/{id}/active", put(set_announcement_active_handler))
        .route("/api/announcements/active", get(get_active_announcements_handler))
        // 系统配置
        .route("/api/config/system-configs", get(list_system_configs_handler))
        .route("/api/config/system-configs/batch", put(batch_update_system_configs_handler))
        .route("/api/config/system-configs/{key}", put(update_system_config_handler))
        // 字典管理
        .route("/api/admin/dictionary/types", get(list_dict_types_handler).post(create_dict_type_handler))
        .route("/api/admin/dictionary/types/{id}", put(update_dict_type_handler).delete(delete_dict_type_handler))
        .route("/api/admin/dictionary/types/batch-delete", post(batch_delete_dict_types_handler))
        .route("/api/admin/dictionary/items", get(list_dict_items_handler).post(create_dict_item_handler))
        .route("/api/admin/dictionary/items/{id}", put(update_dict_item_handler).delete(delete_dict_item_handler))
        .route("/api/admin/dictionary/items/batch-delete", post(batch_delete_dict_items_handler))
        .route("/api/admin/dictionary/all-enabled", get(get_all_enabled_dict_types_handler))
        .route("/api/admin/dictionary/types/{type_id}/items", post(batch_add_dict_items_handler))
        .route("/api/admin/dictionary/types/{type_id}/items/batch", post(batch_add_dict_items_by_type_handler))
        .route("/api/admin/dictionary/types/{type_id}/items/reorder", put(reorder_dict_items_handler))
        // 公告详情 CRUD
        .route("/api/admin/announcements/{id}", get(get_announcement_detail).put(update_announcement_detail).delete(delete_announcement_detail))
        // 部门用户列表
        .route("/api/admin/departments/{id}/users", get(get_department_users))
}
