//! CTP 服务错误类型定义
//!
//! 统一错误处理，将内部错误映射为 HTTP 响应，支持 axum `IntoResponse`。

use axum::http::StatusCode;
    
use common::impl_into_response;
use thiserror::Error;

/// CTP 服务错误枚举
///
/// 覆盖设备连接、命令执行、参数校验、第三方请求及 Redis 缓存等场景。
#[derive(Error, Debug)]
pub enum CtpError {
    #[error("设备连接失败: {0}")]
    DeviceConnection(String),

    #[error("设备命令执行失败: {0}")]
    DeviceCommand(String),

    #[error("设备离线")]
    DeviceOffline,

    #[error("参数无效")]
    InvalidParams,

    #[error("HTTP请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Redis错误: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("设备不存在: {0}")]
    DeviceNotFound(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl_into_response!(CtpError, {
    CtpError::DeviceConnection(_) => StatusCode::BAD_GATEWAY,
    CtpError::DeviceCommand(_) => StatusCode::BAD_GATEWAY,
    CtpError::DeviceOffline => StatusCode::SERVICE_UNAVAILABLE,
    CtpError::InvalidParams => StatusCode::BAD_REQUEST,
    CtpError::RequestError(_) => StatusCode::BAD_GATEWAY,
    CtpError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    CtpError::DeviceNotFound(_) => StatusCode::NOT_FOUND,
    CtpError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
});

pub type Result<T> = std::result::Result<T, CtpError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    #[test]
    fn test_device_connection_status() {
        let response = CtpError::DeviceConnection("timeout".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_device_command_status() {
        let response = CtpError::DeviceCommand("failed".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_device_offline_status() {
        let response = CtpError::DeviceOffline.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_invalid_params_status() {
        let response = CtpError::InvalidParams.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_device_not_found_status() {
        let response = CtpError::DeviceNotFound("dev01".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_internal_error_status() {
        let response = CtpError::Internal("err".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            CtpError::DeviceConnection("timeout".to_string()).to_string(),
            "设备连接失败: timeout"
        );
        assert_eq!(CtpError::DeviceOffline.to_string(), "设备离线");
        assert_eq!(CtpError::InvalidParams.to_string(), "参数无效");
    }
}
