//! 设备集成路由 — Hik/CTP/Browser 设备服务 gRPC 调用
//!
//! 合并自 hik_routes.rs + ctp_routes.rs + browser_routes.rs

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Path}, routing::{get, post}};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

// ==================== Hik ====================

async fn get_hik_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::HikGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.hik_client().await
        .map_err(|e| json_error(&format!("hik-service 不可用: {e}" )))
}

async fn hik_exec(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_hik_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.exec(
        body["method"].as_str().unwrap_or("").to_string(),
        body["phone"].as_i64(),
        body["driver_id"].as_str().map(|s| s.to_string()),
        body["plate_no"].as_str().map(|s| s.to_string()),
        body["plate_color"].as_str().map(|s| s.to_string()),
        body["page_no"].as_i64().map(|v| v as i32),
        body["page_size"].as_i64().map(|v| v as i32),
        body["park_name"].as_str().map(|s| s.to_string()),
        body["park_code"].as_str().map(|s| s.to_string()),
        body["request_type"].as_str().map(|s| s.to_string()),
        body["unique_id"].as_str().map(|s| s.to_string()),
        body["is_use_coupon"].as_str().map(|s| s.to_string()),
        body["appeal_type"].as_str().map(|s| s.to_string()),
        body["appeal_remark"].as_str().map(|s| s.to_string()),
        body["appeal_in_time"].as_str().map(|s| s.to_string()),
        body["appeal_out_time"].as_str().map(|s| s.to_string()),
        body["appeal_source"].as_str().map(|s| s.to_string()),
        body["arrears_ids"].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message, "data": resp.data})),
        Err(e) => json_error(&format!("hik exec 失败: {e}" )),
    }
}

async fn hik_coupon(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_hik_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.coupon(
        body["phone"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0) as i32,
        body["start"].as_i64().unwrap_or(0),
        body["end"].as_str().unwrap_or("").to_string(),
        body["type"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message})),
        Err(e) => json_error(&format!("coupon 失败: {e}" )),
    }
}

async fn hik_signo_open(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_hik_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.signo_open(
        body["place"].as_str().unwrap_or("").to_string(),
        body["name"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!({"message": resp.message, "status": resp.status})),
        Err(e) => json_error(&format!("signo_open 失败: {e}" )),
    }
}

// ==================== CTP ====================

async fn get_ctp_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::CtpGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.ctp_client().await
        .map_err(|e| json_error(&format!("ctp-service 不可用: {e}" )))
}

async fn ctp_receive_data(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_ctp_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.receive_device_data(
        body["device_no"].as_str().unwrap_or("").to_string(),
        body["data_type"].as_i64().unwrap_or(0) as i32,
        body["voltage"].as_str().map(|s| s.to_string()),
        body["status_one"].as_str().map(|s| s.to_string()),
        body["status_two"].as_str().map(|s| s.to_string()),
        body["data_time"].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => json_success(json!({"error_code": resp.error_code, "error_msg": resp.error_msg})),
        Err(e) => json_error(&format!("上报失败: {e}" )),
    }
}

async fn ctp_control(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_ctp_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.control_lock(
        body["factory_id"].as_str().unwrap_or("").to_string(),
        body["device_no"].as_str().unwrap_or("").to_string(),
        body["cmd_type"].as_str().unwrap_or("").to_string(),
        body["data"].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => json_success(json!({"success": resp.success, "message": resp.message})),
        Err(e) => json_error(&format!("控制失败: {e}" )),
    }
}

async fn ctp_get_device_info(State(state): State<Arc<AppState>>, Path(device_no): Path<String>) -> Json<Value> {
    let mut client = match get_ctp_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_device(device_no).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn ctp_list_devices(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_ctp_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_devices(
        body["park_code"].as_str().map(|s| s.to_string()),
        body["status"].as_str().map(|s| s.to_string()),
        body["page"].as_i64().unwrap_or(1) as i32,
        body["page_size"].as_i64().unwrap_or(20) as i32,
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

// ==================== Browser ====================

async fn get_browser_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::BrowserGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.browser_client().await
        .map_err(|e| json_error(&format!("browser-service 不可用: {e}" )))
}

async fn browser_open_page(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut client = match get_browser_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.open_page().await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("打开页面失败: {e}" )),
    }
}

async fn browser_navigate(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_browser_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.navigate(
        body["session_id"].as_str().unwrap_or("").to_string(),
        body["url"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("导航失败: {e}" )),
    }
}

async fn browser_get_text(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_browser_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_text(body["session_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("获取文本失败: {e}" )),
    }
}

async fn browser_get_html(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_browser_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_html(body["session_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("获取HTML失败: {e}" )),
    }
}

async fn browser_screenshot(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_browser_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.screenshot(
        body["session_id"].as_str().unwrap_or("").to_string(),
        body["full_page"].as_bool().unwrap_or(false),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("截图失败: {e}" )),
    }
}

async fn browser_close(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_browser_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.close_page(body["session_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("关闭失败: {e}" )),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Hik
        .route("/api/v1/hik/exec" , post(hik_exec))
        .route("/api/v1/hik/coupon" , post(hik_coupon))
        .route("/api/v1/hik/signo/open" , post(hik_signo_open))
        // CTP
        .route("/api/v1/ctp/report" , post(ctp_receive_data))
        .route("/api/v1/ctp/device/control" , post(ctp_control))
        .route("/api/v1/ctp/device/{device_no}" , get(ctp_get_device_info))
        .route("/api/v1/ctp/device/list" , post(ctp_list_devices))
        // Browser
        .route("/api/v1/browser/open" , post(browser_open_page))
        .route("/api/v1/browser/navigate" , post(browser_navigate))
        .route("/api/v1/browser/text" , post(browser_get_text))
        .route("/api/v1/browser/html" , post(browser_get_html))
        .route("/api/v1/browser/screenshot" , post(browser_screenshot))
        .route("/api/v1/browser/close" , post(browser_close))
}
