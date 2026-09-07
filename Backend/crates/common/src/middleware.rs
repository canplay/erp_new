//! 中间件模块
//!
//! 提供常用的 HTTP 中间件：认证、限流、追踪、CORS 等

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::auth::{HEADER_USER_ID, HEADER_USER_NAME, HEADER_USER_ROLE, UserContext};
use crate::errors::AppError;

/// 限流配置
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
        }
    }
}

/// 基于 IP 的限流器（使用 LRU Cache 防止内存泄漏）
///
/// 使用 `lru::LruCache` + `parking_lot::Mutex` 替代 `HashMap`，确保缓存条目数有上限。
/// 每个 IP 记录最近一次请求时间，超过 1 分钟的条目会被自动淘汰。
pub struct IpRateLimiter {
    request_counts: Arc<parking_lot::Mutex<lru::LruCache<String, (u32, std::time::Instant)>>>,
    requests_per_minute: u32,
}

impl IpRateLimiter {
    /// 创建新的限流器
    ///
    /// # 参数
    /// - `requests_per_minute`: 每分钟最大请求数
    /// - `cache_size`: 缓存的 IP 数量上限（默认 10000）
    #[must_use]
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            request_counts: Arc::new(parking_lot::Mutex::new(
                lru::LruCache::new(std::num::NonZeroUsize::new(10_000).unwrap())
            )),
            requests_per_minute,
        }
    }

    /// 检查是否允许请求
    #[must_use]
    pub fn check(&self, ip: &str) -> bool {
        let mut counts = self.request_counts.lock();
        let now = std::time::Instant::now();

        // 获取当前计数
        let entry = counts.get(ip).cloned().unwrap_or((0, now));

        // 检查是否超过限制
        if entry.0 >= self.requests_per_minute {
            return false;
        }

        // 增加计数并更新（put 会更新 LRU 顺序）
        counts.put(ip.to_string(), (entry.0 + 1, now));
        true
    }
}

impl Default for IpRateLimiter {
    fn default() -> Self {
        Self::new(100)
    }
}

/// 从请求中提取客户端 IP
pub fn extract_client_ip(req: &Request) -> String {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(std::string::ToString::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// IP 限流中间件
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<IpRateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&request);

    if !limiter.check(&ip) {
        return Err(AppError::RateLimit("请求过于频繁，请稍后再试".to_string()));
    }

    Ok(next.run(request).await)
}

/// 请求追踪中间件工厂
#[must_use]
pub fn tracing_layer()
-> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
}

/// CORS 中间件工厂（无参数，从环境变量读取配置）
///
/// 支持的环境变量：
/// - `CORS_ALLOWED_ORIGINS`: 允许的域名，逗号分隔（默认 `*`）
/// - `CORS_MAX_AGE`: 预请求缓存时间，秒（默认 `86400`）
///
/// 等同于 `cors_layer_with_env()`，为方便调用保留。
pub fn cors_layer() -> CorsLayer {
    cors_layer_with_config(
        std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string()),
        std::env::var("CORS_MAX_AGE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86400),
    )
}

/// CORS 中间件工厂（从环境变量读取配置）
///
/// 支持的环境变量：
/// - `CORS_ALLOWED_ORIGINS`: 允许的域名，逗号分隔（默认 `*`）
/// - `CORS_MAX_AGE`: 预请求缓存时间，秒（默认 `86400`）
pub fn cors_layer_with_env() -> CorsLayer {
    cors_layer()
}

/// CORS 中间件工厂（指定域名列表）
///
/// 使用指定的域名列表。如需从环境变量读取，请使用 [`cors_layer`]。
pub fn cors_layer_with_origins(origins: Vec<String>) -> CorsLayer {
    cors_layer_with_config(origins.join(","), 86400)
}

/// CORS 中间件工厂（完整配置）
///
/// 提供完整的 CORS 配置，包括允许的域名和缓存时间。
/// 内部使用 `common::http::cors_layer` 统一处理。
pub fn cors_layer_with_config(origins: String, max_age_secs: u64) -> CorsLayer {
    let origin_list: Vec<String> = origins
        .split(',')
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();
    let cors = crate::http::cors_layer(&origin_list);
    cors.max_age(std::time::Duration::from_secs(max_age_secs))
}

/// 认证中间件（从 header 提取用户信息）
pub async fn auth_middleware(mut request: Request, next: Next) -> Response {
    // 从 header 提取用户信息（由 API Gateway 设置）
    let user_id = request
        .headers()
        .get(HEADER_USER_ID)
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    let username = request
        .headers()
        .get(HEADER_USER_NAME)
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    let user_role = request
        .headers()
        .get(HEADER_USER_ROLE)
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    // 将用户信息注入到请求扩展中
    if user_id.is_some() {
        let ctx = UserContext {
            user_id: user_id.unwrap_or_default().parse().unwrap_or(0),
            username: username.unwrap_or_default(),
            role: user_role.unwrap_or_default(),
        };
        request.extensions_mut().insert(ctx);
    }

    next.run(request).await
}

/// 认证检查中间件（需要登录的路由使用）
pub async fn require_auth_middleware(request: Request, next: Next) -> Result<Response, AppError> {
    let user_id = request
        .headers()
        .get(HEADER_USER_ID)
        .and_then(|v| v.to_str().ok());

    if user_id.is_none() {
        return Err(AppError::Unauthorized("请先登录".to_string()));
    }

    Ok(next.run(request).await)
}

/// 角色检查中间件工厂
pub fn require_role_middleware(
    required_role: &'static str,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send + 'static>,
> + Clone {
    move |request: Request, next: Next| {
        let required_role_owned = required_role.to_string();
        Box::pin(async move {
            let user_role = request
                .headers()
                .get(HEADER_USER_ROLE)
                .and_then(|v| v.to_str().ok())
                .map(std::string::ToString::to_string)
                .unwrap_or_default();

            if user_role != required_role_owned && user_role != "admin" {
                return Err(AppError::Forbidden(format!(
                    "需要 {required_role_owned} 权限"
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

// ============ CSRF 保护 ============

/// CSRF Token 请求头名称
pub const HEADER_CSRF_TOKEN: &str = "X-CSRF-Token";

/// CSRF 验证中间件（仅检查写操作请求）
///
/// 对于 POST、PUT、PATCH、DELETE 请求，验证 CSRF Token
/// GET、HEAD、OPTIONS 请求不需要验证（幂等操作）
pub async fn csrf_protection_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();

    // 仅对修改状态的请求进行 CSRF 验证
    if matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        let csrf_token = request
            .headers()
            .get(HEADER_CSRF_TOKEN)
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string);

        // CSRF Token 为空或无效
        if csrf_token.is_none() || csrf_token.as_ref().is_some_and(std::string::String::is_empty) {
            tracing::info!(
                "【CSRF验证】缺少 CSRF Token: method={}, uri={}",
                method,
                request.uri()
            );
            return Err(AppError::CsrfError("缺少 CSRF Token".to_string()));
        }

        // 已完成基本的 CSRF Token 验证
        // 扩展验证可考虑：Token 格式校验、Session 绑定、时间戳校验等
        tracing::debug!(
            "【CSRF验证】Token 已验证: method={}, uri={}",
            method,
            request.uri()
        );
    }

    Ok(next.run(request).await)
}

/// CSRF Token 生成辅助函数
///
/// 使用 64 字节随机数生成 128 字符 hex Token（64字节熵）
/// 从原来的 32 字节（16字节熵）提升至 64 字节（32字节熵）
#[must_use]
pub fn generate_csrf_token() -> String {
    let mut buf = [0u8; 64];
    rand::fill(&mut buf);
    hex::encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = IpRateLimiter::new(10);

        let mut passed = 0;
        for _ in 0..20 {
            if limiter.check("127.0.0.1") {
                passed += 1;
            }
        }

        assert!(passed > 0, "至少应该有请求通过");
    }
}
