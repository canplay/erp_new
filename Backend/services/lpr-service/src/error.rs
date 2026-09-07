//! 车牌识别服务错误类型
//!
//! 统一错误处理，支持 axum `IntoResponse`。

use axum::http::StatusCode;
    
use common::impl_into_response;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LprError {
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("Redis错误: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("HTTP请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for LprError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("Database error: {e:?}");
        LprError::DatabaseError(e.to_string())
    }
}

impl_into_response!(LprError, {
    LprError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    LprError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    LprError::RequestError(_) => StatusCode::BAD_GATEWAY,
    LprError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
});

pub type Result<T> = std::result::Result<T, LprError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    #[test]
    fn test_internal_error_status() {
        let response = LprError::Internal("test".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_database_error_status() {
        let response = LprError::DatabaseError("conn".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
