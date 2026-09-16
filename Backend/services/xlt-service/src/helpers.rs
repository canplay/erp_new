//! JSON response helper functions for xlt-service

use axum::Json;
use axum::http::StatusCode;
use serde_json::{json, Value};

/// Success response with data
pub fn json_success<T: serde::Serialize>(data: T) -> Json<Value> {
    Json(serde_json::json!(data))
}

/// Error response with formatted message
pub fn json_error(msg: &str) -> Json<Value> {
    Json(json!({"error": msg}))
}

/// Error response with formatted message and status code
pub fn json_error_fmt(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({"error": msg})),
    )
}

/// Success with code and message
pub fn json_ok(code: u16, message: &str) -> Json<Value> {
    Json(json!({"code": code, "message": message}))
}

/// Health check response
pub fn json_health(service: &str) -> Json<Value> {
    Json(json!({"status": "ok", "service": service}))
}

/// Empty success response
pub fn json_empty_success() -> Json<Value> {
    Json(json!({"success": true}))
}
