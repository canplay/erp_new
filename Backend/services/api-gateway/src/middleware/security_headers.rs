//! 安全响应头中间件
//!
//! 为所有 HTTP 响应添加安全相关的响应头，防止常见 Web 安全漏洞。
//!
//! # 添加的响应头
//! - Strict-Transport-Security: 强制 HTTPS
//! - X-Frame-Options: 防止点击劫持
//! - X-Content-Type-Options: 防止 MIME 类型嗅探
//! - Content-Security-Policy: 限制内容来源
//! - Referrer-Policy: 控制 Referrer 信息
//! - Permissions-Policy: 限制 API 权限

use axum::{
    extract::Request,
    http::{HeaderValue, header::HeaderName},
    middleware::Next,
    response::Response,
};

/// 安全响应头中间件
///
/// 为所有 HTTP 响应添加安全相关的响应头，防止常见 Web 安全漏洞。
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        HeaderName::from_static("strict-transport-security" ),
        HeaderValue::from_static("max-age=31536000; includeSubDomains" ),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options" ),
        HeaderValue::from_static("DENY" ),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options" ),
        HeaderValue::from_static("nosniff" ),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy" ),
        HeaderValue::from_static("default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:" ),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy" ),
        HeaderValue::from_static("strict-origin-when-cross-origin" ),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy" ),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()" ),
    );

    response
}
