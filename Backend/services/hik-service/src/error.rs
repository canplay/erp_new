// 错误类型定义
// Error types for hik service

use axum::http::StatusCode;
    
use common::impl_into_response;
use thiserror::Error;

/// 海康服务错误类型
#[derive(Error, Debug)]
pub enum HikError {
    #[error("令牌获取失败")]
    TokenError,

    #[error("方法无效")]
    InvalidMethod,

    #[error("第三方API调用失败: {0}")]
    ApiError(String),

    #[error("参数错误")]
    InvalidParams,

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis错误: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("HTTP请求错误: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl_into_response!(HikError, {
    HikError::TokenError => StatusCode::UNAUTHORIZED,
    HikError::InvalidMethod => StatusCode::BAD_REQUEST,
    HikError::ApiError(_) => StatusCode::BAD_GATEWAY,
    HikError::InvalidParams => StatusCode::BAD_REQUEST,
    HikError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    HikError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    HikError::RequestError(_) => StatusCode::BAD_GATEWAY,
});

/// 结果类型别名
pub type Result<T> = std::result::Result<T, HikError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    #[test]
    fn test_token_error_status() {
        let response = HikError::TokenError.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_invalid_method_status() {
        let response = HikError::InvalidMethod.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_status() {
        let response = HikError::ApiError("test".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_invalid_params_status() {
        let response = HikError::InvalidParams.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(HikError::TokenError.to_string(), "令牌获取失败");
        assert_eq!(HikError::InvalidMethod.to_string(), "方法无效");
        assert_eq!(HikError::InvalidParams.to_string(), "参数错误");
        assert_eq!(
            HikError::ApiError("err".to_string()).to_string(),
            "第三方API调用失败: err"
        );
    }
}
