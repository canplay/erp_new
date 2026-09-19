//! E-Bike 管理路由 — 车辆 + 订单 + 停放区 gRPC 调用
//!
//! 合并自 ebike_car_routes.rs + ebike_order_routes.rs + ebike_storage_routes.rs

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Query, Extension}, routing::{get, post}};
use serde::Deserialize;
use serde_json::{json, Value};
use grpc_proto::ebike;

use crate::AppState;
use crate::middleware::JwtClaims;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct CarQuery {
    code: Option<String>,
    provide: Option<String>,
    status: Option<i64>,
}

/// 校验运营方数据归属
pub fn check_provide_access(claims: &JwtClaims, provide: &str) -> Result<String, Json<Value>> {
    if claims.role == "admin" {
        return Ok(provide.to_string());
    }
    if provide.is_empty() {
        return Ok(claims.role.clone());
    }
    if claims.role == provide {
        return Ok(provide.to_string());
    }
    Err(json_error(&format!(
        "provide mismatch: 运营商标识 ({}) 与账号 ({}) 不匹配",
        provide,
        claims.role
    )))
}

pub async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::EbikeGrpcClient, Json<Value>> {
    let clients = state.grpc_clients.read().await;
    clients.ebike_client().await
        .map_err(|e| json_error(&format!("ebike-service 不可用：{e}")))
}

// ==================== 车辆管理 ====================

fn car_info_to_json(c: &ebike::CarInfo) -> Value {
    let gps = serde_json::from_str::<Value>(&c.gps_json).unwrap_or_else(|_| json!({"lng": " ", "lat": " "}));
    let time = serde_json::from_str::<Value>(&c.time_json).unwrap_or_else(|_| json!({"start": " ", "end": " "}));
    json!({
        "code": c.code, "status": c.status, "provide": c.provide,
        "speed": c.speed, "gps": gps, "time": time,
        "gps_type": c.gps_type, "alert": c.alert, "remark": c.remark,
        "create_date": c.create_date, "update_date": c.update_date, "delete": c.delete,
    })
}

async fn car_query(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CarQuery>,
    Extension(claims): Extension<JwtClaims>,
) -> Json<Value> {
    let provide = if claims.role == "admin" {
        q.provide.clone().unwrap_or_default()
    } else {
        q.provide.clone().unwrap_or_else(|| claims.role.clone())
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.car_query(
        q.code.unwrap_or_default(),
        provide,
        q.status.unwrap_or(-1),
        String::new(), String::new(),
    ).await {
        Ok(resp) => json_success(json!(resp.cars.iter().map(car_info_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败：{e}")),
    }
}

async fn car_add(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let provide = body["provide"].as_str().unwrap_or("");
    let effective_provide = match check_provide_access(&claims, provide) {
        Ok(p) => p, Err(e) => return e,
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    let req = ebike::CarAddRequest {
        code: body["code"].as_str().unwrap_or("").to_string(),
        status: body["status"].as_i64().unwrap_or(0),
        provide: effective_provide,
        speed: body["speed"].as_f64().unwrap_or(0.0),
        gps: Some(ebike::GpsInfo {
            lng: body["gps"]["lng"].as_str().unwrap_or("").to_string(),
            lat: body["gps"]["lat"].as_str().unwrap_or("").to_string(),
        }),
        gps_type: body["gps_type"].as_i64().unwrap_or(0),
        time: Some(ebike::TimeRange {
            start: body["time"]["start"].as_str().unwrap_or("").to_string(),
            end: body["time"]["end"].as_str().unwrap_or("").to_string(),
        }),
        alert: body["alert"].as_str().unwrap_or("").to_string(),
        remark: body["remark"].as_str().unwrap_or("").to_string(),
    };
    match client.car_add(req).await {
        Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message})),
        Err(e) => json_error(&format!("新增失败：{e}")),
    }
}

async fn car_delete(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let provide = body["provide"].as_str().unwrap_or("");
    if let Err(e) = check_provide_access(&claims, provide) {
        return e;
    }
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.car_delete(body["code"].as_str().unwrap_or("").to_string(), provide.to_string()).await {
        Ok(resp) => json_success(json!({"code": resp.code})),
        Err(e) => json_error(&format!("删除失败：{e}")),
    }
}

async fn car_history(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.car_history(body["code"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp.cars.iter().map(car_info_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败：{e}")),
    }
}

async fn car_alert(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    let req = ebike::CarAlertRequest {
        code: body["code"].as_str().unwrap_or("").to_string(),
        provide: body["provide"].as_str().unwrap_or("").to_string(),
        status: body["status"].as_i64().unwrap_or(-1),
        time: body["time"].as_str().unwrap_or("").to_string(),
        alert: body["alert"].as_str().unwrap_or("").to_string(),
        remark: body["remark"].as_str().unwrap_or("").to_string(),
    };
    match client.car_alert(req).await {
        Ok(resp) => json_success(json!(resp.cars.iter().map(car_info_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败：{e}")),
    }
}

// ==================== 订单管理 ====================

fn order_info_to_json(o: &ebike::OrderInfo) -> Value {
    let gps = serde_json::from_str::<Value>(&o.gps_json).unwrap_or_else(|_| json!({"lng": " ", "lat": " "}));
    let time = serde_json::from_str::<Value>(&o.time_json).unwrap_or_else(|_| json!({"start": " ", "end": " "}));
    json!({
        "code": o.code, "status": o.status, "provide": o.provide,
        "speed": o.speed, "gps": gps, "time": time,
        "gps_type": o.gps_type, "alert": o.alert, "remark": o.remark,
        "order": o.order, "payable": o.payable, "pay": o.pay,
        "refund": o.refund, "coupon": o.coupon,
        "pay_type": o.pay_type, "pay_time": o.pay_time, "pay_status": o.pay_status,
        "create_date": o.create_date, "update_date": o.update_date, "delete": o.delete,
    })
}

async fn order_query(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let provide = if claims.role == "admin" {
        body["provide"].as_str().unwrap_or("").to_string()
    } else {
        let p = body["provide"].as_str().unwrap_or("");
        if p.is_empty() { claims.role.clone() } else { p.to_string() }
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.order_query(
        body["code"].as_str().unwrap_or("").to_string(),
        provide,
        body["status"].as_i64().unwrap_or(-1),
        body["time"]["start"].as_str().unwrap_or("").to_string(),
        body["time"]["end"].as_str().unwrap_or("").to_string(),
        body["order"].as_str().unwrap_or("").to_string(),
        body["paystatus"].as_i64().unwrap_or(-1),
        body["paytype"].as_i64().unwrap_or(-1),
        body["paytime"].as_str().unwrap_or("").to_string(),
    ).await {
        Ok(resp) => json_success(json!(resp.orders.iter().map(order_info_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败：{e}")),
    }
}

// ==================== 停放区管理 ====================

fn storage_info_to_json(s: &ebike::StorageInfo) -> Value {
    let gps = serde_json::from_str::<Value>(&s.gps_json).unwrap_or_else(|_| json!({"lng": " ", "lat": " "}));
    json!({
        "code": s.code, "status": s.status, "provide": s.provide,
        "gps": gps, "gps_type": s.gps_type,
        "sum": s.sum, "cur": s.cur, "points": s.points,
        "alert": s.alert, "remark": s.remark,
        "create_date": s.create_date, "update_date": s.update_date, "delete": s.delete,
    })
}

async fn storage_query(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CarQuery>,
    Extension(claims): Extension<JwtClaims>,
) -> Json<Value> {
    let provide = if claims.role == "admin" {
        q.provide.clone().unwrap_or_default()
    } else {
        q.provide.clone().unwrap_or_else(|| claims.role.clone())
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.storage_query(
        q.code.unwrap_or_default(),
        provide,
        q.status.unwrap_or(-1),
    ).await {
        Ok(resp) => json_success(json!(resp.storages.iter().map(storage_info_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败：{e}")),
    }
}

async fn storage_add(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let provide = body["provide"].as_str().unwrap_or("");
    let effective_provide = match check_provide_access(&claims, provide) {
        Ok(p) => p, Err(e) => return e,
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    let req = ebike::StorageAddRequest {
        code: body["code"].as_str().unwrap_or("").to_string(),
        status: body["status"].as_i64().unwrap_or(0),
        provide: effective_provide,
        gps: Some(ebike::GpsInfo {
            lng: body["gps"]["lng"].as_str().unwrap_or("").to_string(),
            lat: body["gps"]["lat"].as_str().unwrap_or("").to_string(),
        }),
        gps_type: body["gps_type"].as_i64().unwrap_or(0),
        sum: body["sum"].as_i64().unwrap_or(0),
        cur: body["cur"].as_i64().unwrap_or(0),
        alert: body["alert"].as_str().unwrap_or("").to_string(),
        remark: body["remark"].as_str().unwrap_or("").to_string(),
        points: body["points"].as_str().unwrap_or("").to_string(),
    };
    match client.storage_add(req).await {
        Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message})),
        Err(e) => json_error(&format!("新增失败：{e}")),
    }
}

async fn storage_delete(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let provide = body["provide"].as_str().unwrap_or("");
    let _effective_provide = match check_provide_access(&claims, provide) {
        Ok(p) => p, Err(e) => return e,
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.storage_delete(body["code"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!({"code": resp.code})),
        Err(e) => json_error(&format!("删除失败：{e}")),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 车辆
        .route("/api/v1/ebike/car/query", get(car_query))
        .route("/api/v1/ebike/car/add", post(car_add))
        .route("/api/v1/ebike/car/delete", post(car_delete))
        .route("/api/v1/ebike/car/history", post(car_history))
        .route("/api/v1/ebike/car/alert", post(car_alert))
        // 订单
        .route("/api/v1/ebike/order/query", post(order_query))
        // 停放区
        .route("/api/v1/ebike/storage/query", get(storage_query))
        .route("/api/v1/ebike/storage/add", post(storage_add))
        .route("/api/v1/ebike/storage/delete", post(storage_delete))
}
