//! JSON response helper functions for browser-service
//!
//! Provides convenience functions for constructing standard JSON responses.

use axum::Json;
use axum::http::StatusCode;
use serde_json::{json, Value};

/// Wrap a serde_json Value into a Json response
pub fn json_ok(data: Value) -> Json<Value> {
    Json(data)
}

/// Error response: `{"error": msg}`
pub fn json_error(msg: &str) -> Json<Value> {
    Json(json!({"error": msg}))
}

/// Error response with formatted message: `{"error": "prefix: detail"}`
pub fn json_error_fmt(prefix: &str, e: &impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("{}: {}", prefix, e)})),
    )
}

/// Error response with a specific status code
pub fn json_error_status(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"error": msg})))
}

/// Service unavailable error
pub fn json_unavailable(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": msg})))
}
