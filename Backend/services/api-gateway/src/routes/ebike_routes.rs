//! E-Bike 路由 — 真实 gRPC 调用
//!
//! 提供共享单车数据接入接口：
//! - 运营方推送：车辆新增/批量/删除（method=add / batch_add / delete）
//! - 政府查询：车辆/历史/告警/订单/停放区/配置
//! - 兼容旧 method 分发格式（POST /api/v1/ebike/car 带 method 字段）
//!
//! 响应统一转换为前端友好格式（gps_json/time_json 字符串 → gps/time 对象）

use std::sync::Arc;
use axum::{Router, Json, extract::{State, Query, Extension}, routing::{get, post}};
use serde::Deserialize;
use serde_json::{json, Value};
use grpc_proto::ebike;

use crate::AppState;
use crate::middleware::JwtClaims;
use crate::routes::helpers::*;
use crate::routes::options_routes::options_query;
use crate::routes::options_routes::options_update;

#[derive(Deserialize)]
struct CarQuery {
    code: Option<String>,
    provide: Option<String>,
    status: Option<i64>,
}

/// 校验运营方数据归属：非 admin 的运营方只能操作自己的 provide
/// 返回 Ok(有效 provide) / Err(错误响应)
/// 运营方未显式传 provide 时，自动使用账号 role（便于删除只传 code）
fn check_provide_access(claims: &JwtClaims, provide: &str) -> Result<String, Json<Value>> {
    // admin（政府管理员）可操作所有运营方数据
    if claims.role == "admin" {
        return Ok(provide.to_string());
    }
    // 运营方账号：role 即 provide 标识
    if provide.is_empty() {
        // 未传 provide → 自动使用账号 role
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

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::EbikeGrpcClient, Json<Value>> {
    // 使用读锁：请求期间只读不写，允许多个请求并发访问同一 channel
    let clients = state.grpc_clients.read().await;
    clients.ebike_client().await
        .map_err(|e| json_error(&format!("ebike-service 不可用：{e}")))
}

/// 将 proto 的 CarInfo 转换为前端友好 JSON（gps_json/time_json 字符串 → 对象）
fn car_info_to_json(c: &ebike::CarInfo) -> Value {
    let gps = serde_json::from_str::<Value>(&c.gps_json).unwrap_or_else(|_| json!({"lng": "", "lat": ""}));
    let time = serde_json::from_str::<Value>(&c.time_json).unwrap_or_else(|_| json!({"start": "", "end": ""}));
    json!({
        "code": c.code,
        "status": c.status,
        "provide": c.provide,
        "speed": c.speed,
        "gps": gps,
        "time": time,
        "gps_type": c.gps_type,
        "alert": c.alert,
        "remark": c.remark,
        "create_date": c.create_date,
        "update_date": c.update_date,
        "delete": c.delete,
    })
}

/// 将 proto 的 StorageInfo 转换为前端友好 JSON
fn storage_info_to_json(s: &ebike::StorageInfo) -> Value {
    let gps = serde_json::from_str::<Value>(&s.gps_json).unwrap_or_else(|_| json!({"lng": "", "lat": ""}));
    json!({
        "code": s.code,
        "status": s.status,
        "provide": s.provide,
        "gps": gps,
        "gps_type": s.gps_type,
        "sum": s.sum,
        "cur": s.cur,
        "points": s.points,
        "alert": s.alert,
        "remark": s.remark,
        "create_date": s.create_date,
        "update_date": s.update_date,
        "delete": s.delete,
    })
}

/// 将 proto 的 OrderInfo 转换为前端友好 JSON
fn order_info_to_json(o: &ebike::OrderInfo) -> Value {
    let gps = serde_json::from_str::<Value>(&o.gps_json).unwrap_or_else(|_| json!({"lng": "", "lat": ""}));
    let time = serde_json::from_str::<Value>(&o.time_json).unwrap_or_else(|_| json!({"start": "", "end": ""}));
    json!({
        "code": o.code,
        "status": o.status,
        "provide": o.provide,
        "speed": o.speed,
        "gps": gps,
        "time": time,
        "gps_type": o.gps_type,
        "alert": o.alert,
        "remark": o.remark,
        "order": o.order,
        "payable": o.payable,
        "pay": o.pay,
        "refund": o.refund,
        "coupon": o.coupon,
        "pay_type": o.pay_type,
        "pay_time": o.pay_time,
        "pay_status": o.pay_status,
        "create_date": o.create_date,
        "update_date": o.update_date,
        "delete": o.delete,
    })
}

// ==================== 车辆管理 ====================

async fn car_query(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CarQuery>,
    Extension(claims): Extension<JwtClaims>,
) -> Json<Value> {
    // 非 admin 运营方只能看到自己的数据
    let provide = if claims.role == "admin" {
        q.provide.clone().unwrap_or_default()
    } else {
        q.provide.clone().unwrap_or_else(|| claims.role.clone())
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match client.car_query(
        q.code.unwrap_or_default(),
        provide,
        q.status.unwrap_or(-1),
        String::new(),
        String::new(),
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
    // 鉴权：运营方只能推自己的车；未传 provide 时自动用账号 role
    let provide = body["provide"].as_str().unwrap_or("");
    let effective_provide = match check_provide_access(&claims, provide) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
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
    // 删除需校验 provide（运营方只能删自己的车）
    let provide = body["provide"].as_str().unwrap_or("");
    if let Err(e) = check_provide_access(&claims, provide) {
        return e;
    }
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match client.car_delete(body["code"].as_str().unwrap_or("").to_string(), provide.to_string()).await {
        Ok(resp) => json_success(json!({"code": resp.code})),
        Err(e) => json_error(&format!("删除失败：{e}")),
    }
}

async fn car_history(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match client.car_history(body["code"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!(resp.cars.iter().map(car_info_to_json).collect::<Vec<_>>())),
        Err(e) => json_error(&format!("查询失败：{e}")),
    }
}

async fn car_alert(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
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

// ==================== 停放区管理 ====================

async fn storage_query(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CarQuery>,
    Extension(claims): Extension<JwtClaims>,
) -> Json<Value> {
    // 非 admin 运营方只能看到自己的数据
    let provide = if claims.role == "admin" {
        q.provide.clone().unwrap_or_default()
    } else {
        q.provide.clone().unwrap_or_else(|| claims.role.clone())
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
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
    // 鉴权：运营方只能登记自己的停放区
    let provide = body["provide"].as_str().unwrap_or("");
    let effective_provide = match check_provide_access(&claims, provide) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
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
    // 鉴权：运营方只能删除自己的停放区
    let provide = body["provide"].as_str().unwrap_or("");
    let _effective_provide = match check_provide_access(&claims, provide) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match client.storage_delete(body["code"].as_str().unwrap_or("").to_string()).await {
        Ok(resp) => json_success(json!({"code": resp.code})),
        Err(e) => json_error(&format!("删除失败：{e}")),
    }
}

// ==================== 订单管理 ====================

async fn order_query(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    // 非 admin 运营方只能看到自己的数据
    let provide = if claims.role == "admin" {
        body["provide"].as_str().unwrap_or("").to_string()
    } else {
        let p = body["provide"].as_str().unwrap_or("");
        if p.is_empty() { claims.role.clone() } else { p.to_string() }
    };
    let mut client = match get_client(&state).await {
        Ok(c) => c,
        Err(r) => return r,
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

// ==================== 兼容旧 method 分发格式 ====================
// 前端旧页面调用：POST /api/v1/ebike/car  body {method: 'query'|'add'|'batch_add'|'delete'|'history'|'alert'}
// POST /api/v1/ebike/storage  body {method: 'query'|'add'|'delete'}
// POST /api/v1/ebike/order    body {method: 'query'}

async fn car_dispatch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let method = body["method"].as_str().unwrap_or("");
    match method {
        "query" => {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            match client.car_query(
                body["code"].as_str().unwrap_or("").to_string(),
                body["provide"].as_str().unwrap_or("").to_string(),
                body["status"].as_i64().unwrap_or(-1),
                body["time"]["start"].as_str().unwrap_or("").to_string(),
                body["time"]["end"].as_str().unwrap_or("").to_string(),
            ).await {
                Ok(resp) => json_success(json!(resp.cars.iter().map(car_info_to_json).collect::<Vec<_>>())),
                Err(e) => json_error(&format!("查询失败：{e}")),
            }
        }
        "add" => car_add(State(state), Extension(claims), Json(body)).await,
        "batch_add" => {
            // 鉴权：批量推送每辆车 provide 必须匹配账号
            if let Some(items) = body["items"].as_array() {
                for item in items {
                    let p = item["provide"].as_str().unwrap_or("");
                    if let Err(e) = check_provide_access(&claims, p) {
                        return e;
                    }
                }
            }
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            let items: Vec<ebike::CarAddRequest> = body["items"]
                .as_array()
                .map(|arr| arr.iter().map(|item| ebike::CarAddRequest {
                    code: item["code"].as_str().unwrap_or("").to_string(),
                    status: item["status"].as_i64().unwrap_or(0),
                    provide: item["provide"].as_str().unwrap_or("").to_string(),
                    speed: item["speed"].as_f64().unwrap_or(0.0),
                    gps: Some(ebike::GpsInfo {
                        lng: item["gps"]["lng"].as_str().unwrap_or("").to_string(),
                        lat: item["gps"]["lat"].as_str().unwrap_or("").to_string(),
                    }),
                    gps_type: item["gps_type"].as_i64().unwrap_or(0),
                    time: Some(ebike::TimeRange {
                        start: item["time"]["start"].as_str().unwrap_or("").to_string(),
                        end: item["time"]["end"].as_str().unwrap_or("").to_string(),
                    }),
                    alert: item["alert"].as_str().unwrap_or("").to_string(),
                    remark: item["remark"].as_str().unwrap_or("").to_string(),
                }).collect()).unwrap_or_default();
            let req = ebike::CarBatchAddRequest { items };
            match client.car_batch_add(req).await {
                Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message})),
                Err(e) => json_error(&format!("批量新增失败：{e}")),
            }
        }
        "delete" => car_delete(State(state), Extension(claims), Json(body)).await,
        "history" => car_history(State(state), Json(body)).await,
        "alert" => car_alert(State(state), Json(body)).await,
        _ => json_error("unknown car method"),
    }
}

async fn storage_dispatch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let method = body["method"].as_str().unwrap_or("");
    match method {
        "query" => {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            match client.storage_query(
                body["code"].as_str().unwrap_or("").to_string(),
                body["provide"].as_str().unwrap_or("").to_string(),
                body["status"].as_i64().unwrap_or(-1),
            ).await {
                Ok(resp) => json_success(json!(resp.storages.iter().map(storage_info_to_json).collect::<Vec<_>>())),
                Err(e) => json_error(&format!("查询失败：{e}")),
            }
        }
        "add" => storage_add(State(state), Extension(claims), Json(body)).await,
        "delete" => storage_delete(State(state), Extension(claims), Json(body)).await,
        "history" => {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            match client.storage_history(body["code"].as_str().unwrap_or("").to_string()).await {
                Ok(resp) => json_success(json!(resp.storages.iter().map(storage_info_to_json).collect::<Vec<_>>())),
                Err(e) => json_error(&format!("查询失败：{e}")),
            }
        }
        _ => json_error("unknown storage method"),
    }
}

async fn order_dispatch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let method = body["method"].as_str().unwrap_or("");
    match method {
        "query" => order_query(State(state), Extension(claims), Json(body)).await,
        _ => json_error("unknown order method"),
    }
}

async fn options_dispatch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if claims.role != "admin" {
        return json_error("仅管理员可查询/修改系统配置");
    }
    let method = body["method"].as_str().unwrap_or("");
    match method {
        "query" => {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            match client.options_query().await {
                Ok(resp) => {
                    let options = serde_json::from_str::<Value>(&resp.options_json).unwrap_or_else(|_| json!({}));
                    json_success(json!([{
                        "name": resp.name,
                        "options": options,
                        "level": resp.level,
                        "create_date": resp.create_date,
                        "update_date": resp.update_date,
                        "delete": resp.delete,
                    }]))
                }
                Err(e) => json_error(&format!("查询失败：{e}")),
            }
        }
        "update" => {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            let req = ebike::OptionsUpdateRequest {
                name: body["name"].as_str().unwrap_or("").to_string(),
                system: body["system"].as_i64().unwrap_or(0),
                alert: body["alert"].as_i64().unwrap_or(0),
                level: body["level"].as_i64().unwrap_or(0),
            };
            match client.options_update(req).await {
                Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message})),
                Err(e) => json_error(&format!("更新失败：{e}")),
            }
        }
        _ => json_error("unknown options method"),
    }
}

// ==================== 路由注册 ====================

async fn delegated_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, Json<Value>> {
    // E-bike login - delegate to auth-service via gRPC
    let username = body["username"].as_str().unwrap_or("").to_string();
    let password = body["password"].as_str().unwrap_or("").to_string();

    let clients = state.grpc_clients.read().await;
    let mut client = clients
        .auth_client()
        .await
        .map_err(|e| json_error(&format!("auth-service 不可用：{e}")))?;

    match client.login(username, password).await {
        Ok(resp) => Ok(json_success(json!({
            "username": resp.username,
            "role": resp.role,
            "must_change_password": resp.must_change_password
        }))),
        Err(e) => Ok(json_error(&format!("登录失败：{e}"))),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 车辆（REST + method 分发兼容）
        .route("/api/v1/ebike/login", post(delegated_login))
        .route("/api/v1/ebike/car/query", get(car_query))
        .route("/api/v1/ebike/car/add", post(car_add))
        .route("/api/v1/ebike/car/delete", post(car_delete))
        .route("/api/v1/ebike/car/history", post(car_history))
        .route("/api/v1/ebike/car/alert", post(car_alert))
        .route("/api/v1/ebike/car", post(car_dispatch))
        // 停放区
        .route("/api/v1/ebike/storage/query", get(storage_query))
        .route("/api/v1/ebike/storage/add", post(storage_add))
        .route("/api/v1/ebike/storage/delete", post(storage_delete))
        .route("/api/v1/ebike/storage", post(storage_dispatch))
        // 订单
        .route("/api/v1/ebike/order/query", post(order_query))
        .route("/api/v1/ebike/order", post(order_dispatch))
        // 配置
        .route("/api/v1/ebike/options/query", get(options_query))
        .route("/api/v1/ebike/options/update", post(options_update))
        .route("/api/v1/ebike/options", post(options_dispatch))
}
