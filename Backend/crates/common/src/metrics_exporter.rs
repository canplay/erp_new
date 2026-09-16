//! Prometheus metrics exporter
//!
//! Provides a simple `/metrics` endpoint for services.
//! Enabled via the `metrics-exporter` feature flag.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use axum::{routing::get, Router};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
    static ref HTTP_ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);
    static ref SERVICE_START: Instant = Instant::now();
}

/// Increment the request counter
pub fn record_request() {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Increment the error counter
pub fn record_error() {
    HTTP_ERRORS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Generate Prometheus-format metrics output
pub fn prometheus_output() -> String {
    let uptime = SERVICE_START.elapsed().as_secs();
    let requests = HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed);
    let errors = HTTP_ERRORS_TOTAL.load(Ordering::Relaxed);

    format!(
        "# HELP service_uptime_seconds Service uptime in seconds\n\
         # TYPE service_uptime_seconds counter\n\
         service_uptime_seconds {}\n\
         \n\
         # HELP http_requests_total Total HTTP requests\n\
         # TYPE http_requests_total counter\n\
         http_requests_total {}\n\
         \n\
         # HELP http_errors_total Total HTTP errors\n\
         # TYPE http_errors_total counter\n\
         http_errors_total {}\n\
         ",
        uptime, requests, errors,
    )
}

/// Create a Router with the /metrics endpoint
pub fn metrics_router() -> Router {
    async fn metrics_handler() -> String {
        prometheus_output()
    }
    Router::new().route("/metrics" , get(metrics_handler))
}
