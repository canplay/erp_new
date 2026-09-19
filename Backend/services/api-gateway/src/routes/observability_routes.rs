//! 可观测性路由 — 指标、统计、追踪、熔断器、限流

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post},
    response::{self, IntoResponse},
};
use common::ApiResponse;
use serde_json::{json, Value};

use crate::{
    AppState, CircuitBreakerStatusResponse, CircuitStatsInfo, RateLimitState,
};

/// 获取当前追踪上下文
async fn trace_context_handler() -> Json<ApiResponse<common::otel::TraceContext>> {
    let ctx = common::otel::get_trace_context("api-gateway");
    Json(ApiResponse::success(ctx))
}

use crate::routes::helpers::*;

async fn rate_limit_info_handler(
    Extension(rate_limit_state): Extension<RateLimitState>,
) -> Json<ApiResponse<Value>> {
    Json(ApiResponse::success(json!({
        "max_requests": rate_limit_state.max_requests,
        "window_secs": rate_limit_state.window_secs,
    })))
}

async fn metrics_handler() -> response::Response {
    let metrics = crate::metrics::metrics_collector();
    let body = metrics.render_prometheus();
    response::Response::new(body.into())
}

async fn stats_handler(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<crate::stats::DashboardStats>> {
    crate::stats::get_dashboard_stats(State(state)).await
}

async fn circuit_breaker_status_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let services = [
        ("auth", "auth-service"),
        ("user", "user-service"),
        ("cms", "cms-service"),
        ("message", "message-service"),
        ("feedback", "feedback-service"),
        ("tenant", "tenant-service"),
        ("file", "file-service"),
        ("workflow", "workflow-service"),
        ("audit", "audit-service"),
        ("api-key", "api-key-service"),
    ];

    let mut statuses = Vec::new();
    for (service_key, service_name) in services {
        let result = state.circuit_breaker_manager.check_circuit(service_key);
        let stats = state.circuit_breaker_manager.get_stats(service_key);

        let stats_info = stats.map(|s| CircuitStatsInfo {
            success_count: s.success_count,
            failure_count: s.failure_count,
            failure_rate: s.failure_rate(),
            total_requests: s.total_requests(),
        });

        statuses.push(CircuitBreakerStatusResponse {
            service: service_name.to_string(),
            state: result.state,
            allowed: result.allowed,
            stats: stats_info,
        });
    }

    tracing::debug!("【熔断器】获取状态: {} 服务", statuses.len());
    Json(ApiResponse::success(statuses))
}

async fn circuit_breaker_reset_handler(
    State(state): State<Arc<AppState>>,
    Path(service): Path<String>,
) -> Json<ApiResponse<Value>> {
    let service_key = match service.as_str() {
        "auth-service" => "auth",
        "user-service" => "user",
        "cms-service" => "cms",
        "message-service" => "message",
        "feedback-service" => "feedback",
        "tenant-service" => "tenant",
        "file-service" => "file",
        "workflow-service" => "workflow",
        "audit-service" => "audit",
        "api-key-service" => "api-key",
        _ => {
            return Json(ApiResponse::error(&format!("未知服务: {service}")));
        }
    };

    state.circuit_breaker_manager.reset(service_key);
    tracing::info!("【熔断器】重置服务: {service}");

    Json(ApiResponse::success(json!({
        "message": format!("熔断器已重置: {}", service)
    })))
}

async fn circuit_breaker_reset_all_handler(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Value>> {
    state.circuit_breaker_manager.reset_all();
    tracing::info!("【熔断器】重置所有熔断器");

    Json(ApiResponse::success(json!({
        "message": "所有熔断器已重置"
    })))
}

async fn monitor_handler() -> Json<Value> {
    json_success(json!({"cpu": 0.35, "memory": 0.62, "disk": 0.45}))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/rate-limit", get(rate_limit_info_handler))
        .route("/metrics", get(metrics_handler))
        .route("/api/admin/stats", get(stats_handler))
        .route("/trace", get(trace_context_handler))
        .route("/circuit-breaker/status", get(circuit_breaker_status_handler))
        .route("/circuit-breaker/reset/{service}", post(circuit_breaker_reset_handler))
        .route("/circuit-breaker/reset-all", post(circuit_breaker_reset_all_handler))
        .route("/api/admin/monitor", get(monitor_handler))
}
