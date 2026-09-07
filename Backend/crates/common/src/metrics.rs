//! 基础指标收集模块
//!
//! 提供轻量级的请求计数、错误计数和响应时间记录能力。
//! 不依赖外部监控库，使用原子操作实现线程安全计数器。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 端点指标（按 method:path 细分）
#[derive(Debug, Clone, Default)]
pub struct EndpointMetrics {
    pub total: u64,
    pub success: u64,
    pub error: u64,
    pub total_response_time_us: u64,
}

/// 指标收集器
#[derive(Debug)]
pub struct MetricsCollector {
    /// 总请求数
    total_requests: AtomicU64,
    /// 成功请求数
    success_requests: AtomicU64,
    /// 失败请求数（HTTP 4xx/5xx）
    error_requests: AtomicU64,
    /// 累计响应时间（微秒）
    total_response_time_us: AtomicU64,
    /// 当前正在处理的请求数
    active_requests: AtomicU64,
    /// 按端点细分的指标（key: method:path）
    by_endpoint: parking_lot::RwLock<indexmap::IndexMap<String, EndpointMetrics>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// 创建新的指标收集器
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            success_requests: AtomicU64::new(0),
            error_requests: AtomicU64::new(0),
            total_response_time_us: AtomicU64::new(0),
            active_requests: AtomicU64::new(0),
            by_endpoint: parking_lot::RwLock::new(
                indexmap::IndexMap::new()
            ),
        }
    }

    /// 记录请求开始，返回一个记录器用于后续计时
    pub fn start_request(&self) -> RequestTimer<'_> {
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        RequestTimer {
            collector: self,
            start: Instant::now(),
        }
    }

    /// 直接记录一次请求（无需计时器）
    pub fn record_request(&self, is_error: bool, duration_us: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.error_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.success_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.total_response_time_us.fetch_add(duration_us, Ordering::Relaxed);
    }

    /// 获取总请求数
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// 获取成功请求数
    pub fn success_requests(&self) -> u64 {
        self.success_requests.load(Ordering::Relaxed)
    }

    /// 获取失败请求数
    pub fn error_requests(&self) -> u64 {
        self.error_requests.load(Ordering::Relaxed)
    }

    /// 获取平均响应时间（微秒）
    pub fn avg_response_time_us(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let sum = self.total_response_time_us.load(Ordering::Relaxed);
        sum as f64 / total as f64
    }

    /// 获取当前活跃请求数
    pub fn active_requests(&self) -> u64 {
        self.active_requests.load(Ordering::Relaxed)
    }

    /// 获取错误率（百分比）
    pub fn error_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let errors = self.error_requests.load(Ordering::Relaxed);
        (errors as f64 / total as f64) * 100.0
    }

    /// 获取按端点细分的指标快照
    #[must_use]
    pub fn endpoint_metrics(&self) -> Vec<(String, EndpointMetrics)> {
        let map = self.by_endpoint.read();
        map.iter()
            .map(|(key, metrics)| (key.clone(), metrics.clone()))
            .collect()
    }

    /// 记录按端点细分的指标（内部方法，由中间件调用）
    pub fn record_by_endpoint(&self, endpoint_key: &str, is_error: bool, duration_us: u64) {
        let mut map = self.by_endpoint.write();
        let metrics = map.entry(endpoint_key.to_string()).or_insert_with(EndpointMetrics::default);
        metrics.total += 1;
        if is_error {
            metrics.error += 1;
        } else {
            metrics.success += 1;
        }
        metrics.total_response_time_us += duration_us;
    }

    /// 获取指标快照
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests(),
            success_requests: self.success_requests(),
            error_requests: self.error_requests(),
            active_requests: self.active_requests(),
            avg_response_time_us: self.avg_response_time_us(),
            error_rate: self.error_rate(),
        }
    }

    /// 获取 Prometheus 格式的指标数据
    ///
    /// # 返回
    /// Prometheus metrics 格式的字符串
    #[must_use]
    pub fn prometheus_metrics(&self) -> String {
        let snap = self.snapshot();
        let mut output = String::new();
        output.push_str("# HELP http_requests_total Total number of HTTP requests\n");
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!("http_requests_total {}\n", snap.total_requests));
        output.push_str("# HELP http_requests_success Total number of successful HTTP requests\n");
        output.push_str("# TYPE http_requests_success counter\n");
        output.push_str(&format!("http_requests_success {}\n", snap.success_requests));
        output.push_str("# HELP http_requests_error Total number of failed HTTP requests\n");
        output.push_str("# TYPE http_requests_error counter\n");
        output.push_str(&format!("http_requests_error {}\n", snap.error_requests));
        output.push_str("# HELP http_requests_active Current number of active requests\n");
        output.push_str("# TYPE http_requests_active gauge\n");
        output.push_str(&format!("http_requests_active {}\n", snap.active_requests));
        output.push_str("# HELP http_request_duration_avg_us Average response time in microseconds\n");
        output.push_str("# TYPE http_request_duration_avg_us gauge\n");
        output.push_str(&format!("http_request_duration_avg_us {}\n", snap.avg_response_time_us as u64));
        output.push_str("# HELP http_request_error_rate Error rate percentage\n");
        output.push_str("# TYPE http_request_error_rate gauge\n");
        output.push_str(&format!("http_request_error_rate {}\n", snap.error_rate));
        output
    }
}

/// 请求计时器 — 在 drop 时自动记录耗时
pub struct RequestTimer<'a> {
    collector: &'a MetricsCollector,
    start: Instant,
}

impl RequestTimer<'_> {
    /// 标记请求为错误并记录耗时
    pub fn mark_error(self) {
        let duration = self.start.elapsed().as_micros() as u64;
        self.collector.record_request(true, duration);
    }

    /// 标记请求为成功并记录耗时
    pub fn mark_success(self) {
        let duration = self.start.elapsed().as_micros() as u64;
        self.collector.record_request(false, duration);
    }
}

impl Drop for RequestTimer<'_> {
    fn drop(&mut self) {
        self.collector.active_requests.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 指标快照
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub success_requests: u64,
    pub error_requests: u64,
    pub active_requests: u64,
    pub avg_response_time_us: f64,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_basic() {
        let c = MetricsCollector::new();
        assert_eq!(c.total_requests(), 0);

        c.record_request(false, 100);
        assert_eq!(c.total_requests(), 1);
        assert_eq!(c.success_requests(), 1);
    }

    #[test]
    fn test_request_timer() {
        let c = MetricsCollector::new();
        {
            let timer = c.start_request();
            timer.mark_success();
        }
        assert_eq!(c.total_requests(), 1);
        assert_eq!(c.success_requests(), 1);
        assert_eq!(c.active_requests(), 0);
    }

    #[test]
    fn test_snapshot() {
        let c = MetricsCollector::new();
        c.record_request(false, 100);
        c.record_request(true, 200);

        let snap = c.snapshot();
        assert_eq!(snap.total_requests, 2);
        assert_eq!(snap.error_rate, 50.0);
    }
}
