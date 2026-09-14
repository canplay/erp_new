//! gRPC 错误中间件
//!
//! 提供 axum 中间件，将 gRPC 调用产生的 `tonic::Status` 错误
//! 统一转换为标准的 HTTP 错误响应。
//!
//! # 使用方式
//!
//! 将此中间件添加到 axum 路由链中：
//! ```ignore
//! .layer(axum::middleware::from_fn(grpc_error_handler_middleware))
//! ```
//!
//! 当 handler 通过 `?` 传播 `tonic::Status` 时，
//! From<tonic::Status> for `AppError` 会自动将其转换，
//! 再由 `AppError` 的 `IntoResponse` 转化为 HTTP 错误体。

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::routes::helpers::json_grpc_error;
use common::AppError;

/// gRPC 错误处理中间件
///
/// 包装所有上游 handler，确保 gRPC 调用产生的错误被正确记录
/// 并转换为标准 HTTP 错误响应。
///
/// 此中间件返回 `Result<Response, AppError>`，当 handler 内部
/// 通过 `?` 操作符传播 `tonic::Status` 时，它会被自动转换为
/// AppError，再由 axum 通过 `IntoResponse` 序列化为 JSON 错误体。
pub async fn grpc_error_handler_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let trace_id = request
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(request).await;

    if response.status().is_server_error() {
        tracing::error!(
            "[gRPC] 上游服务返回服务端错误 | trace_id={} method={} path={} status={}",
            trace_id,
            method,
            path,
            response.status().as_u16()
        );
    } else if response.status().is_client_error() {
        tracing::warn!(
            "[gRPC] 上游服务返回客户端错误 | trace_id={} method={} path={} status={}",
            trace_id,
            method,
            path,
            response.status().as_u16()
        );
    }

    Ok(response)
}

/// 构造一个 gRPC 服务不可用的错误响应。
///
/// 当 api-gateway 无法连接到下游 gRPC 服务时使用。
/// 将 tonic 连接/传输错误转换为 HTTP 503 响应。
#[must_use]
pub fn grpc_unavailable_response(service_name: &str, err: &tonic::Status) -> Response {
    let (status_code, message) = if err.code() == tonic::Code::Unavailable {
        (StatusCode::SERVICE_UNAVAILABLE, format!("服务不可用: {service_name}"))
    } else {
        (
            StatusCode::BAD_GATEWAY,
            format!("上游服务错误 ({}): {}", service_name, err.message()),
        )
    };

    let body = json_grpc_error(message);

    (status_code, body).into_response()
}

/// 便捷宏：在 axum handler 中调用 gRPC 客户端并自动转换错误。
///
/// 当 gRPC 调用返回 `tonic::Status` 错误时，使用此宏可以自动
/// 将错误转换为 `AppError` 并提前返回。
///
/// # 示例
/// ```ignore
/// async fn get_user_handler(
///     State(state): State<Arc<AppState>>,
///     Path(user_id): Path<i64>,
/// ) -> Result<Json<Value>, AppError> {
///     let mut client = state.grpc_clients.write().user_service
///         .clone()
///         .ok_or_else(|| AppError::ServiceUnavailable("user-service 不可用".into()))?;
///
///     let response = grpc_call!(client.get_user(user_id))?;
///     Ok(Json(serde_json::to_value(&response).unwrap()))
/// }
/// ```
#[macro_export]
macro_rules! grpc_call {
    ($expr:expr) => {
        match $expr.await {
            Ok(val) => val,
            Err(status) => return Err(Into::<AppError>::into(status)),
        }
    };
}
