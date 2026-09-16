//! JSON response helper functions for tow-service
//!
//! Provides convenience functions for constructing standard JSON responses.

use axum::Json;
use axum::http::StatusCode;
use serde_json::{json, Value};

/// Success response with data
pub fn json_success(data: Value) -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({"status": 1, "message": "success" , "data": data})),
    )
}

/// Error response with message
pub fn json_error(msg: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status": 0, "message": msg})),
    )
}

/// Health check response
pub fn json_health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({"status": 1, "message": "success" })),
    )
}
