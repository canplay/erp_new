//! API Gateway 中间件
//!
//! @date 2026-05-01
//! @note JWT 验证后将用户信息注入到请求头，传递给后端服务
//!
//! # 模块结构
//! - `jwt`: JWT 认证中间件（AuthState, JwtClaims, auth_middleware）
//! - `rate_limit`: 限流中间件（RateLimitState, rate_limit_middleware）
//! - `cors`: CORS 配置（cors_layer）
//! - `logging`: 请求日志中间件（logging_middleware）
//! - `security_headers`: 安全响应头中间件（security_headers_middleware）
//! - `operation_log`: 操作日志中间件（operation_log_middleware）

// Re-export from submodules for backward compatibility
mod jwt;
mod rate_limit;
mod cors;
mod logging;
mod security_headers;
mod operation_log;

pub use jwt::{AuthState, JwtClaims, auth_middleware};
pub use rate_limit::rate_limit_middleware;
pub use cors::cors_layer;
pub use cors::CorsConfig;
pub use logging::logging_middleware;
pub use security_headers::security_headers_middleware;
pub use operation_log::operation_log_middleware;

// Keep RateLimitState here since it's already defined in the original
// and used by the rate_limit module

use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};

/// 限流状态 - 支持滑动窗口限流和突发请求
#[derive(Clone)]
pub struct RateLimitState {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    pub max_requests: u32,
    pub window_secs: u64,
    pub burst: u32, // 突发请求允许量
}

impl RateLimitState {
    /// 创建新的限流状态
    ///
    /// # 参数
    /// - `max_requests`: 窗口内最大请求数
    /// - `window_secs`: 时间窗口（秒）
    /// - burst: 突发请求允许量
    #[must_use]
    pub fn new(max_requests: u32, window_secs: u64, burst: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_secs,
            burst,
        }
    }

    /// 检查是否允许请求
    pub fn check_rate_limit(&self, key: &str) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        // 安全修复: 使用 parking_lot::RwLock 的阻塞写锁
        // parking_lot 的锁实现非常高效（使用 futex），不会长时间阻塞线程
        let mut guard = self.requests.write();

        let timestamps = guard.entry(key.to_string()).or_insert_with(Vec::new);
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= self.max_requests as usize {
            return false;
        }

        timestamps.push(now);
        true
    }

    /// 获取剩余请求次数
    #[must_use]
    pub fn remaining(&self, key: &str) -> u32 {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let Some(guard) = self.requests.try_read() else {
            return self.max_requests;
        };

        if let Some(timestamps) = guard.get(key) {
            let valid_count = timestamps
                .iter()
                .filter(|&&t| now.duration_since(t) < window)
                .count();
            return self.max_requests.saturating_sub(valid_count as u32);
        }
        self.max_requests
    }
}
