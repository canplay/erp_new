//! HTTP 响应构建器
//!
//! 提供统一的 HTTP API 响应辅助函数

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::json;

/// HTTP 响应构建器 trait
pub trait HttpResponse {
    /// 转换为 `IntoResponse`
    fn into_http_response(self) -> impl IntoResponse;
}

/// 创建 200 OK 响应（带数据）
pub fn ok_response<T: Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": data,
        })),
    )
}

/// 创建 201 Created 响应
pub fn created_response<T: Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": data,
        })),
    )
}

/// 创建 204 No Content 响应
#[must_use]
pub fn no_content_response() -> impl IntoResponse {
    (
        StatusCode::NO_CONTENT,
        Json(json!({
            "success": true,
            "message": "操作成功"
        })),
    )
}

/// 创建 400 Bad Request 响应
#[must_use]
pub fn bad_request_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建 401 Unauthorized 响应
#[must_use]
pub fn unauthorized_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建 403 Forbidden 响应
#[must_use]
pub fn forbidden_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建 404 Not Found 响应
#[must_use]
pub fn not_found_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建 409 Conflict 响应
#[must_use]
pub fn conflict_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建 500 Internal Server Error 响应
#[must_use]
pub fn internal_error_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建 429 Too Many Requests 响应
#[must_use]
pub fn rate_limit_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "success": false,
            "error": message,
        })),
    )
}

/// 创建分页响应
#[must_use]
pub fn paginated_response<T: Serialize>(
    list: Vec<T>,
    total: i64,
    page: i32,
    page_size: i32,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": {
                "list": list,
                "total": total,
                "page": page,
                "page_size": page_size,
            }
        })),
    )
}

/// 创建批量操作结果响应
#[must_use]
pub fn batch_result_response(
    success_count: usize,
    fail_count: usize,
    errors: Vec<serde_json::Value>,
) -> impl IntoResponse {
    let status = if fail_count > 0 && success_count > 0 {
        StatusCode::MULTI_STATUS // 207
    } else if fail_count > 0 {
        StatusCode::BAD_REQUEST // 400
    } else {
        StatusCode::OK // 200
    };

    (
        status,
        Json(json!({
            "success": fail_count == 0,
            "message": format!("成功 {} 个，失败 {} 个" , success_count, fail_count),
            "data": {
                "success_count": success_count,
                "fail_count": fail_count,
                "errors": errors,
            }
        })),
    )
}

/// 创建 CSV 下载响应
#[must_use]
pub fn csv_download_response(content: String, filename: &str) -> impl IntoResponse {
    match axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type" , "text/csv; charset=utf-8" )
        .header(
            "Content-Disposition" ,
            format!("attachment; filename={filename}" ),
        )
        .body(axum::body::Body::from(content))
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("csv_download_response 构建失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "构建响应失败" ,
            )
                .into_response()
        }
    }
}

/// 创建成功响应（带消息）
#[must_use]
pub fn success_with_message_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": message,
        })),
    )
}

/// 创建成功响应（带数据数组）
#[must_use]
pub fn list_response<T: Serialize>(items: Vec<T>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": items,
        })),
    )
}

/// 创建成功响应（带 ID）
#[must_use]
pub fn created_with_id_response(id: i64, name: &str) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": {
                "id": id,
                "name": name,
            }
        })),
    )
}

/// 创建成功响应（带消息和数据）
pub fn success_with_data_and_message<T: Serialize>(data: T, message: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": message,
            "data": data,
        })),
    )
}

/// 验证并提取分页参数
#[must_use]
pub fn validate_pagination_params(
    page: Option<i32>,
    page_size: Option<i32>,
    default_page: i32,
    default_page_size: i32,
    min_page: i32,
    max_page_size: i32,
) -> (i32, i32) {
    let p = page.unwrap_or(default_page).max(min_page);
    let size = page_size
        .unwrap_or(default_page_size)
        .clamp(1, max_page_size);
    (p, size)
}

/// 创建统一的 CORS 层
///
/// 处理 "*" 通配符和具体 origins 列表。
///
/// # 用法
/// ```ignore
/// let cors = common::http::cors_layer(&origins);
/// ```
///
/// # 示例
/// ```rust,ignore
/// // 所有 origins 为 "*" 时:
/// cors_layer(&["*".to_string()])
///     .allow_origin(tower_http::cors::Any)
///
/// // 具体 origins 时:
/// cors_layer(&["https://example.com".to_string()])
///     .allow_origin(header_values)
/// ```
pub fn cors_layer(origins: &[String]) -> tower_http::cors::CorsLayer {
    let any = origins.len() == 1 && origins[0] == "*";
    let mut layer = tower_http::cors::CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .expose_headers(tower_http::cors::Any);
    if any {
        layer = layer.allow_origin(tower_http::cors::Any);
    } else {
        let header_values: Vec<_> = origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        layer = layer.allow_origin(header_values);
    }
    layer
}
