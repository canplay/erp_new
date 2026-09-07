//! XLT 服务错误类型定义
//!
//! 统一错误处理，将内部错误映射为 HTTP 响应。

use axum::http::StatusCode;
use common::impl_into_response;
use thiserror::Error;

/// XLT 服务错误枚举
#[derive(Error, Debug)]
pub enum XltError {
    #[error("MQTT连接失败: {0}")]
    MqttConnection(String),

    #[error("MQTT消息发布失败: {0}")]
    MqttPublish(String),

    #[error("车辆在场内未找到")]
    VehicleNotFound,

    #[error("计费计算错误: {0}")]
    BillingError(String),

    #[error("参数无效")]
    InvalidParams,

    #[error("HTTP请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Redis错误: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl_into_response!(XltError, {
    XltError::MqttConnection(_) => StatusCode::BAD_GATEWAY,
    XltError::MqttPublish(_) => StatusCode::BAD_GATEWAY,
    XltError::VehicleNotFound => StatusCode::NOT_FOUND,
    XltError::BillingError(_) => StatusCode::UNPROCESSABLE_ENTITY,
    XltError::InvalidParams => StatusCode::BAD_REQUEST,
    XltError::RequestError(_) => StatusCode::BAD_GATEWAY,
    XltError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    XltError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
});

pub type Result<T> = std::result::Result<T, XltError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn test_mqtt_connection_status() {
        let response = XltError::MqttConnection("refused".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_mqtt_publish_status() {
        let response = XltError::MqttPublish("timeout".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_vehicle_not_found_status() {
        let response = XltError::VehicleNotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_billing_error_status() {
        let response = XltError::BillingError("calc".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_invalid_params_status() {
        let response = XltError::InvalidParams.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_internal_error_status() {
        let response = XltError::Internal("err".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            XltError::MqttConnection("refused".to_string()).to_string(),
            "MQTT连接失败: refused"
        );
        assert_eq!(XltError::VehicleNotFound.to_string(), "车辆在场内未找到");
        assert_eq!(XltError::InvalidParams.to_string(), "参数无效");
    }
}
