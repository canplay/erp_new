//! 部门管理路由 — 部门 CRUD、树形结构

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, put},
    Json,
};
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;


async fn get_user_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::UserGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("user-service 不可用: {e}")))
}

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

async fn get_department_users(Path(_id): Path<i64>) -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn move_department_down(Path(_id): Path<i64>) -> Json<Value> { json_ok() }

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/departments", get(list_departments_handler).post(create_department_handler))
        .route("/api/admin/departments/tree", get(get_department_tree_handler))
        .route("/api/admin/departments/{id}", get(get_department_handler).put(update_department_handler).delete(delete_department_handler))
        .route("/api/admin/departments/{id}/move", put(move_department_down))
        .route("/api/admin/departments/{id}/users", get(get_department_users))
}
