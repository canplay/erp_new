//! 数据清理服务错误定义

use common::impl_into_response;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum CleanError {#[error("资源不存在")]
 NotFound, #[error("数据库错误: {0}")]
 DatabaseError(String),

 #[error("参数错误: {0}")]
 InvalidParam(String),

 #[error("删除失败: {0}")]
 DeleteFailed(String),
}

impl_into_response!(CleanError, {CleanError::NotFound => StatusCode::NOT_FOUND, CleanError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR, CleanError::InvalidParam(_) => StatusCode::BAD_REQUEST, CleanError::DeleteFailed(_) => StatusCode::INTERNAL_SERVER_ERROR});

pub type CleanServiceError = CleanError;
pub type Result<T> = std::result::Result<T, CleanError>;
pub type CleanResult<T> = std::result::Result<T, CleanError>;

impl From<sqlx::Error> for CleanError {fn from(err: sqlx::Error) -> Self {
 tracing::error!("数据库错误: {err:?}");
 Self::DatabaseError(err.to_string())
 }
}
