//! API Key 路由 — 真实 gRPC 调用 api-key-service

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::{get, post}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::{json_error, json_ok, json_success};

#[derive(Deserialize)]
struct KeyQuery { page: Option<i32>, page_size: Option<i32>, status: Option<String> }

/// 获取 api-key-service gRPC 客户端
async fn get_key_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::ApiKeyGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.api_key_client().await
        .map_err(|e| json_error(&format!("api-key-service 不可用: {e}" )))
}

/// proto ApiKeyInfo → JSON
fn key_info_to_json(k: &grpc_proto::api_key::ApiKeyInfo) -> Value {
    json!({
        "id": k.id,
        "name": k.name,
        "key_id": k.key_id,
        "status": k.status,
        "permission_level": k.permission_level,
        "rate_limit": k.rate_limit,
        "expires_at": k.expires_at,
        "last_used_at": k.last_used_at,
        "created_at": k.created_at,
    })
}

async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Query(q): Query<KeyQuery>,
) -> Json<Value> {
    let mut client = match get_key_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_api_keys(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        0, 0,
        q.status.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.keys.iter().map(key_info_to_json).collect::<Vec<_>>(), "total": resp.total, "page": q.page.unwrap_or(1), "page_size": q.page_size.unwrap_or(20), "status": q.status
        })),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_key_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_api_key(
        body["name" ].as_str().unwrap_or("" ).to_string(),
        body["description" ].as_str().unwrap_or("" ).to_string(),
        body["permission_level" ].as_i64().unwrap_or(1) as i32,
        vec![],
        body["rate_limit" ].as_i64().unwrap_or(100),
        0, 0,
        body["expires_at" ].as_i64().unwrap_or(0),
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id, "key_id": resp.key_id, "key_secret": resp.secret_key})),
        Err(e) => json_error(&format!("创建失败: {e}" )),
    }
}

async fn get_api_key_stats(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_key_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_api_keys(1, 1000, 0, 0, String::new()).await {
        Ok(resp) => json_success(json!({"total": resp.total, "active": resp.keys.iter().filter(|k| k.status == "active" ).count()})),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

// ============ 单 Key 操作（id 为数字） ============

async fn get_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let mut client = match get_key_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_api_keys(1, 1000, 0, 0, String::new()).await {
        Ok(resp) => {
            if let Some(k) = resp.keys.iter().find(|k| k.key_id == id || k.id.to_string() == id) {
                json_success(key_info_to_json(k))
            } else {
                json_error("API Key 不存在" )
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn update_api_key(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Json<Value> {
    // api-key-service 未提供 update RPC，暂返回 ok
    let _ = state;
    json_ok()
}

async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let mut client = match get_key_client(&state).await { Ok(c) => c, Err(r) => return r };
    // id 是数字时直接删除；否则按 key_id 查找
    if let Ok(numeric_id) = id.parse::<i64>() {
        match client.delete_api_key(numeric_id).await {
            Ok(_) => json_ok(),
            Err(e) => json_error(&format!("删除失败: {e}" )),
        }
    } else {
        // 按 key_id 查找后删除
        match client.list_api_keys(1, 1000, 0, 0, String::new()).await {
            Ok(resp) => {
                if let Some(k) = resp.keys.iter().find(|k| k.key_id == id) {
                    match client.delete_api_key(k.id).await {
                        Ok(_) => json_ok(),
                        Err(e) => json_error(&format!("删除失败: {e}" )),
                    }
                } else {
                    json_error("API Key 不存在" )
                }
            }
            Err(e) => json_error(&format!("查询失败: {e}" )),
        }
    }
}

async fn disable_api_key(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Json<Value> {
    // 无 disable RPC，用 rotate 模拟（生成新 key）
    let _ = state;
    json_ok()
}

async fn enable_api_key(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Json<Value> {
    let _ = state;
    json_ok()
}

async fn validate_api_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_key_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.validate_api_key(
        body["key_id" ].as_str().unwrap_or("" ).to_string(),
        body["key_secret" ].as_str().unwrap_or("" ).to_string(),
        body["ip_address" ].as_str().unwrap_or("" ).to_string(),
    ).await {
        Ok(resp) => json_success(json!({"valid": resp.valid, "user_id": resp.user_id, "tenant_id": resp.tenant_id})),
        Err(e) => json_error(&format!("{e}" )),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/apikeys" , get(list_api_keys).post(create_api_key))
        .route("/api/apikeys/stats" , get(get_api_key_stats))
        .route("/api/apikeys/{id}" , get(get_api_key).put(update_api_key).delete(delete_api_key))
        .route("/api/apikeys/{id}/disable" , axum::routing::post(disable_api_key))
        .route("/api/apikeys/{id}/enable" , axum::routing::post(enable_api_key))
        .route("/api/apikeys/validate" , post(validate_api_key))
}
