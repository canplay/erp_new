//! LPR 服务路由 — 车牌识别回调 + 查询
//!
//! 合并自 lpr_routes.rs + lpr_query_routes.rs

use std::sync::Arc;
use axum::{Router, extract::{State, Query}, routing::{get, post}, Json};
use serde::Deserialize;
use common::AppError;
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::{json_error, json_lpr_response, json_success};

// ==================== LPR 回调 ====================

/// 车牌识别回调请求体
#[derive(Debug, Deserialize)]
pub struct LprCallbackPayload {
    pub plate_no: Option<String>,
    pub plate_color: Option<String>,
    pub plate_type: Option<String>,
    pub vehicle_type: Option<String>,
    pub pass_time: Option<String>,
    pub image_url: Option<String>,
    pub device_id: Option<String>,
    pub park_code: Option<String>,
    pub direction: Option<String>,
    pub confidence: Option<f64>,
    pub device_name: Option<String>,
    pub lane_code: Option<String>,
}

async fn get_lpr_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::LprGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.lpr_client().await
        .map_err(|e| json_error(&format!("lpr-service 不可用: {e}")))
}

/// 车牌识别回调处理器
async fn lpr_callback_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LprCallbackPayload>,
) -> Result<Json<Value>, AppError> {
    let clients = state.grpc_clients.read().await;
    let mut client = clients.lpr_client().await.map_err(|e| {
        tracing::error!("lpr-service 不可用: {e}");
        AppError::ServiceUnavailable("lpr-service 不可用".into())
    })?;

    let result = client
        .lpr_callback(
            payload.plate_no.unwrap_or_default(),
            payload.plate_color.unwrap_or_default(),
            payload.plate_type.unwrap_or_default(),
            payload.vehicle_type.unwrap_or_default(),
            payload.pass_time.unwrap_or_default(),
            payload.image_url.unwrap_or_default(),
            payload.device_id.unwrap_or_default(),
            payload.park_code.unwrap_or_default(),
            payload.direction.unwrap_or_default(),
            payload.confidence.unwrap_or(0.0),
            payload.device_name.unwrap_or_default(),
            payload.lane_code.unwrap_or_default(),
        )
        .await;

    match result {
        Ok(response) => Ok(json_lpr_response(&response)),
        Err(e) => {
            tracing::error!("gRPC LPR 回调转发失败: {e}");
            Err(AppError::ServiceUnavailable(format!("lpr-service 调用失败: {e}")))
        }
    }
}

/// LPR 车辆授权请求
#[derive(Debug, Deserialize)]
pub struct VehicleAuthPayload {
    pub plate_no: Option<String>,
    pub park_code: Option<String>,
    pub device_id: Option<String>,
}

async fn vehicle_auth_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleAuthPayload>,
) -> Result<Json<Value>, AppError> {
    let clients = state.grpc_clients.read().await;
    let mut client = clients.lpr_client().await.map_err(|e| {
        tracing::error!("lpr-service 不可用: {e}");
        AppError::ServiceUnavailable("lpr-service 不可用".into())
    })?;

    let result = client
        .get_vehicle_auth(
            &payload.plate_no.unwrap_or_default(),
            &payload.park_code.unwrap_or_default(),
        )
        .await;

    match result {
        Ok(response) => Ok(json_success(json!({
            "is_authorized": response.is_authorized,
            "auth_type": response.auth_type,
            "driver_name": response.driver_name,
            "driver_phone": response.driver_phone,
            "valid_until": response.valid_until,
        }))),
        Err(e) => {
            tracing::error!("gRPC LPR 车辆授权转发失败: {e}");
            Ok(json_error(&format!("lpr-service 调用失败: {e}")))
        }
    }
}

// ==================== LPR 查询 ====================

#[derive(Deserialize)]
struct RecordQuery {
    plate_no: Option<String>,
    park_code: Option<String>,
    direction: Option<String>,
    status: Option<String>,
    page: Option<i32>,
    page_size: Option<i32>,
}

async fn list_records(State(state): State<Arc<AppState>>, Query(q): Query<RecordQuery>) -> Json<Value> {
    let mut client = match get_lpr_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_pass_records(
        q.plate_no.as_deref().unwrap_or(""),
        q.park_code.as_deref().unwrap_or(""),
        q.direction.as_deref().unwrap_or(""),
        q.status.as_deref().unwrap_or(""),
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_stats(State(state): State<Arc<AppState>>, Query(q): Query<RecordQuery>) -> Json<Value> {
    let mut client = match get_lpr_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_pass_stats(
        q.park_code.as_deref().unwrap_or(""),
        "", "",
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

// ============ LPR 设备管理 (内存实现，待接入 gRPC) ============

async fn list_lpr_devices() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn get_lpr_device(axum::extract::Path(id): axum::extract::Path<String>) -> Json<Value> {
    json_success(json!({"id": id}))
}

async fn get_lpr_record_detail(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Json<Value> {
    let mut client = match get_lpr_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_pass_record(id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_vehicle_auth_query(Query(_q): Query<RecordQuery>) -> Json<Value> {
    json_success(json!({
        "is_authorized": false,
        "auth_type": " ",
        "driver_name": " ",
        "driver_phone": " ",
        "valid_until": null
    }))
}

// ==================== 路由定义 ====================

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // LPR 回调
        .route("/api/v1/lpr/callback", post(lpr_callback_handler))
        .route("/api/lpr/callback", post(lpr_callback_handler))
        .route("/api/lpr/vehicle/auth", post(vehicle_auth_handler))
        // LPR 查询
        .route("/api/v1/lpr/records", get(list_records))
        .route("/api/v1/lpr/stats", get(get_stats))
        .route("/api/v1/lpr/devices", get(list_lpr_devices))
        .route("/api/v1/lpr/devices/{id}", get(get_lpr_device))
        .route("/api/lpr/records/{id}", get(get_lpr_record_detail))
        .route("/api/lpr/vehicle/auth", get(get_vehicle_auth_query))
}
