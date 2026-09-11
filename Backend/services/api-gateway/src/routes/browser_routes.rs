//! Browser 服务路由 — gRPC 调用

use std::sync::Arc;
use axum::{Router, Json, extract::State, routing::post};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::BrowserGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.browser_client().await
        .map_err(|e| json_error(&format!("browser-service 不可用: {e}")))
}

async fn open_page(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.open_page().await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("打开页面失败: {e}")),
    }
}

async fn navigate(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.navigate(
        body["session_id"].as_str().unwrap_or("").to_string(),
        body["url"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("导航失败: {e}")),
    }
}

async fn get_text(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_text(body["session_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("获取文本失败: {e}")),
    }
}

async fn get_html(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_html(body["session_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("获取HTML失败: {e}")),
    }
}

async fn screenshot(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.screenshot(
        body["session_id"].as_str().unwrap_or("").to_string(),
        body["full_page"].as_bool().unwrap_or(false),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("截图失败: {e}")),
    }
}

async fn close(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.close_page(body["session_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("关闭失败: {e}")),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/browser/open", post(open_page))
        .route("/api/v1/browser/navigate", post(navigate))
        .route("/api/v1/browser/text", post(get_text))
        .route("/api/v1/browser/html", post(get_html))
        .route("/api/v1/browser/screenshot", post(screenshot))
        .route("/api/v1/browser/close", post(close))
}
