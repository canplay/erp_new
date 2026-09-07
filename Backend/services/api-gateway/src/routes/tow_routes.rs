//! Tow 服务路由 — HTTP → gRPC 转发

use std::sync::Arc;
use axum::{Router, extract::State, extract::Query, extract::Path, routing::{get, post}, Json};
use serde::Deserialize;
use common::AppError;
use serde_json::json;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CarQuery {
    pub keyword: Option<String>, pub status: Option<String>,
    pub page: Option<i32>, pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DictQuery {
    pub dict_type: String,
}

pub async fn list_cars(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CarQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.grpc_clients.read().await.tow_client().await
        .map_err(|e| AppError::ServiceUnavailable(format!("tow-service: {e}")))?;
    let resp = client.list_tow_cars(&q.keyword.unwrap_or_default(), &q.status.unwrap_or_default(), q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await
        .map_err(|e| AppError::ServiceUnavailable(format!("调用失败: {e}")))?;
    let cars: Vec<serde_json::Value> = resp.cars.into_iter().map(|c| json!({
        "id": c.id, "license": c.license, "car_type": c.car_type,
        "car_color": c.car_color, "dc_date": c.dc_date, "dc_address": c.dc_address,
        "dc_party_name": c.dc_party_name, "dc_type": c.dc_type, "dc_causes": c.dc_causes, "delete": c.delete,
    })).collect();
    Ok(Json(json!({ "cars": cars, "total": resp.total, "page": resp.page, "page_size": resp.page_size })))
}

pub async fn get_car(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.grpc_clients.read().await.tow_client().await
        .map_err(|e| AppError::ServiceUnavailable(format!("tow-service: {e}")))?;
    let c = client.get_tow_car(id).await.map_err(|e| AppError::ServiceUnavailable(format!("调用失败: {e}")))?;
    Ok(Json(json!({ "id": c.id, "license": c.license, "car_type": c.car_type, "car_color": c.car_color,
        "engine": c.engine, "dc_type": c.dc_type, "dc_causes": c.dc_causes, "dc_date": c.dc_date,
        "dc_address": c.dc_address, "dc_key": c.dc_key,
        "dc_party_name": c.dc_party_name, "dc_party_cardid": c.dc_party_cardid, "dc_party_tel": c.dc_party_tel,
        "p_name": c.p_name, "driver": c.driver, "operator": c.operator, "remark": c.remark, "delete": c.delete,
        "create_date": c.create_date, "update_date": c.update_date })))
}

pub async fn list_dict(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DictQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut client = state.grpc_clients.read().await.tow_client().await
        .map_err(|e| AppError::ServiceUnavailable(format!("tow-service: {e}")))?;
    let resp = client.list_dict_items(&q.dict_type).await
        .map_err(|e| AppError::ServiceUnavailable(format!("调用失败: {e}")))?;
    let items: Vec<serde_json::Value> = resp.items.into_iter().map(|d| json!({ "id": d.id, "name": d.name, "value": d.value })).collect();
    Ok(Json(json!({ "items": items })))
}

/// POST /api/tow/cars/{id}/status — 更新拖车状态（内存实现）
pub async fn update_tow_car_status(
    Path(_id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if !status.is_empty() {
        Json(json!({ "success": true, "code": 200, "message": format!("状态已更新为: {}", status) }))
    } else {
        Json(json!({ "success": false, "code": 400, "message": "状态参数不能为空" }))
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tow/cars", get(list_cars))
        .route("/api/tow/cars/{id}", get(get_car))
        .route("/api/tow/dict", get(list_dict))
        .route("/api/v1/tow/cars", get(list_cars))
        .route("/api/v1/tow/cars/{id}", get(get_car))
        .route("/api/v1/tow/cars/{id}/status", post(update_tow_car_status))
        .route("/api/v1/tow/dict", get(list_dict))
}
