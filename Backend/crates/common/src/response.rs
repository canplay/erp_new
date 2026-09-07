//! 统一响应格式

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// API 统一响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub const fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    /// 创建成功响应带消息
    pub fn success_with_message(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.to_string()),
            error: None,
        }
    }

    /// 创建错误响应
    #[must_use]
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(message.to_string()),
        }
    }
}

/// 创建成功响应 (无数据)
impl ApiResponse<()> {
    #[must_use]
    pub fn ok() -> Self {
        Self {
            success: true,
            data: None,
            message: Some("操作成功".to_string()),
            error: None,
        }
    }
}

/// 辅助函数：返回错误响应（带 HTTP 状态码）
impl ApiResponse<()> {
    /// 返回 500 Internal Server Error 响应
    #[must_use]
    pub fn error_response(msg: &str) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Self::error(msg)),
        )
            .into_response()
    }

    /// 返回 404 Not Found 响应
    #[must_use]
    pub fn not_found_response(msg: &str) -> Response {
        (
            StatusCode::NOT_FOUND,
            Json(Self::error(msg)),
        )
            .into_response()
    }

    /// 返回 200 OK 响应（带消息）
    #[must_use]
    pub fn ok_response(msg: &str) -> Response {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                data: None,
                message: Some(msg.to_string()),
                error: None,
            }),
        )
            .into_response()
    }
}
