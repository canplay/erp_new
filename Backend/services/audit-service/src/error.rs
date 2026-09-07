//! 审计服务错误定义

use common::impl_into_response;
use axum::http::StatusCode;

/// 审计服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("日志不存在")]
    NotFound,

    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("参数错误: {0}")]
    InvalidParam(String),

    #[error("删除失败: {0}")]
    DeleteFailed(String),
}

impl_into_response!(AuditError, {
    AuditError::NotFound => StatusCode::NOT_FOUND,
    AuditError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    AuditError::InvalidParam(_) => StatusCode::BAD_REQUEST,
    AuditError::DeleteFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
});

/// 审计服务操作结果
pub type AuditResult<T> = Result<T, AuditError>;

impl From<sqlx::Error> for AuditError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("数据库错误: {err:?}");
        Self::DatabaseError(err.to_string())
    }
}
