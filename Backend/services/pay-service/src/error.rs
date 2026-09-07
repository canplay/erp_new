// 错误类型定义
// Error types for pay service

use axum::http::StatusCode;
    
use common::impl_into_response;
use thiserror::Error;

/// 支付服务错误类型
/// 注意: 预留部分变体用于未来功能扩展
#[derive(Error, Debug)]
pub enum PayError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis错误: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("HTTP请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("参数错误")]
    InvalidParams,

    #[error("支付订单不存在")]
    NotFound,

    #[error("内部错误: {0}")]
    InternalError(String),
}

impl_into_response!(PayError, {
    PayError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    PayError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    PayError::RequestError(_) => StatusCode::BAD_GATEWAY,
    PayError::InvalidParams => StatusCode::BAD_REQUEST,
    PayError::NotFound => StatusCode::NOT_FOUND,
    PayError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
});

/// 结果类型别名
pub type Result<T> = std::result::Result<T, PayError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    #[test]
    fn test_invalid_params_status() {
        let response = PayError::InvalidParams.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_status() {
        let response = PayError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_internal_error_status() {
        let response = PayError::InternalError("test".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(PayError::InvalidParams.to_string(), "参数错误");
        assert_eq!(PayError::NotFound.to_string(), "支付订单不存在");
        assert_eq!(
            PayError::InternalError("msg".to_string()).to_string(),
            "内部错误: msg"
        );
    }
}
