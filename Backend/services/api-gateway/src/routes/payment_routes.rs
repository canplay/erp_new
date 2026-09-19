//! 支付服务路由 — 计费计划、订阅、发票 + 支付网关 (CCB/UMS)
//!
//! 合并自 billing_routes.rs + pay_routes.rs

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Path}, routing::{get, post}};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

// ==================== 计费 ====================

async fn list_plans(State(_state): State<Arc<AppState>>) -> Json<Value> {
    json_success(serde_json::json!({"plans": []}))
}

async fn list_subscriptions(State(_state): State<Arc<AppState>>) -> Json<Value> {
    json_success(serde_json::json!({"subscriptions": []}))
}

async fn list_invoices(State(_state): State<Arc<AppState>>) -> Json<Value> {
    json_success(serde_json::json!({"invoices": []}))
}

// ==================== 支付客户端 ====================

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::PayGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.pay_client().await
        .map_err(|e| json_error(&format!("pay-service 不可用: {e}" )))
}

async fn pay_list(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list(
        body["status"].as_str().unwrap_or("").to_string(),
        body["pay_type"].as_str().unwrap_or("").to_string(),
        body["remark"].as_str().unwrap_or("").to_string(),
        body["sort_by"].as_str().unwrap_or("create_date").to_string(),
        body["descending"].as_bool().unwrap_or(true),
        body["page"].as_i64().unwrap_or(1),
        body["page_size"].as_i64().unwrap_or(20),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn pay_count(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.count(
        body["status"].as_str().unwrap_or("").to_string(),
        body["pay_type"].as_str().unwrap_or("").to_string(),
        body["remark"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn pay_latest(State(state): State<Arc<AppState>>, Path(user_id): Path<String>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.latest(user_id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn pay_create(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_order(
        body["order"].as_str().unwrap_or("").to_string(),
        body["status"].as_str().unwrap_or("").to_string(),
        body["pay_type"].as_str().unwrap_or("").to_string(),
        body["order_pay_json"].as_str().unwrap_or("{}").to_string(),
        body["amount"].as_i64().unwrap_or(0) as i32,
        body["remark"].as_str().unwrap_or("").to_string(),
        body["create_params_json"].as_str().unwrap_or("{}").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("创建失败: {e}" )),
    }
}

async fn ccb_query(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ccb_query(body["order_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn ccb_create(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ccb_create(
        body["order_id"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0) as i32,
        body["subject"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("创建失败: {e}" )),
    }
}

async fn ccb_verify(State(state): State<Arc<AppState>>, Path(order_id): Path<String>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ccb_verify(order_id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("验证失败: {e}" )),
    }
}

async fn ccb_refund(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ccb_refund(
        body["order_id"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("退款失败: {e}" )),
    }
}

async fn ums_query(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ums_query(body["order_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn ums_create(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ums_create(
        body["order_id"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0) as i32,
        body["subject"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("创建失败: {e}" )),
    }
}

async fn ums_close(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ums_close(body["order_id"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("关闭失败: {e}" )),
    }
}

async fn ums_refund(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ums_refund(
        body["order_id"].as_str().unwrap_or("").to_string(),
        body["amount"].as_i64().unwrap_or(0),
        body["reason"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("退款失败: {e}" )),
    }
}

async fn ums_info(State(state): State<Arc<AppState>>, Path(order_id): Path<String>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ums_info(order_id).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn ums_notify(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.ums_notify(
        body["order"].as_str().unwrap_or("").to_string(),
        body["time"].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => json_success(json!(resp)),
        Err(e) => json_error(&format!("回调处理失败: {e}" )),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 计费
        .route("/api/v1/billing/plans" , get(list_plans))
        .route("/api/v1/billing/subscriptions" , get(list_subscriptions))
        .route("/api/v1/billing/invoices" , get(list_invoices))
        // 支付列表
        .route("/api/pay/list" , post(pay_list))
        .route("/api/v1/pay/list" , post(pay_list))
        .route("/api/v1/pay/list/records" , post(pay_list))
        .route("/api/v1/pay/count" , post(pay_count))
        .route("/api/v1/pay/latest/{user_id}" , get(pay_latest))
        .route("/api/v1/pay/create" , post(pay_create))
        .route("/api/v1/pay/ccb/query" , post(ccb_query))
        .route("/api/v1/pay/ccb/create" , post(ccb_create))
        .route("/api/v1/pay/ccb/verify/{order_id}" , get(ccb_verify))
        .route("/api/v1/pay/ccb/refund" , post(ccb_refund))
        .route("/api/v1/pay/ums/query" , post(ums_query))
        .route("/api/v1/pay/ums/create" , post(ums_create))
        .route("/api/v1/pay/ums/close" , post(ums_close))
        .route("/api/v1/pay/ums/refund" , post(ums_refund))
        .route("/api/v1/pay/ums/info/{order_id}" , get(ums_info))
        .route("/api/v1/pay/ums/notify" , post(ums_notify))
}
