//! gRPC 错误映射模块
//!
//! 提供 `tonic::Status` ↔ `AppError` 的双向转换，以及辅助函数，
//! 确保 gRPC 边界上的错误处理统一且类型安全。

use crate::errors::AppError;
use tonic::Status;

/// gRPC 结果类型
pub type GrpcResult<T> = Result<T, Status>;

// ============================================================
// tonic::Status → AppError（客户端方向：api-gateway 接收下游错误）
// ============================================================

/// 将 `tonic::Status` 的 gRPC 状态码映射到对应的 `AppError` 变体。
///
/// # 映射规则
/// | gRPC Code               | HTTP Status  | AppError Variant      |
/// |--------------------------|--------------|-----------------------|
/// | INVALID_ARGUMENT         | 400          | BadRequest            |
/// | NOT_FOUND                | 404          | NotFound              |
/// | ALREADY_EXISTS           | 409          | AlreadyExists         |
/// | PERMISSION_DENIED        | 403          | Forbidden             |
/// | UNAUTHENTICATED          | 401          | Unauthorized          |
/// | RESOURCE_EXHAUSTED       | 429          | RateLimit             |
/// | FAILED_PRECONDITION      | 400          | BadRequest            |
/// | DEADLINE_EXCEEDED        | 504          | ServiceUnavailable    |
/// | UNAVAILABLE              | 503          | ServiceUnavailable    |
/// | 其他                     | 500          | Internal              |
fn map_tonic_code_to_app_error(code: tonic::Code, message: String) -> AppError {
    match code {
        tonic::Code::InvalidArgument => AppError::BadRequest(message),
        tonic::Code::NotFound => AppError::NotFound(message),
        tonic::Code::AlreadyExists => AppError::UserAlreadyExists(message),
        tonic::Code::PermissionDenied => AppError::Forbidden(message),
        tonic::Code::Unauthenticated => AppError::Unauthorized(message),
        tonic::Code::ResourceExhausted => AppError::RateLimit(message),
        tonic::Code::FailedPrecondition => AppError::BadRequest(message),
        tonic::Code::OutOfRange => AppError::BadRequest(message),
        tonic::Code::DeadlineExceeded => AppError::ServiceUnavailable(message),
        tonic::Code::Unavailable => AppError::ServiceUnavailable(message),
        tonic::Code::Cancelled => AppError::NotFound(format!("请求已取消: {message}")),
        tonic::Code::Aborted => AppError::Internal(format!("操作已中止: {message}")),
        tonic::Code::Unimplemented => {
            AppError::Internal(format!("不支持的操作: {message}"))
        }
        tonic::Code::DataLoss => AppError::Internal(format!("数据丢失: {message}")),
        // Unknown / Internal / 其他未显式匹配的
        _ => AppError::Internal(format!("gRPC 错误: {message}")),
    }
}

impl From<tonic::Status> for AppError {
    fn from(status: tonic::Status) -> Self {
        let message = status.message().to_string();
        let code = status.code();
        map_tonic_code_to_app_error(code, message)
    }
}

impl From<&tonic::Status> for AppError {
    fn from(status: &tonic::Status) -> Self {
        let message = status.message().to_string();
        let code = status.code();
        map_tonic_code_to_app_error(code, message)
    }
}

// ============================================================
// AppError → tonic::Status（服务端方向：微服务返回错误给 gRPC 调用方）
// ============================================================

/// 将 `AppError` 映射到对应的 `tonic::Status`。
///
/// # 映射规则
/// | AppError Variant          | gRPC Code              |
/// |---------------------------|------------------------|
/// | Unauthorized / Token*     | UNAUTHENTICATED        |
/// | Forbidden / CsrfError     | PERMISSION_DENIED      |
/// | BadRequest / Invalid*     | INVALID_ARGUMENT       |
/// | NotFound / UserNotFound   | NOT_FOUND              |
/// | UserAlreadyExists / Role* | ALREADY_EXISTS         |
/// | RateLimit                 | RESOURCE_EXHAUSTED     |
/// | ServiceUnavailable        | UNAVAILABLE            |
/// | 其他 / Internal / Database| INTERNAL               |
fn map_app_error_to_tonic_status(err: &AppError) -> tonic::Status {
    match err {
        // 认证授权
        AppError::Unauthorized(msg) => Status::unauthenticated(msg),
        AppError::Forbidden(msg) => Status::permission_denied(msg),
        AppError::TokenExpired => Status::unauthenticated("Token 已过期"),
        AppError::TokenInvalid(msg) => Status::unauthenticated(format!("Token 无效: {msg}")),

        // 参数校验
        AppError::BadRequest(msg) => Status::invalid_argument(msg),
        AppError::InvalidUsername => Status::invalid_argument("用户名不能为空"),
        AppError::InvalidUsernameFormat => {
            Status::invalid_argument("用户名格式不正确（长度 3-50 位字母数字下划线）")
        }
        AppError::InvalidPassword => Status::invalid_argument("密码长度至少 8 位"),
        AppError::InvalidEmail => Status::invalid_argument("邮箱格式不正确"),
        AppError::InvalidPhone => Status::invalid_argument("手机号格式不正确"),
        AppError::InvalidRole(role) => Status::invalid_argument(format!("无效的角色值: {role}")),
        AppError::InvalidStatus(status) => {
            Status::invalid_argument(format!("无效的状态值: {status}"))
        }

        // 业务逻辑
        AppError::NotFound(msg) => Status::not_found(msg),
        AppError::UserNotFound => Status::not_found("用户不存在"),
        AppError::UserAlreadyExists(username) => {
            Status::already_exists(format!("用户已存在: {username}"))
        }
        AppError::RoleNotFound(role) => Status::not_found(format!("角色不存在: {role}")),
        AppError::RoleAlreadyExists(role) => {
            Status::already_exists(format!("角色已存在: {role}"))
        }
        AppError::DepartmentNotFound => Status::not_found("部门不存在"),

        // 系统错误
        AppError::Internal(msg) => Status::internal(msg),
        AppError::Database(e) => Status::internal(format!("数据库错误: {e}")),
        AppError::Config(msg) => Status::internal(format!("配置错误: {msg}")),
        AppError::Token(msg) => Status::internal(format!("Token 错误: {msg}")),
        AppError::RateLimit(msg) => Status::resource_exhausted(msg),
        AppError::ServiceUnavailable(msg) => Status::unavailable(msg),
        AppError::CsrfError(msg) => Status::permission_denied(msg),

        // 部分失败
        AppError::PartialFailure {
            success_count,
            fail_count,
        } => Status::internal(format!(
            "部分操作失败: 成功 {success_count} 个, 失败 {fail_count} 个"
        )),
    }
}

impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        map_app_error_to_tonic_status(&err)
    }
}

impl From<&AppError> for tonic::Status {
    fn from(err: &AppError) -> Self {
        map_app_error_to_tonic_status(err)
    }
}

/// `AppError` 的便捷扩展方法，用于 gRPC 上下文。
pub trait GrpcAppErrorExt {
    /// 转换为 `tonic::Status（gRPC` 返回错误）
    fn into_grpc_status(self) -> tonic::Status;
}

impl GrpcAppErrorExt for AppError {
    fn into_grpc_status(self) -> tonic::Status {
        self.into()
    }
}

/// `tonic::Status` 的便捷扩展方法，用于 HTTP 上下文。
pub trait GrpcStatusExt {
    /// 转换为 AppError（HTTP 服务处理错误）
    fn into_app_error(self) -> AppError;
}

impl GrpcStatusExt for tonic::Status {
    fn into_app_error(self) -> AppError {
        self.into()
    }
}

/// 将实现了 `Into<AppError>` 的类型转换为 `tonic::Status`。
///
/// 用于 gRPC 服务端 handler 中，将业务错误统一转换为 gRPC 状态码。
/// 这允许使用 `map_err(|e| e.into_grpc_status())` 替代
/// 手动构造 `Status::internal(...)` 或 `Status::not_found(...)`。
///
/// # 示例
/// ```ignore
/// use common::IntoTonicStatus;
///
/// async fn get_user(&self, req: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
///     let user = self.repo.find_by_id(req.into_inner().user_id)
///         .await
///         .map_err(|e| e.into_grpc_status())?;
///     // ...
/// }
/// ```
pub trait IntoTonicStatus {
    /// 转换为 `tonic::Status`。
    fn into_grpc_status(self) -> tonic::Status;
}

impl<T: Into<AppError>> IntoTonicStatus for T {
    fn into_grpc_status(self) -> tonic::Status {
        let app_err: AppError = self.into();
        app_err.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    #[test]
    fn test_status_to_app_error_invalid_argument() {
        let status = Status::invalid_argument("username is empty");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::BadRequest(msg) if msg == "username is empty"));
    }

    #[test]
    fn test_status_to_app_error_not_found() {
        let status = Status::not_found("user not found");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::NotFound(msg) if msg == "user not found"));
    }

    #[test]
    fn test_status_to_app_error_unauthenticated() {
        let status = Status::unauthenticated("invalid token");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::Unauthorized(msg) if msg == "invalid token"));
    }

    #[test]
    fn test_status_to_app_error_permission_denied() {
        let status = Status::permission_denied("no access");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::Forbidden(msg) if msg == "no access"));
    }

    #[test]
    fn test_status_to_app_error_already_exists() {
        let status = Status::already_exists("user exists");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::UserAlreadyExists(msg) if msg == "user exists"));
    }

    #[test]
    fn test_status_to_app_error_resource_exhausted() {
        let status = Status::resource_exhausted("rate limit");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::RateLimit(msg) if msg == "rate limit"));
    }

    #[test]
    fn test_status_to_app_error_unavailable() {
        let status = Status::unavailable("service down");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::ServiceUnavailable(msg) if msg == "service down"));
    }

    #[test]
    fn test_status_to_app_error_internal() {
        let status = Status::internal("db error");
        let err: AppError = status.into();
        assert!(matches!(err, AppError::Internal(msg) if msg == "gRPC 错误: db error"));
    }

    #[test]
    fn test_app_error_to_status_unauthorized() {
        let err = AppError::Unauthorized("bad credentials".into());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
        assert_eq!(status.message(), "bad credentials");
    }

    #[test]
    fn test_app_error_to_status_not_found() {
        let err = AppError::NotFound("user".into());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert_eq!(status.message(), "user");
    }

    #[test]
    fn test_app_error_to_status_invalid_argument() {
        let err = AppError::BadRequest("invalid param".into());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert_eq!(status.message(), "invalid param");
    }

    #[test]
    fn test_app_error_to_status_internal() {
        let err = AppError::Internal("server error".into());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert_eq!(status.message(), "server error");
    }

    #[test]
    fn test_app_error_to_status_rate_limit() {
        let err = AppError::RateLimit("too fast".into());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::ResourceExhausted);
    }

    #[test]
    fn test_app_error_to_status_service_unavailable() {
        let err = AppError::ServiceUnavailable("down".into());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unavailable);
    }

    #[test]
    fn test_status_ext_trait() {
        let status = Status::not_found("test");
        let err = status.into_app_error();
        assert!(matches!(err, AppError::NotFound(msg) if msg == "test"));
    }

    #[test]
    fn test_app_error_ext_trait() {
        let err = AppError::NotFound("test".into());
        let status = <AppError as GrpcAppErrorExt>::into_grpc_status(err);
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[test]
    fn test_ref_from_impl() {
        let status = Status::invalid_argument("bad");
        let err = AppError::from(&status);
        assert!(matches!(err, AppError::BadRequest(msg) if msg == "bad"));
    }

    #[test]
    fn test_ref_app_error_to_status() {
        let err = AppError::Forbidden("no".into());
        let status = tonic::Status::from(&err);
        assert_eq!(status.code(), tonic::Code::PermissionDenied);
    }
}
