//! 文件服务错误定义

use common::impl_into_response;
use axum::http::StatusCode;

/// 文件服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("文件不存在")]
    NotFound,

    #[error("文件已存在")]
    AlreadyExists,

    #[error("文件类型不支持: {0}")]
    UnsupportedType(String),

    #[error("文件大小超出限制: {0}")]
    FileTooLarge(String),

    #[error("上传失败: {0}")]
    UploadFailed(String),

    #[error("删除失败: {0}")]
    DeleteFailed(String),

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("权限不足")]
    PermissionDenied,

    #[error("参数错误: {0}")]
    InvalidParam(String),
}

impl_into_response!(FileError, {
    FileError::NotFound => StatusCode::NOT_FOUND,
    FileError::AlreadyExists => StatusCode::CONFLICT,
    FileError::UnsupportedType(_) => StatusCode::BAD_REQUEST,
    FileError::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
    FileError::UploadFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    FileError::DeleteFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    FileError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    FileError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    FileError::PermissionDenied => StatusCode::FORBIDDEN,
    FileError::InvalidParam(_) => StatusCode::BAD_REQUEST,
});

/// 文件服务操作结果
pub type FileResult<T> = Result<T, FileError>;

impl From<sqlx::Error> for FileError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("数据库错误: {err:?}");
        Self::DatabaseError(err.to_string())
    }
}

impl From<crate::storage::StorageError> for FileError {
    fn from(err: crate::storage::StorageError) -> Self {
        tracing::error!("存储错误: {err:?}");
        match err {
            crate::storage::StorageError::NotFound(_) => Self::NotFound,
            _ => Self::StorageError(err.to_string()),
        }
    }
}
