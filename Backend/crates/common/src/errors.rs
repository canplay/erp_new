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
    #[error("认证失败: {0}" )]
    Unauthorized(String),

    #[error("禁止访问: {0}" )]
    Forbidden(String),

    #[error("Token 已过期" )]
    TokenExpired,

    #[error("Token 无效: {0}" )]
    TokenInvalid(String),

    // ============ 参数校验 (3xxxx) ============
    #[error("请求参数错误: {0}" )]
    BadRequest(String),

    #[error("用户名不能为空" )]
    InvalidUsername,

    #[error("用户名格式不正确（长度 3-50 位字母数字下划线）" )]
    InvalidUsernameFormat,

    #[error("密码长度至少 8 位" )]
    InvalidPassword,

    #[error("邮箱格式不正确" )]
    InvalidEmail,

    #[error("手机号格式不正确" )]
    InvalidPhone,

    #[error("无效的角色值: {0}" )]
    InvalidRole(String),

    #[error("无效的状态值: {0}" )]
    InvalidStatus(i32),

    // ============ 业务逻辑 (4xxxx) ============
    #[error("资源未找到: {0}" )]
    NotFound(String),

    #[error("用户不存在" )]
    UserNotFound,

    #[error("用户已存在: {0}" )]
    UserAlreadyExists(String),

    #[error("角色不存在: {0}" )]
    RoleNotFound(String),

    #[error("角色已存在: {0}" )]
    RoleAlreadyExists(String),

    #[error("部门不存在" )]
    DepartmentNotFound,

    // ============ API 密钥服务 (41xxx) ============
    #[error("密钥不存在: {0}" )]
    ApiKeyNotFound(String),

    #[error("密钥已过期" )]
    ApiKeyExpired,

    #[error("密钥已被禁用" )]
    ApiKeyDisabled,

    #[error("密钥验证失败" )]
    ApiKeyInvalid,

    #[error("IP地址不被允许" )]
    ApiKeyIpNotAllowed,

    #[error("超过请求限制" )]
    ApiKeyRateLimitExceeded,

    // ============ 审计服务 (42xxx) ============
    #[error("日志不存在" )]
    AuditNotFound,

    #[error("删除失败: {0}" )]
    AuditDeleteFailed(String),

    // ============ 计费服务 (43xxx) ============
    #[error("计费计划不存在: {0}" )]
    BillingPlanNotFound(String),

    #[error("订阅不存在: {0}" )]
    BillingSubscriptionNotFound(String),

    #[error("发票不存在: {0}" )]
    BillingInvoiceNotFound(String),

    #[error("计费计算错误: {0}" )]
    BillingCalculationError(String),

    #[error("用量记录不存在: {0}" )]
    BillingUsageNotFound(String),

    // ============ CTP 锁平台服务 (44xxx) ============
    #[error("设备连接失败: {0}" )]
    CtpDeviceConnection(String),

    #[error("设备命令执行失败: {0}" )]
    CtpDeviceCommand(String),

    #[error("设备离线" )]
    CtpDeviceOffline,

    #[error("设备不存在: {0}" )]
    CtpDeviceNotFound(String),

    #[error("CTP参数无效" )]
    CtpInvalidParams,

    #[error("CTP内部错误: {0}" )]
    CtpInternal(String),

    // ============ 文件服务 (45xxx) ============
    #[error("文件不存在" )]
    FileNotFound,

    #[error("文件已存在" )]
    FileAlreadyExists,

    #[error("文件类型不支持: {0}" )]
    FileUnsupportedType(String),

    #[error("文件大小超出限制: {0}" )]
    FileTooLarge(String),

    #[error("文件上传失败: {0}" )]
    FileUploadFailed(String),

    #[error("文件删除失败: {0}" )]
    FileDeleteFailed(String),

    #[error("存储错误: {0}" )]
    FileStorageError(String),

    #[error("文件权限不足" )]
    FilePermissionDenied,

    // ============ 海康服务 (46xxx) ============
    #[error("海康令牌获取失败" )]
    HikTokenError,

    #[error("海康方法无效" )]
    HikInvalidMethod,

    #[error("第三方API调用失败: {0}" )]
    HikApiError(String),

    #[error("海康参数错误" )]
    HikInvalidParams,

    // ============ 车牌识别服务 (47xxx) ============
    #[error("LPR数据库错误: {0}" )]
    LprDatabaseError(String),

    #[error("LPR内部错误: {0}" )]
    LprInternal(String),

    // ============ XLT 信路通服务 (48xxx) ============
    #[error("MQTT连接失败: {0}" )]
    XltMqttConnection(String),

    #[error("MQTT消息发布失败: {0}" )]
    XltMqttPublish(String),

    #[error("车辆在场内未找到" )]
    XltVehicleNotFound,

    #[error("XLT参数无效" )]
    XltInvalidParams,

    // ============ 支付服务 (49xxx) ============
    #[error("支付订单不存在" )]
    PayOrderNotFound,

    #[error("支付参数错误" )]
    PayInvalidParams,

    #[error("支付内部错误: {0}" )]
    PayInternalError(String),

    // ============ 清理服务 (50xxx) ============
    #[error("清理资源不存在" )]
    CleanNotFound,

    #[error("清理删除失败: {0}" )]
    CleanDeleteFailed(String),

    // ============ 租户服务 (51xxx) ============
    #[error("租户不存在" )]
    TenantNotFound,

    #[error("租户已存在" )]
    TenantAlreadyExists,

    // ============ 工作流服务 (52xxx) ============
    #[error("工作流不存在: {0}" )]
    WorkflowNotFound(String),

    #[error("工作流未发布: {0}" )]
    WorkflowNotPublished(String),

    #[error("工作流实例不存在: {0}" )]
    InstanceNotFound(String),

    #[error("工作流节点不存在: {0}" )]
    NodeNotFound(String),

    #[error("无效的状态转换: {0}" )]
    InvalidStateTransition(String),

    #[error("执行失败: {0}" )]
    ExecutionFailed(String),

    // ============ 系统错误 (1xxxx) ============
    #[error("内部服务器错误: {0}" )]
    Internal(String),

    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("配置错误: {0}" )]
    Config(String),

    #[error("Token 错误: {0}" )]
    Token(String),

    #[error("限流: {0}" )]
    RateLimit(String),

    #[error("服务不可用: {0}" )]
    ServiceUnavailable(String),

    #[error("CSRF 验证失败: {0}" )]
    CsrfError(String),

    #[error("Redis错误: {0}" )]
    Redis(redis::RedisError),

    #[error("HTTP客户端错误: {0}" )]
    HttpError(reqwest::Error),

    /// 操作失败（带数量统计）
    #[error("部分操作失败: 成功 {success_count} 个, 失败 {fail_count} 个" )]
    PartialFailure {
        success_count: usize,
        fail_count: usize,
    },

    // ============ 通用业务错误 ============
    #[error("无效参数: {0}" )]
    InvalidParam(String),

    #[error("未授权访问" )]
    UnauthorizedAccess,

    #[error("禁止操作" )]
    ForbiddenOperation,

    #[error("数据库错误: {0}" )]
    DatabaseError(String),

    /// 功能未实现（桩 handler 占位）
    #[error("功能未实现: {0}" )]
    NotImplemented(String),

    // ============ CMS 服务 (53xxx) ============
    #[error("文章不存在" )]
    ArticleNotFound,

    #[error("文章代码已存在" )]
    ArticleAlreadyExists,

    #[error("分类不存在" )]
    CategoryNotFound,

    #[error("分类代码已存在" )]
    CategoryAlreadyExists,

    #[error("存在子分类或关联文章，无法删除" )]
    CategoryHasChildren,

    // ============ 反馈服务 (54xxx) ============
    #[error("反馈不存在" )]
    FeedbackNotFound,

    // ============ 消息服务 (55xxx) ============
    #[error("SMTP 连接失败: {0}" )]
    EmailConnection(String),

    #[error("SMTP 认证失败: {0}" )]
    EmailAuthentication(String),

    #[error("邮件发送失败: {0}" )]
    EmailSend(String),

    #[error("邮件模板渲染失败: {0}" )]
    EmailTemplate(String),

    // ============ 租户流程 (56xxx) ============
    #[error("租户入驻流程失败: {0}" )]
    TenantOnboarding(String),

    #[error("租户注销流程失败: {0}" )]
    TenantOffboarding(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message) = match &self {
            // 认证授权
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, 20001, msg.clone()),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, 20002, msg.clone()),
            Self::TokenExpired => (StatusCode::UNAUTHORIZED, 20003, self.to_string()),
            Self::TokenInvalid(msg) => (StatusCode::UNAUTHORIZED, 20004, msg.clone()),

            // 参数校验
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, 30001, msg.clone()),
            Self::InvalidUsername => (StatusCode::BAD_REQUEST, 30002, self.to_string()),
            Self::InvalidUsernameFormat => (StatusCode::BAD_REQUEST, 30003, self.to_string()),
            Self::InvalidPassword => (StatusCode::BAD_REQUEST, 30004, self.to_string()),
            Self::InvalidEmail => (StatusCode::BAD_REQUEST, 30005, self.to_string()),
            Self::InvalidPhone => (StatusCode::BAD_REQUEST, 30006, self.to_string()),
            Self::InvalidRole(role) => (
                StatusCode::BAD_REQUEST,
                30007,
                format!("无效的角色值: {role}" ),
            ),
            Self::InvalidStatus(status) => (
                StatusCode::BAD_REQUEST,
                30008,
                format!("无效的状态值: {status}" ),
            ),

            // 业务逻辑
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, 40001, msg.clone()),
            Self::UserNotFound => (StatusCode::NOT_FOUND, 40001, self.to_string()),
            Self::UserAlreadyExists(username) => (
                StatusCode::CONFLICT,
                40002,
                format!("用户已存在: {username}" ),
            ),
            Self::RoleNotFound(role) => {
                (StatusCode::NOT_FOUND, 40003, format!("角色不存在: {role}" ))
            }
            Self::RoleAlreadyExists(role) => {
                (StatusCode::CONFLICT, 40004, format!("角色已存在: {role}" ))
            }
            Self::DepartmentNotFound => (StatusCode::NOT_FOUND, 40005, self.to_string()),

            // API 密钥服务
            Self::ApiKeyNotFound(_) => (StatusCode::NOT_FOUND, 41001, self.to_string()),
            Self::ApiKeyExpired => (StatusCode::UNAUTHORIZED, 41002, self.to_string()),
            Self::ApiKeyDisabled => (StatusCode::FORBIDDEN, 41003, self.to_string()),
            Self::ApiKeyInvalid => (StatusCode::UNAUTHORIZED, 41004, self.to_string()),
            Self::ApiKeyIpNotAllowed => (StatusCode::FORBIDDEN, 41005, self.to_string()),
            Self::ApiKeyRateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, 41006, self.to_string())
            }

            // 审计服务
            Self::AuditNotFound => (StatusCode::NOT_FOUND, 42001, self.to_string()),
            Self::AuditDeleteFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 42002, self.to_string())
            }

            // 计费服务
            Self::BillingPlanNotFound(_) => (StatusCode::NOT_FOUND, 43001, self.to_string()),
            Self::BillingSubscriptionNotFound(_) => {
                (StatusCode::NOT_FOUND, 43002, self.to_string())
            }
            Self::BillingInvoiceNotFound(_) => (StatusCode::NOT_FOUND, 43003, self.to_string()),
            Self::BillingCalculationError(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, 43004, self.to_string())
            }
            Self::BillingUsageNotFound(_) => (StatusCode::NOT_FOUND, 43005, self.to_string()),

            // CTP 锁平台服务
            Self::CtpDeviceConnection(_) => (StatusCode::BAD_GATEWAY, 44001, self.to_string()),
            Self::CtpDeviceCommand(_) => (StatusCode::BAD_GATEWAY, 44002, self.to_string()),
            Self::CtpDeviceOffline => (StatusCode::SERVICE_UNAVAILABLE, 44003, self.to_string()),
            Self::CtpDeviceNotFound(_) => (StatusCode::NOT_FOUND, 44004, self.to_string()),
            Self::CtpInvalidParams => (StatusCode::BAD_REQUEST, 44005, self.to_string()),
            Self::CtpInternal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 44006, self.to_string()),

            // 文件服务
            Self::FileNotFound => (StatusCode::NOT_FOUND, 45001, self.to_string()),
            Self::FileAlreadyExists => (StatusCode::CONFLICT, 45002, self.to_string()),
            Self::FileUnsupportedType(_) => (StatusCode::BAD_REQUEST, 45003, self.to_string()),
            Self::FileTooLarge(_) => (StatusCode::PAYLOAD_TOO_LARGE, 45004, self.to_string()),
            Self::FileUploadFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 45005, self.to_string())
            }
            Self::FileDeleteFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 45006, self.to_string())
            }
            Self::FileStorageError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 45007, self.to_string())
            }
            Self::FilePermissionDenied => (StatusCode::FORBIDDEN, 45008, self.to_string()),

            // 海康服务
            Self::HikTokenError => (StatusCode::UNAUTHORIZED, 46001, self.to_string()),
            Self::HikInvalidMethod => (StatusCode::BAD_REQUEST, 46002, self.to_string()),
            Self::HikApiError(_) => (StatusCode::BAD_GATEWAY, 46003, self.to_string()),
            Self::HikInvalidParams => (StatusCode::BAD_REQUEST, 46004, self.to_string()),

            // 车牌识别服务
            Self::LprDatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 47001, self.to_string())
            }
            Self::LprInternal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 47002, self.to_string()),

            // XLT 信路通服务
            Self::XltMqttConnection(_) => (StatusCode::BAD_GATEWAY, 48001, self.to_string()),
            Self::XltMqttPublish(_) => (StatusCode::BAD_GATEWAY, 48002, self.to_string()),
            Self::XltVehicleNotFound => (StatusCode::NOT_FOUND, 48003, self.to_string()),
            Self::XltInvalidParams => (StatusCode::BAD_REQUEST, 48004, self.to_string()),

            // 支付服务
            Self::PayOrderNotFound => (StatusCode::NOT_FOUND, 49001, self.to_string()),
            Self::PayInvalidParams => (StatusCode::BAD_REQUEST, 49002, self.to_string()),
            Self::PayInternalError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 49003, self.to_string())
            }

            // 清理服务
            Self::CleanNotFound => (StatusCode::NOT_FOUND, 50001, self.to_string()),
            Self::CleanDeleteFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 50002, self.to_string())
            }

            // 租户服务
            Self::TenantNotFound => (StatusCode::NOT_FOUND, 51001, self.to_string()),
            Self::TenantAlreadyExists => (StatusCode::CONFLICT, 51002, self.to_string()),

            // 工作流服务
            Self::WorkflowNotFound(_) => (StatusCode::NOT_FOUND, 52001, self.to_string()),
            Self::WorkflowNotPublished(_) => (StatusCode::BAD_REQUEST, 52002, self.to_string()),
            Self::InstanceNotFound(_) => (StatusCode::NOT_FOUND, 52003, self.to_string()),
            Self::NodeNotFound(_) => (StatusCode::NOT_FOUND, 52004, self.to_string()),
            Self::InvalidStateTransition(_) => (StatusCode::BAD_REQUEST, 52005, self.to_string()),
            Self::ExecutionFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 52006, self.to_string())
            }

            // 系统错误
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 10001, msg.clone()),
            Self::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10002,
                format!("数据库错误: {e}" ),
            ),
            Self::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 10003, msg.clone()),
            Self::Token(msg) => (StatusCode::UNAUTHORIZED, 20004, msg.clone()),
            Self::RateLimit(msg) => (StatusCode::TOO_MANY_REQUESTS, 10004, msg.clone()),
            Self::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, 10005, msg.clone()),

            // CSRF 错误
            Self::CsrfError(msg) => (StatusCode::FORBIDDEN, 20005, msg.clone()),

            // Redis 错误
            Self::Redis(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10006,
                format!("Redis错误: {e}" ),
            ),

            // HTTP 客户端错误
            Self::HttpError(e) => (
                StatusCode::BAD_GATEWAY,
                10007,
                format!("HTTP客户端错误: {e}" ),
            ),

            // 部分失败
            Self::PartialFailure {
                success_count,
                fail_count,
            } => (
                StatusCode::MULTI_STATUS, // 207 Multi-Status
                30010,
                format!("部分操作失败: 成功 {success_count} 个, 失败 {fail_count} 个" ),
            ),

            // 通用业务错误
            Self::InvalidParam(msg) => (StatusCode::BAD_REQUEST, 30011, format!("无效参数: {msg}" )),
            Self::UnauthorizedAccess => (StatusCode::UNAUTHORIZED, 20006, self.to_string()),
            Self::ForbiddenOperation => (StatusCode::FORBIDDEN, 20007, self.to_string()),
            Self::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10002,
                format!("数据库错误: {msg}" ),
            ),
            Self::NotImplemented(msg) => (
                StatusCode::NOT_IMPLEMENTED, // 501
                30012,
                format!("功能未实现: {msg}" ),
            ),

            // CMS 服务
            Self::ArticleNotFound => (StatusCode::NOT_FOUND, 53001, self.to_string()),
            Self::ArticleAlreadyExists => (StatusCode::CONFLICT, 53002, self.to_string()),
            Self::CategoryNotFound => (StatusCode::NOT_FOUND, 53003, self.to_string()),
            Self::CategoryAlreadyExists => (StatusCode::CONFLICT, 53004, self.to_string()),
            Self::CategoryHasChildren => (StatusCode::BAD_REQUEST, 53005, self.to_string()),

            // 反馈服务
            Self::FeedbackNotFound => (StatusCode::NOT_FOUND, 54001, self.to_string()),

            // 消息服务
            Self::EmailConnection(_) => (StatusCode::BAD_GATEWAY, 55001, self.to_string()),
            Self::EmailAuthentication(_) => (StatusCode::UNAUTHORIZED, 55002, self.to_string()),
            Self::EmailSend(_) => (StatusCode::INTERNAL_SERVER_ERROR, 55003, self.to_string()),
            Self::EmailTemplate(_) => (StatusCode::INTERNAL_SERVER_ERROR, 55004, self.to_string()),

            // 租户流程
            Self::TenantOnboarding(_) => (StatusCode::INTERNAL_SERVER_ERROR, 56001, self.to_string()),
            Self::TenantOffboarding(_) => (StatusCode::INTERNAL_SERVER_ERROR, 56002, self.to_string()),
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

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        Self::Redis(e)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e)
    }
}

/// 将文件服务的 StorageError 转换为 AppError
/// 注意：此函数需要由使用方实现，因为 StorageError 是 file-service 的本地类型
pub fn map_storage_error(err: impl ToString, is_not_found: bool) -> AppError {
    if is_not_found {
        AppError::FileNotFound
    } else {
        AppError::FileStorageError(err.to_string())
    }
}
