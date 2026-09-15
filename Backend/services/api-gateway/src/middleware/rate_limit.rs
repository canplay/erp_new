//! 限流中间件
//!
//! 基于滑动窗口的 per-IP 限流，支持突发请求。
//!
//! # 功能
//! - 滑动窗口限流算法
//! - 基于 IP + User-Agent + Path + Method 的组合 key
//! - 返回 X-RateLimit-Remaining / X-RateLimit-Limit 头
//! - 支持突发请求 (burst)

use crate::RateLimitState;

use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// 限流中间件
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    let rate_limit_state = request.extensions().get::<RateLimitState>().cloned();

    let state = match rate_limit_state {
        Some(s) => s,
        None => return next.run(request).await,
    };

    // M10修复: 限流key使用IP+User-Agent哈希,防伪造
    // P5修复: 使用 enhanced_logging::get_client_ip 统一获取客户端真实IP（支持多header降级）
    let client_ip = crate::enhanced_logging::get_client_ip(&request)
        .unwrap_or_else(|| "unknown".to_string());
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let client_key = format!(
        "{}:{}:{}:{}",
        client_ip.split(',').next().unwrap_or(&client_ip).trim(),
        user_agent.chars().take(32).collect::<String>(),
        request.uri().path(),
        request.method().as_str()
    );

    if state.check_rate_limit(&client_key) {
        let remaining = state.remaining(&client_key);
        let mut response = next.run(request).await;

        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Remaining",
            HeaderValue::from_str(&remaining.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );
        headers.insert(
            "X-RateLimit-Limit",
            HeaderValue::from_str(&state.max_requests.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );

        response
    } else {
        tracing::warn!("限流触发: {client_key}");
        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .body(r#"{"error":"请求过于频繁，请稍后重试"}"#.into())
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "构造 429 响应失败");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            })
    }
}
