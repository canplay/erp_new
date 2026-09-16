//! Hik 服务路由 — gRPC 调用

use std::sync::Arc;
use axum::{Router, Json, extract::State, routing::post};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

async fn get_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::HikGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.hik_client().await
        .map_err(|e| json_error(&format!("hik-service 不可用: {e}" )))
}

async fn hik_exec(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let mut client = match get_client(&state).await {
        Ok(c) => c, Err(r) => return r,
    };
    match client.exec(
        body["method" ].as_str().unwrap_or("" ).to_string(),
        body["phone" ].as_i64(),
        body["driver_id" ].as_str().map(|s| s.to_string()),
        body["plate_no" ].as_str().map(|s| s.to_string()),
        body["plate_color" ].as_str().map(|s| s.to_string()),
        body["page_no" ].as_i64().map(|v| v as i32),
        body["page_size" ].as_i64().map(|v| v as i32),
        body["park_name" ].as_str().map(|s| s.to_string()),
        body["park_code" ].as_str().map(|s| s.to_string()),
        body["request_type" ].as_str().map(|s| s.to_string()),
        body["unique_id" ].as_str().map(|s| s.to_string()),
        body["is_use_coupon" ].as_str().map(|s| s.to_string()),
        body["appeal_type" ].as_str().map(|s| s.to_string()),
        body["appeal_remark" ].as_str().map(|s| s.to_string()),
        body["appeal_in_time" ].as_str().map(|s| s.to_string()),
        body["appeal_out_time" ].as_str().map(|s| s.to_string()),
        body["appeal_source" ].as_str().map(|s| s.to_string()),
        body["arrears_ids" ].as_str().map(|s| s.to_string()),
    ).await {
        Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message, "data": resp.data})),
        Err(e) => json_error(&format!("hik exec 失败: {e}" )),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/hik/exec" , post(hik_exec))
        .route("/api/v1/hik/coupon" , post(|State(state): State<Arc<AppState>>, Json(body): Json<Value>| async move {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            match client.coupon(
                body["phone" ].as_str().unwrap_or("" ).to_string(),
                body["amount" ].as_i64().unwrap_or(0) as i32,
                body["start" ].as_i64().unwrap_or(0),
                body["end" ].as_str().unwrap_or("" ).to_string(),
                body["type" ].as_str().unwrap_or("" ).to_string(),
            ).await {
                Ok(resp) => json_success(json!({"code": resp.code, "message": resp.message})),
                Err(e) => json_error(&format!("coupon 失败: {e}" )),
            }
        }))
        .route("/api/v1/hik/signo/open" , post(|State(state): State<Arc<AppState>>, Json(body): Json<Value>| async move {
            let mut client = match get_client(&state).await { Ok(c) => c, Err(r) => return r };
            match client.signo_open(
                body["place" ].as_str().unwrap_or("" ).to_string(),
                body["name" ].as_str().unwrap_or("" ).to_string(),
            ).await {
                Ok(resp) => json_success(json!({"message": resp.message, "status": resp.status})),
                Err(e) => json_error(&format!("signo_open 失败: {e}" )),
            }
        }))
}
