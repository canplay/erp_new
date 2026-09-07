//! Prometheus 指标模块
//!
//! 提供 API 性能监控、请求计数等指标
//! @date 2026-05-06

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

/// 请求计数指标
#[derive(Debug)]
pub struct RequestCounter {
    pub method: String,
    pub path: String,
    pub status: String,
    count: AtomicU64,
}

impl RequestCounter {
    #[must_use]
    pub fn new(method: &str, path: &str, status: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            status: status.to_string(),
            count: AtomicU64::new(0),
        }
    }

    pub fn inc(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

/// 请求延迟指标
#[derive(Debug)]
pub struct RequestLatency {
    pub method: String,
    pub path: String,
    sum: AtomicU64,
    count: AtomicU64,
}

impl RequestLatency {
    #[must_use]
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    pub fn observe(&self, duration_ms: u64) {
        self.sum.fetch_add(duration_ms, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

/// Prometheus 指标收集器
#[derive(Debug)]
pub struct MetricsCollector {
    requests: Arc<RwLock<Vec<Arc<RequestCounter>>>>,
    latencies: Arc<RwLock<Vec<Arc<RequestLatency>>>>,
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    active_requests: AtomicU64,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    #[must_use]
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(Vec::new())),
            latencies: Arc::new(RwLock::new(Vec::new())),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            active_requests: AtomicU64::new(0),
        }
    }

    /// 记录请求开始
    pub fn request_start(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.active_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录请求结束
    pub fn request_end(&self, method: &str, path: &str, status: u16, duration_ms: u64) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);

        if status >= 400 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        // 获取或创建请求计数器
        let status_str = status.to_string();
        let counter = self.get_or_create_counter(method, path, &status_str);
        counter.inc();

        // 记录延迟
        let latency = self.get_or_create_latency(method, path);
        latency.observe(duration_ms);
    }

    fn get_or_create_counter(&self, method: &str, path: &str, status: &str) -> Arc<RequestCounter> {
        let mut requests = self.requests.write();

        // 尝试查找现有计数器
        for counter in requests.iter() {
            if counter.method == method && counter.path == path && counter.status == status {
                return Arc::clone(counter);
            }
        }

        // 创建新计数器
        let counter = Arc::new(RequestCounter::new(method, path, status));
        requests.push(Arc::clone(&counter));
        counter
    }

    fn get_or_create_latency(&self, method: &str, path: &str) -> Arc<RequestLatency> {
        let mut latencies = self.latencies.write();

        // 尝试查找现有延迟记录
        for latency in latencies.iter() {
            if latency.method == method && latency.path == path {
                return Arc::clone(latency);
            }
        }

        // 创建新延迟记录
        let latency = Arc::new(RequestLatency::new(method, path));
        latencies.push(Arc::clone(&latency));
        latency
    }

    /// 生成 Prometheus 格式的指标输出
    pub fn render_prometheus(&self) -> String {
        let mut output = String::new();

        // 帮助文本
        output.push_str("# HELP http_requests_total Total number of HTTP requests\n");
        output.push_str("# TYPE http_requests_total counter\n");

        // 请求计数器
        let requests = self.requests.read();
        for counter in requests.iter() {
            let labels = format!(
                "method=\"{}\",path=\"{}\",status=\"{}\"",
                escape_label(&counter.method),
                escape_label(&counter.path),
                escape_label(&counter.status)
            );
            output.push_str(&format!(
                "http_requests_total{{{}}} {}\n",
                labels,
                counter.get()
            ));
        }

        output.push_str(
            "\n# HELP http_request_duration_milliseconds HTTP request duration in milliseconds\n",
        );
        output.push_str("# TYPE http_request_duration_milliseconds summary\n");

        // 延迟记录
        let latencies = self.latencies.read();
        for latency in latencies.iter() {
            let labels = format!(
                "method=\"{}\",path=\"{}\"",
                escape_label(&latency.method),
                escape_label(&latency.path)
            );
            let sum = latency.sum();
            let count = latency.count();
            output.push_str(&format!(
                "http_request_duration_milliseconds_sum{{{labels}}} {sum}\n"
            ));
            output.push_str(&format!(
                "http_request_duration_milliseconds_count{{{labels}}} {count}\n"
            ));
        }

        output.push_str("\n# HELP http_requests_active Current number of active requests\n");
        output.push_str("# TYPE http_requests_active gauge\n");
        output.push_str(&format!(
            "http_requests_active {}\n",
            self.active_requests.load(Ordering::Relaxed)
        ));

        output.push_str("\n# HELP http_requests_total_count Total number of HTTP requests\n");
        output.push_str("# TYPE http_requests_total_count counter\n");
        output.push_str(&format!(
            "http_requests_total_count {}\n",
            self.total_requests.load(Ordering::Relaxed)
        ));

        output.push_str("\n# HELP http_errors_total Total number of HTTP errors\n");
        output.push_str("# TYPE http_errors_total counter\n");
        output.push_str(&format!(
            "http_errors_total {}\n",
            self.total_errors.load(Ordering::Relaxed)
        ));

        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 转义标签值中的特殊字符
fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub static METRICS: LazyLock<Arc<MetricsCollector>> = LazyLock::new(|| Arc::new(MetricsCollector::new()));

/// 导出指标收集器
pub fn metrics_collector() -> Arc<MetricsCollector> {
    METRICS.clone()
}
