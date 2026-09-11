//! CTP 服务路由 — gRPC 调用（厂商回调由 gateway 统一接收）

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Path}, routing::{get, post}};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::CtpGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.ctp_client().await
        .map_err(|e| json_error(&format!("ctp-service 不可用: {e}")))
}

async fn receive_data(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.receive_device_data(
        body["device_no"].as_str().unwrap_or("").to_string(),
        body["data_type"].as_i64().unwrap_or(0) as i32,
        body["voltage"].as_str().map(|s| s.to_string()),
        body["status_one"].as_str().map(|s| s.to_string()),
        body["status_two"].as_str().map(|s| s.to_string()),
        body["data_time"].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => Json(json!({"success": true, "error_code": resp.error_code, "error_msg": resp.error_msg})),
        Err(e) => json_error(&format!("上报失败: {e}")),
    }
}

async fn control(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.control_lock(
        body["factory_id"].as_str().unwrap_or("").to_string(),
        body["device_no"].as_str().unwrap_or("").to_string(),
        body["cmd_type"].as_str().unwrap_or("").to_string(),
        body["data"].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => Json(json!({"success": resp.success, "message": resp.message})),
        Err(e) => json_error(&format!("控制失败: {e}")),
    }
}

async fn get_device_info(State(state): State<Arc<AppState>>, Path(device_no): Path<String>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_device(device_no).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn list_devices(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_devices(
        body["park_code"].as_str().map(|s| s.to_string()),
        body["status"].as_str().map(|s| s.to_string()),
        body["page"].as_i64().unwrap_or(1) as i32,
        body["page_size"].as_i64().unwrap_or(20) as i32,
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/ctp/report", post(receive_data))
        .route("/api/v1/ctp/device/control", post(control))
        .route("/api/v1/ctp/device/{device_no}", get(get_device_info))
        .route("/api/v1/ctp/device/list", post(list_devices))
}
