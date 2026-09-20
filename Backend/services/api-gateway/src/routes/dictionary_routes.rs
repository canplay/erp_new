//! 字典管理路由 — 字典类型/项 CRUD

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

async fn get_user_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::UserGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("user-service 不可用: {e}")))
}

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

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
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
}
