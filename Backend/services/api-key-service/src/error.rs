//! API密钥服务错误定义

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde_json::json;
use thiserror::Error;

/// API密钥服务错误类型
#[derive(Error, Debug)]
pub enum ApiKeyError {
    #[error("密钥不存在: {0}")]
    KeyNotFound(String),

    #[error("密钥已过期")]
    KeyExpired,

    #[error("密钥已被禁用")]
    KeyDisabled,

    #[error("密钥验证失败")]
    KeyInvalid,

    #[error("IP地址不被允许")]
    IpNotAllowed,

    #[error("超过请求限制")]
    RateLimitExceeded,

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("未授权访问")]
    Unauthorized,

    #[error("禁止访问")]
    Forbidden,
}

/// 错误响应
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

/// 自定义 code 映射
fn error_code(err: &ApiKeyError) -> u16 {
    match err {
        ApiKeyError::KeyNotFound(_) => 40401,
        ApiKeyError::KeyExpired => 40101,
        ApiKeyError::KeyDisabled => 40301,
        ApiKeyError::KeyInvalid => 40102,
        ApiKeyError::IpNotAllowed => 40302,
        ApiKeyError::RateLimitExceeded => 42901,
        ApiKeyError::Unauthorized => 40100,
        ApiKeyError::Forbidden => 40300,
        ApiKeyError::DatabaseError(_) => 50001,
    }
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiKeyError::KeyNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiKeyError::KeyExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiKeyError::KeyDisabled => (StatusCode::FORBIDDEN, self.to_string()),
            ApiKeyError::KeyInvalid => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiKeyError::IpNotAllowed => (StatusCode::FORBIDDEN, self.to_string()),
            ApiKeyError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            ApiKeyError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiKeyError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiKeyError::DatabaseError(e) => {
                tracing::error!("Database error: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误".to_string())
            }
        };

        let body = Json(json!({
            "code": error_code(&self),
            "message": message,
        }));

        (status, body).into_response()
    }
}

/// 服务结果类型别名
pub type Result<T> = std::result::Result<T, ApiKeyError>;
