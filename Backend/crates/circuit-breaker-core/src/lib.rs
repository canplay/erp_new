//! 熔断器核心库
//!
//! 提供基于失败率的熔断、自动恢复和状态监控

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭状态 - 正常请求
    Closed,
    /// 打开状态 - 请求被拒绝
    Open,
    /// 半开状态 - 尝试请求
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "closed" ),
            Self::Open => write!(f, "open" ),
            Self::HalfOpen => write!(f, "half_open" ),
        }
    }
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败率阈值（百分比），超过此值将打开熔断器
    pub failure_threshold: f64,
    /// 半开后允许的试探请求数
    pub half_open_requests: u32,
    /// 打开熔断器后的恢复超时（秒）
    pub recovery_timeout_secs: u64,
    /// 统计窗口大小（秒）
    pub window_size_secs: u64,
    /// 最小请求数（低于此数量的请求不触发熔断）
    pub min_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 50.0,
            half_open_requests: 3,
            recovery_timeout_secs: 30,
            window_size_secs: 60,
            min_requests: 10,
        }
    }
}

/// 熔断器
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    /// 熔断器状态
    state: Arc<Mutex<CircuitState>>,
    /// 状态统计
    stats: Arc<Mutex<HashMap<String, CircuitStats>>>,
    /// 打开状态开始时间
    opened_at: Arc<Mutex<Option<Instant>>>,
}

/// 熔断器统计信息
#[derive(Debug, Clone)]
pub struct CircuitStats {
    /// 成功请求数
    pub success_count: u32,
    /// 失败请求数
    pub failure_count: u32,
    /// 半开状态成功数
    pub half_open_success: u32,
    /// 半开状态失败数
    pub half_open_failure: u32,
    /// 统计窗口开始时间
    pub window_start: Instant,
}

impl Default for CircuitStats {
    fn default() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            half_open_success: 0,
            half_open_failure: 0,
            window_start: Instant::now(),
        }
    }
}

impl CircuitStats {
    /// 计算失败率
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.0;
        }
        (f64::from(self.failure_count) / f64::from(total)) * 100.0
    }

    /// 计算总请求数
    #[must_use]
    pub const fn total_requests(&self) -> u32 {
        self.success_count + self.failure_count
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.success_count = 0;
        self.failure_count = 0;
        self.half_open_success = 0;
        self.half_open_failure = 0;
        self.window_start = Instant::now();
    }

    /// 记录成功
    pub const fn record_success(&mut self) {
        self.success_count += 1;
    }

    /// 记录失败
    pub const fn record_failure(&mut self) {
        self.failure_count += 1;
    }

    /// 记录半开成功
    pub const fn record_half_open_success(&mut self) {
        self.half_open_success += 1;
    }

    /// 记录半开失败
    pub const fn record_half_open_failure(&mut self) {
        self.half_open_failure += 1;
    }
}

impl CircuitBreaker {
    /// 创建新的熔断器
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            stats: Arc::new(Mutex::new(HashMap::new())),
            opened_at: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取当前状态
    #[must_use]
    pub fn state(&self) -> CircuitState {
        *self.state.lock()
    }

    /// 检查是否允许请求
    #[must_use]
    pub fn allow(&self, key: &str) -> bool {
        let mut state = self.state.lock();

        match *state {
            CircuitState::Closed => {
                // 检查是否需要重置统计窗口
                self.check_window_reset(key);
                true
            }
            CircuitState::Open => {
                // 检查是否超时可以进入半开状态
                if self.should_enter_half_open() {
                    *state = CircuitState::HalfOpen;
                    tracing::info!("Circuit breaker for '{key}' entering half-open state" );
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// 记录请求结果
    pub fn record(&self, key: &str, success: bool) {
        let mut state = self.state.lock();
        let mut stats = self.stats.lock();

        let current_state = *state;
        let stat = stats.entry(key.to_string()).or_default();

        match current_state {
            CircuitState::Closed => {
                if success {
                    stat.record_success();
                } else {
                    stat.record_failure();
                }
                // 检查是否需要打开熔断器
                self.check_failure_threshold(stat, key, &mut state);
            }
            CircuitState::HalfOpen => {
                if success {
                    stat.record_half_open_success();
                } else {
                    stat.record_half_open_failure();
                }
                // 检查半开状态结果
                self.check_half_open_result(stat, key, &mut state);
            }
            CircuitState::Open => {
                // 不记录统计数据
            }
        }
    }

    /// 手动打开熔断器
    pub fn force_open(&self, key: &str) {
        let mut state = self.state.lock();
        *state = CircuitState::Open;
        *self.opened_at.lock() = Some(Instant::now());

        let mut stats = self.stats.lock();
        if let Some(stat) = stats.get_mut(key) {
            stat.reset();
        }

        tracing::warn!("Circuit breaker for '{key}' force opened" );
    }

    /// 手动重置熔断器
    pub fn reset(&self, key: &str) {
        let mut state = self.state.lock();
        *state = CircuitState::Closed;
        *self.opened_at.lock() = None;

        let mut stats = self.stats.lock();
        if let Some(stat) = stats.get_mut(key) {
            stat.reset();
        }

        tracing::info!("Circuit breaker for '{key}' reset to closed state" );
    }

    /// 获取统计信息
    #[must_use]
    pub fn get_stats(&self, key: &str) -> Option<CircuitStats> {
        let stats = self.stats.lock();
        stats.get(key).cloned()
    }

    /// 获取所有统计信息
    #[must_use]
    pub fn get_all_stats(&self) -> HashMap<String, CircuitStats> {
        let stats = self.stats.lock();
        stats.clone()
    }

    /// 检查窗口是否需要重置
    fn check_window_reset(&self, key: &str) {
        let mut stats = self.stats.lock();
        if let Some(stat) = stats.get_mut(key) {
            let elapsed = stat.window_start.elapsed().as_secs();
            if elapsed > self.config.window_size_secs {
                stat.reset();
                tracing::debug!("Circuit breaker stats window reset for '{key}'" );
            }
        }
    }

    /// 检查失败率是否超过阈值
    fn check_failure_threshold(
        &self,
        stat: &mut CircuitStats,
        key: &str,
        state: &mut CircuitState,
    ) {
        let total = stat.total_requests();
        if total < self.config.min_requests {
            return;
        }

        let rate = stat.failure_rate();
        if rate >= self.config.failure_threshold {
            *state = CircuitState::Open;
            *self.opened_at.lock() = Some(Instant::now());
            stat.reset();
            tracing::warn!(
                "Circuit breaker for '{}' opened due to failure rate {:.2}% (threshold: {:.2}%)" ,
                key,
                rate,
                self.config.failure_threshold
            );
        }
    }

    /// 检查半开状态的结果
    fn check_half_open_result(&self, stat: &mut CircuitStats, key: &str, state: &mut CircuitState) {
        let total = stat.half_open_success + stat.half_open_failure;

        if stat.half_open_failure > 0 {
            // 半开状态有失败，回到打开状态
            *state = CircuitState::Open;
            *self.opened_at.lock() = Some(Instant::now());
            stat.reset();
            tracing::warn!(
                "Circuit breaker for '{key}' returned to open from half-open due to failure"
            );
        } else if total >= self.config.half_open_requests
            && stat.half_open_success >= self.config.half_open_requests
        {
            // 半开状态全部成功，关闭熔断器
            *state = CircuitState::Closed;
            *self.opened_at.lock() = None;
            stat.reset();
            tracing::info!(
                "Circuit breaker for '{key}' closed after successful half-open requests"
            );
        }
    }

    /// 检查是否应该进入半开状态
    fn should_enter_half_open(&self) -> bool {
        let opened_at = self.opened_at.lock();
        if let Some(at) = *opened_at {
            let elapsed = at.elapsed().as_secs();
            elapsed >= self.config.recovery_timeout_secs
        } else {
            true
        }
    }
}

/// 熔断器结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerResult {
    /// 是否允许
    pub allowed: bool,
    /// 当前状态
    pub state: String,
    /// 剩余重试时间（秒）
    pub retry_after_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow("test" ));
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 50.0,
            min_requests: 5,
            ..Default::default()
        };

        let cb = CircuitBreaker::new(config);

        // 模拟失败率达到阈值
        for _ in 0..6 {
            cb.record("test" , false);
        }

        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow("test" ));
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        cb.force_open("test" );
        assert_eq!(cb.state(), CircuitState::Open);

        // 手动重置以测试半开逻辑
        cb.reset("test" );
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_stats() {
        let mut stats = CircuitStats::default();
        assert_eq!(stats.failure_rate(), 0.0);

        stats.record_failure();
        stats.record_success();
        assert_eq!(stats.failure_rate(), 50.0);

        stats.record_success();
        stats.record_success();
        assert_eq!(stats.failure_rate(), 25.0);
    }
}
