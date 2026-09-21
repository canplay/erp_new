//! 系统配置路由 — 系统参数 + 业务选项
//!
//! 合并自 system_config_routes.rs + options_routes.rs

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State, Extension},
    routing::{get, put, post},
    Json,
};
use serde_json::{json, Value};

use crate::AppState;
use crate::middleware::JwtClaims;
use crate::routes::helpers::*;

// ==================== 系统配置 ====================

async fn list_system_configs_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Query(q): Query<PageQuery>,
) -> Result<Json<Value>, Json<Value>> {
    if claims.role != "admin" {
        return Ok(json_error("仅管理员可查询系统配置"));
    }

    let mut client = state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("用户服务不可用: {e}")))?;

    let category = q.keyword.unwrap_or_default();
    let resp = client.list_system_configs(category).await
        .map_err(|e| json_error(&format!("查询系统配置失败: {e}")))?;

    let configs: Vec<Value> = resp.configs.iter().map(|c| json!({
        "id": c.id, "category": c.category, "key": c.key, "value": c.value,
        "type": c.r#type, "label": c.label, "description": c.description,
        "sort": c.sort, "status": c.status,
    })).collect();
    Ok(json_success(json!(configs)))
}

async fn batch_update_system_configs_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Json<Value>> {
    if claims.role != "admin" {
        return Ok(json_error("仅管理员可修改系统配置"));
    }

    let configs = body.as_array()
        .ok_or_else(|| json_error("请求体必须为配置数组"))?
        .iter()
        .filter_map(|c| {
            let key = c.get("key")?.as_str()?.to_string();
            let value = c.get("value").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            Some(grpc_proto::user::UpdateSystemConfigRequest { key, value })
        })
        .collect::<Vec<_>>();
    if configs.is_empty() {
        return Ok(json_error("配置数组不能为空"));
    }

    let mut client = state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("用户服务不可用: {e}")))?;

    let resp = client.batch_update_system_configs(configs).await
        .map_err(|e| json_error(&format!("批量更新系统配置失败: {e}")))?;

    Ok(json_success(json!({"success": resp.success})))
}

async fn update_system_config_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(key): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Json<Value>> {
    if claims.role != "admin" {
        return Ok(json_error("仅管理员可修改系统配置"));
    }

    let value = body.get("value").and_then(|v| v.as_str()).unwrap_or_default().to_string();

    let mut client = state.grpc_clients.read().await.user_client().await
        .map_err(|e| json_error(&format!("用户服务不可用: {e}")))?;

    let resp = client.update_system_config(key.clone(), value).await
        .map_err(|e| json_error(&format!("更新系统配置失败: {e}")))?;

    Ok(json_success(json!({"key": key, "success": resp.success})))
}

// ==================== 业务选项 ====================

pub async fn options_query(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<Value>, Json<Value>> {
    if claims.role != "admin" {
        return Ok(json_error("仅管理员可查询系统配置" ));
    }

    let clients = state.grpc_clients.read().await;
    let mut client = clients
        .ebike_client()
        .await
        .map_err(|e| json_error(&format!("ebike-service 不可用：{e}" )))?;

    let resp = client.options_query().await.map_err(|e| json_error(&format!("查询失败：{e}" )))?;

    Ok(json_success(json![
        {
            "name": resp.name,
            "options": serde_json::from_str::<Value>(&resp.options_json).unwrap_or_else(|_| json!({})),
            "level": resp.level,
            "create_date": resp.create_date,
            "update_date": resp.update_date,
            "delete": resp.delete,
        }
    ]))
}

pub async fn options_update(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Json<Value>> {
    if claims.role != "admin" {
        return Ok(json_error("仅管理员可修改系统配置" ));
    }

    let clients = state.grpc_clients.read().await;
    let mut client = clients
        .ebike_client()
        .await
        .map_err(|e| json_error(&format!("ebike-service 不可用：{e}" )))?;

    let req = grpc_proto::ebike::OptionsUpdateRequest {
        name: body["name"].as_str().unwrap_or("").to_string(),
        system: body["system"].as_i64().unwrap_or(0),
        alert: body["alert"].as_i64().unwrap_or(0),
        level: body["level"].as_i64().unwrap_or(0),
    };

    match client.options_update(req).await {
        Ok(resp) => Ok(json_success(json!({"code": resp.code, "message": resp.message}))),
        Err(e) => Ok(json_error(&format!("更新失败：{e}" ))),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 系统配置
        .route("/api/config/system-configs", get(list_system_configs_handler))
        .route("/api/config/system-configs/batch", put(batch_update_system_configs_handler))
        .route("/api/config/system-configs/{key}", put(update_system_config_handler))
        // 业务选项
        .route("/options/query" , get(options_query))
        .route("/options/update" , post(options_update))
}

#[derive(Debug, serde::Deserialize)]
struct PageQuery {
    keyword: Option<String>,
}
