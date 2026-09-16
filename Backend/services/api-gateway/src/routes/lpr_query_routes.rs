//! LPR 查询路由 — gRPC 调用

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Query}, routing::get};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::LprGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.lpr_client().await
        .map_err(|e| json_error(&format!("lpr-service 不可用: {e}" )))
}

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
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_pass_records(
        q.plate_no.as_deref().unwrap_or("" ),
        q.park_code.as_deref().unwrap_or("" ),
        q.direction.as_deref().unwrap_or("" ),
        q.status.as_deref().unwrap_or("" ),
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn get_stats(State(state): State<Arc<AppState>>, Query(q): Query<RecordQuery>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_pass_stats(
        q.park_code.as_deref().unwrap_or("" ),
        "" , "" ,
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

// ============ LPR 设备管理 (内存实现，待接入 gRPC) ============

async fn list_lpr_devices() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn get_lpr_device(axum::extract::Path(id): axum::extract::Path<String>) -> Json<Value> {
    json_success(json!({"id": id}))
}

/// GET /api/lpr/records/{id} — LPR记录详情
async fn get_lpr_record_detail(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_pass_record(id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn get_vehicle_auth(Query(_q): Query<RecordQuery>) -> Json<Value> {
    json_success(json!({
            "is_authorized": false,
            "auth_type": " ",
            "driver_name": " ",
            "driver_phone": " ",
            "valid_until": null
        }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/lpr/records" , get(list_records))
        .route("/api/v1/lpr/stats" , get(get_stats))
        .route("/api/v1/lpr/devices" , get(list_lpr_devices))
        .route("/api/v1/lpr/devices/{id}" , get(get_lpr_device))
        .route("/api/lpr/records/{id}" , get(get_lpr_record_detail))
        .route("/api/lpr/vehicle/auth" , get(get_vehicle_auth))
}
