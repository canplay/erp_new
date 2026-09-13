//! API Gateway 中间件
//!
//! 提供 JWT 认证、租户解析、配额检查等中间功能

pub mod jwt;
pub mod tenant;
pub mod quota;
pub mod rate_limit;
pub mod cors;
pub mod logging;
pub mod security_headers;
pub mod operation_log;

pub use jwt::{AuthState, JwtClaims, auth_middleware};
pub use tenant::tenant_extraction_middleware;
pub use quota::{quota_enforcement_middleware, usage_recorder_middleware};
pub use rate_limit::rate_limit_middleware;
pub use cors::{CorsConfig, cors_layer};
pub use logging::logging_middleware;
pub use security_headers::security_headers_middleware;
pub use operation_log::operation_log_middleware;

use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};

/// 限流状态
#[derive(Clone)]
pub struct RateLimitState {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    pub max_requests: u32,
    pub window_secs: u64,
    pub burst: u32,
}

impl RateLimitState {
    pub fn new(max_requests: u32, window_secs: u64, burst: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_secs,
            burst,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);
        let mut guard = self.requests.write();
        let timestamps = guard.entry(key.to_string()).or_insert_with(Vec::new);
        timestamps.retain(|&t| now.duration_since(t) < window);
        if timestamps.len() >= self.max_requests as usize {
            return false;
        }
        timestamps.push(now);
        true
    }

    /// 返回指定 key 的剩余请求次数
    pub fn remaining(&self, key: &str) -> u32 {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);
        let guard = self.requests.read();
        let timestamps = guard.get(key).map(|v| v.as_slice()).unwrap_or(&[]);
        let active = timestamps.iter().filter(|&t| now.duration_since(*t) < window).count();
        self.max_requests.saturating_sub(active as u32)
    }
}
