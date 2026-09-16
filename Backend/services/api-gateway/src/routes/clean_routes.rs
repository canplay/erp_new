//! Clean 服务路由 — gRPC 实现
//!
//! 提供环卫清运业务中发票、订单、账单和支付记录的查询接口
//!
//! @date 2026-07-17

use std::sync::Arc;
use axum::{
    Router,
    extract::{Query, State},
    routing::{get, post},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;

// ============ 查询参数 ============

#[derive(Debug, Deserialize)]
pub struct InvoiceQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub no: Option<String>,
    pub imposing_no: Option<i32>,
    pub imposing_name: Option<String>,
    pub collection_name: Option<String>,
    pub fingerprint: Option<String>,
    pub zone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub serial_number: Option<String>,
    pub numbering: Option<String>,
    pub cashier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BillQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub numbering: Option<String>,
    pub fzr: Option<String>,
    pub status: Option<String>,
    pub create_date_start: Option<String>,
    pub create_date_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub numbering: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub create_date_start: Option<String>,
    pub create_date_end: Option<String>,
    pub payment_date_start: Option<String>,
    pub payment_date_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub statistics_type: Option<String>,
}

// ============ gRPC Handlers ============

/// GET /api/clean/invoices — 分页查询发票列表
async fn list_invoices_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<InvoiceQuery>,
) -> Json<Value> {
    let mut client = match state.grpc_clients.read().await.clean_client().await {
        Ok(c) => c,
        Err(e) => return json_error_fmt("clean-service 不可用" , &e),
    };
    match client.list_invoices(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.no.clone(),
        q.imposing_no,
        q.imposing_name.clone(),
        q.collection_name.clone(),
        q.fingerprint.clone(),
        q.zone.clone(),
    ).await {
        Ok(resp) => json_success(json!({"list": resp.invoices, "total": resp.total})),
        Err(e) => json_error_fmt("查询发票列表失败" , &e),
    }
}

/// GET /api/clean/orders — 分页查询订单列表
async fn list_orders_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<OrderQuery>,
) -> Json<Value> {
    let mut client = match state.grpc_clients.read().await.clean_client().await {
        Ok(c) => c,
        Err(e) => return json_error_fmt("clean-service 不可用" , &e),
    };
    match client.list_orders(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.serial_number.clone(),
        q.numbering.clone(),
        q.cashier.clone(),
    ).await {
        Ok(resp) => json_success(json!({"list": resp.orders, "total": resp.total})),
        Err(e) => json_error_fmt("查询订单列表失败" , &e),
    }
}

/// GET /api/clean/bills — 分页查询正式账单
async fn list_formal_bills_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<BillQuery>,
) -> Json<Value> {
    let mut client = match state.grpc_clients.read().await.clean_client().await {
        Ok(c) => c,
        Err(e) => return json_error_fmt("clean-service 不可用" , &e),
    };
    match client.list_formal_bills(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.numbering.clone(),
        q.fzr.clone(),
        q.status.clone(),
        q.create_date_start.clone(),
        q.create_date_end.clone(),
    ).await {
        Ok(resp) => json_success(json!({"list": resp.bills, "total": resp.total})),
        Err(e) => json_error_fmt("查询正式账单失败" , &e),
    }
}

/// GET /api/clean/payments — 分页查询网络支付记录
async fn list_payment_web_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PaymentQuery>,
) -> Json<Value> {
    let mut client = match state.grpc_clients.read().await.clean_client().await {
        Ok(c) => c,
        Err(e) => return json_error_fmt("clean-service 不可用" , &e),
    };
    match client.list_payment_webs(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.numbering.clone(),
        q.r#type.clone(),
        q.status.clone(),
        q.create_date_start.clone(),
        q.create_date_end.clone(),
        q.payment_date_start.clone(),
        q.payment_date_end.clone(),
    ).await {
        Ok(resp) => json_success(json!({"list": resp.payments, "total": resp.total})),
        Err(e) => json_error_fmt("查询支付记录失败" , &e),
    }
}

/// GET /api/clean/stats — 获取统计数据
async fn get_statistics_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<StatsQuery>,
) -> Json<Value> {
    let mut client = match state.grpc_clients.read().await.clean_client().await {
        Ok(c) => c,
        Err(e) => return json_error_fmt("clean-service 不可用" , &e),
    };
    match client.get_statistics(q.statistics_type.unwrap_or_default()).await {
        Ok(resp) => json_success(json!({
            "total": resp.total,
            "paid": resp.paid,
            "unpaid": resp.unpaid,
        })),
        Err(e) => json_error_fmt("获取统计数据失败" , &e),
    }
}

// ============ 明细查询 Handler ============

async fn clean_invoice_count() -> Json<Value> {
    json_success(json!({"count": 0}))
}
async fn clean_invoice_info() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn clean_invoice_update() -> Json<Value> { json_ok() }
async fn clean_order_count() -> Json<Value> {
    json_success(json!({"count": 0}))
}
async fn clean_order_info() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn clean_staff_info() -> Json<Value> { json_ok() }
async fn clean_formal_count() -> Json<Value> {
    json_success(json!({"count": 0}))
}
async fn clean_formal_info() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn clean_formal_total() -> Json<Value> {
    json_success(json!({"total": 0}))
}
async fn clean_payment_info_count() -> Json<Value> {
    json_success(json!({"count": 0}))
}
async fn clean_payment_info_info() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn clean_payment_info_total() -> Json<Value> {
    json_success(json!({"total": 0}))
}
async fn clean_payment_web_count() -> Json<Value> {
    json_success(json!({"count": 0}))
}
async fn clean_payment_web_info() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn clean_payment_web_total() -> Json<Value> {
    json_success(json!({"total": 0}))
}

// ============ 路由定义 ============

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/clean/invoices" , get(list_invoices_handler))
        .route("/api/clean/orders" , get(list_orders_handler))
        .route("/api/clean/bills" , get(list_formal_bills_handler))
        .route("/api/clean/payments" , get(list_payment_web_handler))
        .route("/api/clean/stats" , get(get_statistics_handler))
        .route("/api/clean/invoice/count" , get(clean_invoice_count))
        .route("/api/clean/invoice/info" , get(clean_invoice_info))
        .route("/api/clean/invoice/update" , post(clean_invoice_update))
        .route("/api/clean/order/count" , get(clean_order_count))
        .route("/api/clean/order/info" , get(clean_order_info))
        .route("/api/clean/staff/info" , post(clean_staff_info))
        .route("/api/clean/formal/count" , get(clean_formal_count))
        .route("/api/clean/formal/info" , get(clean_formal_info))
        .route("/api/clean/formal/total" , get(clean_formal_total))
        .route("/api/clean/payment/info/count" , get(clean_payment_info_count))
        .route("/api/clean/payment/info/info" , get(clean_payment_info_info))
        .route("/api/clean/payment/info/total" , get(clean_payment_info_total))
        .route("/api/clean/payment/web/count" , get(clean_payment_web_count))
        .route("/api/clean/payment/web/info" , get(clean_payment_web_info))
        .route("/api/clean/payment/web/total" , get(clean_payment_web_total))
}
