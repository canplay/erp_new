//! JSON response helper functions
//!
//! Provides convenience functions for constructing standard JSON responses.

use axum::Json;
use axum::http::StatusCode;
use serde_json::{json, Value};

/// Success response with data
pub fn json_success<T: Into<Value>>(data: T) -> Json<Value> {
    Json(json!({"success": true, "data": data.into()}))
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
    Json(json!({"success": false, "error": format!("{}: {}", msg, e)}))
}

/// Error response with message field
pub fn json_error_msg(msg: &str) -> Json<Value> {
    Json(json!({"success": false, "message": msg}))
}

/// Error response with formatted message field
pub fn json_error_msg_fmt(msg: &str, e: &impl std::fmt::Display) -> Json<Value> {
    Json(json!({"success": false, "message": format!("{}: {}", msg, e)}))
}

/// Health check response
pub fn json_health(service_name: &str) -> Json<Value> {
    Json(json!({"status": "healthy", "service": service_name}))
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

/// API Key created response (201)
pub fn json_api_key_created(key_id: &str, key_secret: &str, name: &str, permission_level: &str, expires_at: Option<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CREATED,
        Json(json!({
            "code": 201,
            "message": "密钥创建成功，请妥善保管密钥Secret，仅此次可见",
            "data": {
                "key_id": key_id,
                "key_secret": key_secret,
                "name": name,
                "permission_level": permission_level,
                "expires_at": expires_at
            }
        }))
    )
}

/// API Key list response
pub fn json_api_key_list(keys: Vec<serde_json::Value>, total: i64, page: i64, page_size: i64) -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": "操作成功",
        "data": {
            "records": keys,
            "total": total,
            "page": page,
            "page_size": page_size
        }
    }))
}

/// API Key error response
pub fn json_api_key_error(code: u16, message: &str) -> Json<Value> {
    Json(json!({
        "code": code,
        "message": message,
        "data": null
    }))
}

/// API Key detail response
pub fn json_api_key_detail(key: &crate::models::ApiKey) -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": "操作成功",
        "data": key
    }))
}

/// API Key updated response
pub fn json_api_key_updated(key: &crate::models::ApiKey) -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": "更新成功",
        "data": key
    }))
}

/// API Key deleted response
pub fn json_api_key_deleted() -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": "删除成功"
    }))
}

/// API Key disabled response
pub fn json_api_key_disabled() -> Json<Value> {
    Json(json!({
        "valid": false,
        "error": "密钥已被禁用"
    }))
}

/// API Key expired response
pub fn json_api_key_expired() -> Json<Value> {
    Json(json!({
        "valid": false,
        "error": "密钥已过期"
    }))
}

/// API Key IP not allowed response
pub fn json_api_key_ip_not_allowed() -> Json<Value> {
    Json(json!({
        "valid": false,
        "error": "IP地址不被允许"
    }))
}

/// API Key invalid response
pub fn json_api_key_invalid() -> Json<Value> {
    Json(json!({
        "valid": false,
        "error": "密钥无效"
    }))
}

/// API Key valid response
pub fn json_api_key_valid(key_id: &str, permission_level: &str) -> Json<Value> {
    Json(json!({
        "valid": true,
        "key_id": key_id,
        "permission_level": permission_level
    }))
}

/// API Key simple message response
pub fn json_api_key_simple(message: &str) -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": message
    }))
}

/// API Key stats response
pub fn json_api_key_stats(total: i64) -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": "操作成功",
        "data": {
            "total_keys": total,
            "active_keys": total,
            "expired_keys": 0,
            "revoked_keys": 0,
            "total_requests": 0,
            "failed_requests": 0,
            "last_active": null
        }
    }))
}

/// API Key error with status
pub fn json_api_key_error_response(status: StatusCode, code: u16, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "code": code,
            "message": message
        }))
    )
}
