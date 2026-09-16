//! 共享辅助函数 — api-gateway 路由模块通用工具
//!
//! 所有 `*_routes.rs` 应优先使用此模块的函数，
//! 避免在各路由文件中重复定义。

use axum::Json;
use axum::http::StatusCode;
use serde_json::{json, Value};

/// 成功响应（带 data）
pub fn json_success<T: Into<Value>>(data: T) -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": data.into()}))
}

/// 成功响应（无 data）
pub fn json_ok() -> Json<Value> {
    Json(json!({"success": true, "code": 200}))
}

/// 错误响应（500）
pub fn json_error(msg: &str) -> Json<Value> {
    Json(json!({"success": false, "code": 500, "message": msg}))
}

/// 未实现响应（501）
pub fn json_not_implemented(msg: &str) -> Json<Value> {
    Json(json!({"success": false, "code": 501, "message": msg}))
}

/// 错误响应（带 error Display）— 双参数版本，接受任意 Display 类型
pub fn json_error_fmt(msg: &str, e: &impl std::fmt::Display) -> Json<Value> {
    Json(json!({"success": false, "code": 500, "message": format!("{}: {}" , msg, e)}))
}

/// 错误响应（带 StatusCode）— 用于 map_err 场景，返回 (StatusCode, Json<Value>) 元组
pub fn json_error_response(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"success": false, "code": status.as_u16(), "message": msg})))
}

/// 错误响应（带 StatusCode）— 用于 map_err 场景，接受任意 Display 类型
pub fn json_error_response_fmt(
    status: StatusCode,
    msg: &str,
    e: &impl std::fmt::Display,
) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({"success": false, "code": status.as_u16(), "message": format!("{}: {}" , msg, e)})),
    )
}

/// gRPC 服务不可用的错误响应
pub fn json_grpc_error(message: String) -> Json<Value> {
    Json(json!({"success": false, "code": 503, "message": message}))
}

/// LPR 回调响应 — 使用 grpc_proto 生成的响应
pub fn json_lpr_response(response: &grpc_proto::lpr::LprCallbackResponse) -> Json<Value> {
    Json(json!({"success": response.success, "code": response.code, "message": response.message}))
}

/// Deprecated: 空列表 — 使用 `json_success(Vec::new())` 替代
#[deprecated(note = "使用 json_success(Vec::new()) 替代" )]
pub async fn stub_list() -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": {"list": [], "total": 0}}))
}

/// Deprecated: 空成功 — 使用 `json_ok()` 替代
#[deprecated(note = "使用 json_ok() 替代" )]
pub async fn stub_ok() -> Json<Value> {
    Json(json!({"success": true, "code": 200}))
}

/// Deprecated: 空 JSON — 使用 `json_success(())` 替代
#[deprecated(note = "使用 json_success(()) 替代" )]
pub async fn stub_json() -> Json<Value> {
    Json(json!({"success": true, "code": 200, "data": null}))
}

/// gRPC client accessor macro — reads client from AppState and returns
/// a JSON error response on failure (eliminates ~10 duplicate get_*_client fns).
///
/// Usage: `grpc_client!(state, audit_client)`
/// Expands to: `state.grpc_clients.read().await.audit_client().await.map_err(|e| json_error(&format!("{} 不可用: {}" , "audit_client" , e)))?`
#[macro_export]
macro_rules! grpc_client {
    ($state:expr, $client:ident) => {
        match $state.grpc_clients.read().await.$client().await {
            Ok(c) => c,
            Err(e) => return json_error(&format!("{} 不可用: {}" , stringify!($client), e)),
        }
    };
}
