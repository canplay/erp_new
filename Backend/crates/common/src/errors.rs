//! 统一错误类型定义

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// 统一错误响应生成
/// 所有服务错误都应通过此函数转换为 HTTP 响应，确保格式一致。
pub fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    let body = Json(json!({
        "success": false,
        "error": message.into()
    }));
    (status, body).into_response()
}

/// 为服务级错误类型实现 `IntoResponse` 的辅助宏。
///
/// # 用法
/// ```ignore
/// impl_into_response!(MyError, {
///     MyError::NotFound => StatusCode::NOT_FOUND,
///     MyError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
///     MyError::InvalidParam(_) => StatusCode::BAD_REQUEST,
/// });
/// ```
#[macro_export]
macro_rules! impl_into_response {
    ($err_type:ty, { $($variant:pat => $status:expr),+ $(,)? }) => {
        impl axum::response::IntoResponse for $err_type {
            fn into_response(self) -> axum::response::Response {
                let status = match self {
                    $( $variant => $status, )+
                };
                $crate::errors::error_response(status, self.to_string())
            }
        }
    };
}

/// 错误码（用于客户端处理）
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ============ 认证授权 (2xxxx) ============
    #[error("认证失败: {0}")]
    Unauthorized(String),

    #[error("禁止访问: {0}")]
    Forbidden(String),

    #[error("Token 已过期")]
    TokenExpired,

    #[error("Token 无效: {0}")]
    TokenInvalid(String),

    // ============ 参数校验 (3xxxx) ============
    #[error("请求参数错误: {0}")]
    BadRequest(String),

    #[error("用户名不能为空")]
    InvalidUsername,

    #[error("用户名格式不正确（长度 3-50 位字母数字下划线）")]
    InvalidUsernameFormat,

    #[error("密码长度至少 8 位")]
    InvalidPassword,

    #[error("邮箱格式不正确")]
    InvalidEmail,

    #[error("手机号格式不正确")]
    InvalidPhone,

    #[error("无效的角色值: {0}")]
    InvalidRole(String),

    #[error("无效的状态值: {0}")]
    InvalidStatus(i32),

    // ============ 业务逻辑 (4xxxx) ============
    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("用户不存在")]
    UserNotFound,

    #[error("用户已存在: {0}")]
    UserAlreadyExists(String),

    #[error("角色不存在: {0}")]
    RoleNotFound(String),

    #[error("角色已存在: {0}")]
    RoleAlreadyExists(String),

    #[error("部门不存在")]
    DepartmentNotFound,

    // ============ 系统错误 (1xxxx) ============
    #[error("内部服务器错误: {0}")]
    Internal(String),

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("Token 错误: {0}")]
    Token(String),

    #[error("限流: {0}")]
    RateLimit(String),

    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),

    #[error("CSRF 验证失败: {0}")]
    CsrfError(String),

    /// 操作失败（带数量统计）
    #[error("部分操作失败: 成功 {success_count} 个, 失败 {fail_count} 个")]
    PartialFailure {
        success_count: usize,
        fail_count: usize,
    },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message) = match &self {
            // 认证授权
            Self::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                20001,
                msg.clone(),
            ),
            Self::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                20002,
                msg.clone(),
            ),
            Self::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                20003,
                self.to_string(),
            ),
            Self::TokenInvalid(msg) => (
                StatusCode::UNAUTHORIZED,
                20004,
                msg.clone(),
            ),

            // 参数校验
            Self::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                30001,
                msg.clone(),
            ),
            Self::InvalidUsername => (
                StatusCode::BAD_REQUEST,
                30002,
                self.to_string(),
            ),
            Self::InvalidUsernameFormat => (
                StatusCode::BAD_REQUEST,
                30003,
                self.to_string(),
            ),
            Self::InvalidPassword => (
                StatusCode::BAD_REQUEST,
                30004,
                self.to_string(),
            ),
            Self::InvalidEmail => (
                StatusCode::BAD_REQUEST,
                30005,
                self.to_string(),
            ),
            Self::InvalidPhone => (
                StatusCode::BAD_REQUEST,
                30006,
                self.to_string(),
            ),
            Self::InvalidRole(role) => (
                StatusCode::BAD_REQUEST,
                30007,
                format!("无效的角色值: {role}"),
            ),
            Self::InvalidStatus(status) => (
                StatusCode::BAD_REQUEST,
                30008,
                format!("无效的状态值: {status}"),
            ),

            // 业务逻辑
            Self::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                40001,
                msg.clone(),
            ),
            Self::UserNotFound => (
                StatusCode::NOT_FOUND,
                40001,
                self.to_string(),
            ),
            Self::UserAlreadyExists(username) => (
                StatusCode::CONFLICT,
                40002,
                format!("用户已存在: {username}"),
            ),
            Self::RoleNotFound(role) => (
                StatusCode::NOT_FOUND,
                40003,
                format!("角色不存在: {role}"),
            ),
            Self::RoleAlreadyExists(role) => (
                StatusCode::CONFLICT,
                40004,
                format!("角色已存在: {role}"),
            ),
            Self::DepartmentNotFound => (
                StatusCode::NOT_FOUND,
                40005,
                self.to_string(),
            ),

            // 系统错误
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10001,
                msg.clone(),
            ),
            Self::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10002,
                format!("数据库错误: {e}"),
            ),
            Self::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10003,
                msg.clone(),
            ),
            Self::Token(msg) => (
                StatusCode::UNAUTHORIZED,
                20004,
                msg.clone(),
            ),
            Self::RateLimit(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                10004,
                msg.clone(),
            ),
            Self::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                10005,
                msg.clone(),
            ),

            // CSRF 错误
            Self::CsrfError(msg) => (
                StatusCode::FORBIDDEN,
                20005,
                msg.clone(),
            ),

            // 部分失败
            Self::PartialFailure {
                success_count,
                fail_count,
            } => (
                StatusCode::MULTI_STATUS, // 207 Multi-Status
                30010,
                format!("部分操作失败: 成功 {success_count} 个, 失败 {fail_count} 个"),
            ),
        };

        let body = Json(json!({
            "success": false,
            "code": error_code,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// 应用结果类型
pub type AppResult<T> = Result<T, AppError>;

impl From<AppError> for std::io::Error {
    fn from(e: AppError) -> Self {
        Self::other(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Token(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::Internal(format!("HTTP client error: {e}"))
    }
}
