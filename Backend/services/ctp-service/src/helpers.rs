//! JSON response helper functions
//!
//! Provides convenience functions for constructing standard JSON responses.

use axum::Json;
use serde_json::{json, Value};

/// Success response with data
pub fn json_success<T: Into<Value>>(data: T) -> Json<Value> {
    Json(json!({"success": true, "data": data.into()}))
}

/// Success response with a raw Value
pub fn json_success_value(data: serde_json::Value) -> Json<Value> {
    Json(json!({"success": true, "data": data}))
}

/// Success response with data and message
pub fn json_success_msg<T: Into<Value>>(data: T, message: &str) -> Json<Value> {
    Json(json!({"success": true, "data": data.into(), "message": message}))
}

/// Success response (no data)
pub fn json_ok() -> Json<Value> {
    Json(json!({"success": true}))
}

/// Success response with message only
pub fn json_ok_msg(message: &str) -> Json<Value> {
    Json(json!({"success": true, "message": message}))
}

/// Error response
pub fn json_error(msg: &str) -> Json<Value> {
    Json(json!({"success": false, "error": msg}))
}

/// Error response with formatted message
pub fn json_error_fmt(msg: &str, e: &impl std::fmt::Display) -> Json<Value> {
    Json(json!({"success": false, "error": format!("{}: {}" , msg, e)}))
}

/// Error response with message field
pub fn json_error_msg(msg: &str) -> Json<Value> {
    Json(json!({"success": false, "message": msg}))
}

/// Error response with formatted message field
pub fn json_error_msg_fmt(msg: &str, e: &impl std::fmt::Display) -> Json<Value> {
    Json(json!({"success": false, "message": format!("{}: {}" , msg, e)}))
}

/// Health check response
pub fn json_health(service_name: &str) -> Json<Value> {
    Json(json!({"status": "healthy" , "service": service_name}))
}

/// Created response with message
pub fn json_create<T: Into<Value>>(data: T, message: &str) -> Json<Value> {
    Json(json!({"success": true, "code": 201, "data": data.into(), "message": message}))
}

/// OK response with code
pub fn json_ok_code<T: Into<Value>>(data: T, code: u16, message: &str) -> Json<Value> {
    Json(json!({"success": true, "code": code, "data": data.into(), "message": message}))
}

/// Error response with code
pub fn json_error_code(code: u16, message: &str) -> Json<Value> {
    Json(json!({"success": false, "code": code, "message": message}))
}

/// Validation error response
pub fn json_validation_error(msg: &str) -> Json<Value> {
    Json(json!({"valid": false, "error": msg}))
}

/// Validation success response
pub fn json_validation_success(key_id: &str, permission_level: &str) -> Json<Value> {
    Json(json!({"valid": true, "key_id": key_id, "permission_level": permission_level}))
}

/// Device upload response
pub fn json_device_upload_response(error_code: i32, error_msg: &str) -> Json<Value> {
    Json(json!({"error_code": error_code, "error_msg": error_msg}))
}

/// Device control lock response
pub fn json_ctp_lock_response(success: bool, message: &str, device_no: &str, action: &str) -> Json<Value> {
    Json(json!({"success": success, "message": message, "device_no": device_no, "action": action}))
}
