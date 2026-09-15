use std::sync::atomic::{AtomicU64, Ordering};

pub static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
pub static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn increment_request_count() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_error_count() {
    ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn get_metrics() -> serde_json::Value {
    serde_json::json!({
        "request_count": REQUEST_COUNT.load(Ordering::Relaxed),
        "error_count": ERROR_COUNT.load(Ordering::Relaxed),
    })
}
