//! XLT 停车服务路由 — gRPC 调用

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Query}, routing::{get, post}};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::XltGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.xlt_client().await
        .map_err(|e| json_error(&format!("xlt-service 不可用: {e}")))
}

async fn vehicle_entry(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.vehicle_entry(
        body["plate_no"].as_str().unwrap_or("").to_string(),
        body["plate_color"].as_str().unwrap_or("").to_string(),
        body["park_code"].as_str().unwrap_or("").to_string(),
        body["lane_code"].as_str().unwrap_or("").to_string(),
        body["event_time"].as_str().unwrap_or("").to_string(),
        body["vehicle_type"].as_str().unwrap_or("").to_string(),
        body["image_url"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0),
        body["pay_type"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("入场登记失败: {e}")),
    }
}

async fn vehicle_exit(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.vehicle_exit(
        body["plate_no"].as_str().unwrap_or("").to_string(),
        body["plate_color"].as_str().unwrap_or("").to_string(),
        body["park_code"].as_str().unwrap_or("").to_string(),
        body["lane_code"].as_str().unwrap_or("").to_string(),
        body["event_time"].as_str().unwrap_or("").to_string(),
        body["vehicle_type"].as_str().unwrap_or("").to_string(),
        body["image_url"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0),
        body["pay_type"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("出场登记失败: {e}")),
    }
}

#[derive(Deserialize)]
struct VehicleQuery {
    plate_no: Option<String>,
    park_code: Option<String>,
    status: Option<String>,
    page: Option<i32>,
    page_size: Option<i32>,
}

async fn list_records(State(state): State<Arc<AppState>>, Query(q): Query<VehicleQuery>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_records(
        q.plate_no, q.park_code, q.status, q.page.unwrap_or(1), q.page_size.unwrap_or(20),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn list_devices(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_devices().await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn calc_billing(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.calc_billing(
        body["plate_no"].as_str().unwrap_or("").to_string(),
        body["park_code"].as_str().unwrap_or("").to_string(),
        body["entry_time"].as_str().unwrap_or("").to_string(),
        body["exit_time"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("计费失败: {e}")),
    }
}

async fn get_parking_vehicle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((park_code, plate_no)): axum::extract::Path<(String, String)>,
) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_parking_vehicle(park_code, plate_no).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn open_barrier(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    let sn = body["sn"].as_str().unwrap_or("").to_string();
    let request_id = format!("xlt-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
    match client.open_barrier(sn, request_id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("开闸失败: {e}")),
    }
}

async fn close_barrier(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    let sn = body["sn"].as_str().unwrap_or("").to_string();
    let request_id = format!("xlt-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
    match client.close_barrier(sn, request_id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("关闸失败: {e}")),
    }
}

/// POST /api/xlt/vehicle/search — 车辆搜索（与 list_records 相同逻辑，但为 POST）
async fn search_vehicle(State(state): State<Arc<AppState>>, Query(q): Query<VehicleQuery>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_records(
        q.plate_no, q.park_code, q.status, q.page.unwrap_or(1), q.page_size.unwrap_or(20),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/xlt/parking/entry", post(vehicle_entry))
        .route("/api/v1/xlt/parking/exit", post(vehicle_exit))
        .route("/api/v1/xlt/parking/billing", post(calc_billing))
        .route("/api/v1/xlt/parking/records", get(list_records))
        .route("/api/v1/xlt/parking/vehicle/{parkCode}/{plateNo}", get(get_parking_vehicle))
        .route("/api/v1/xlt/device/list", get(list_devices))
        .route("/api/v1/xlt/device/open", post(open_barrier))
        .route("/api/v1/xlt/device/close", post(close_barrier))
        // 车辆搜索
        .route("/api/xlt/vehicle/search", post(search_vehicle))
}
