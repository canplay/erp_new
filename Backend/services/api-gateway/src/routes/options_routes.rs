//! Options 配置路由 — 仅管理员可查询/修改系统配置

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Extension}, routing::{get, post}};
use serde_json::{json, Value};

use crate::AppState;
use crate::middleware::JwtClaims;
use grpc_proto::ebike;
use crate::routes::helpers::*;


pub async fn options_query(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<Value>, Json<Value>> {
    if claims.role != "admin" {
        return Ok(json_error("仅管理员可查询系统配置"));
    }

    let clients = state.grpc_clients.read().await;
    let mut client = clients
        .ebike_client()
        .await
        .map_err(|e| json_error(&format!("ebike-service 不可用：{e}")))?;

    let resp = client.options_query().await.map_err(|e| json_error(&format!("查询失败：{e}")))?;

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
        return Ok(json_error("仅管理员可修改系统配置"));
    }

    let clients = state.grpc_clients.read().await;
    let mut client = clients
        .ebike_client()
        .await
        .map_err(|e| json_error(&format!("ebike-service 不可用：{e}")))?;

    let req = ebike::OptionsUpdateRequest {
        name: body["name"].as_str().unwrap_or("").to_string(),
        system: body["system"].as_i64().unwrap_or(0),
        alert: body["alert"].as_i64().unwrap_or(0),
        level: body["level"].as_i64().unwrap_or(0),
    };

    match client.options_update(req).await {
        Ok(resp) => Ok(Json(json!({"success": true, "code": resp.code, "message": resp.message}))),
        Err(e) => Ok(json_error(&format!("更新失败：{e}"))),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/options/query", get(options_query))
        .route("/options/update", post(options_update))
}
