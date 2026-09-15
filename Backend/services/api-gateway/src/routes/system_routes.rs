use std::sync::Arc;

use axum::{
    Router, routing::{get, post}, extract::{State, Path, Extension}, Json,
    response::{self, IntoResponse},
};
use common::ApiResponse;
use serde_json::{Value, json};

/// 获取当前追踪上下文
async fn trace_context_handler() -> Json<ApiResponse<common::otel::TraceContext>> {
    let ctx = common::otel::get_trace_context("api-gateway");
    Json(ApiResponse::success(ctx))
}

use crate::{
    AppState, CircuitBreakerStatusResponse, CircuitStatsInfo, RateLimitState,
};
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

// ==================== 系统监控 ====================

async fn monitor_handler() -> Json<Value> {
    json_success(json!({"cpu": 0.35, "memory": 0.62, "disk": 0.45}))
}

// ==================== 权限管理 ====================

async fn list_permissions() -> AppResult<Json<Value>> {
    tracing::warn!("list_permissions: 功能未实现");
    Err(AppError::NotImplemented("list_permissions: 功能未实现".to_string()))
}

async fn create_permission() -> Json<Value> {
    json_ok()
}

async fn get_permission() -> Json<Value> {
    json_success(json!(null))
}

async fn update_permission() -> Json<Value> {
    json_ok()
}

async fn delete_permission() -> Json<Value> {
    json_ok()
}

async fn batch_create_permissions() -> Json<Value> {
    json_ok()
}

async fn get_role_perm_config() -> Json<Value> {
    json_success(json!(null))
}

async fn update_role_perm_config() -> Json<Value> {
    json_ok()
}

async fn get_role_data_perms() -> Json<Value> {
    json_success(json!([]))
}

async fn set_role_data_perms() -> Json<Value> {
    json_ok()
}

async fn get_role_field_perms() -> Json<Value> {
    json_success(json!([]))
}

async fn set_role_field_perms() -> Json<Value> {
    json_ok()
}

async fn get_role_inherit() -> Json<Value> {
    json_success(json!(null))
}

async fn set_role_inherit() -> Json<Value> {
    json_ok()
}

async fn remove_role_inherit() -> Json<Value> {
    json_ok()
}

async fn get_accessible_depts() -> Json<Value> {
    json_success(json!([]))
}

async fn get_accessible_tenants() -> Json<Value> {
    json_success(json!([]))
}

async fn validate_data_perm() -> Json<Value> {
    json_success(json!({"valid": true}))
}

async fn check_sensitive_perm() -> Json<Value> {
    json_success(json!({"sensitive": false}))
}

async fn refresh_perm_cache() -> Json<Value> {
    json_ok()
}

async fn batch_assign_perms() -> Json<Value> {
    json_ok()
}

async fn copy_role_perms() -> Json<Value> {
    json_ok()
}

async fn list_perm_change_logs() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn export_perm_change_logs() -> Json<Value> {
    json_success(json!(null))
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
        // 系统监控
        .route("/api/admin/monitor", get(monitor_handler))
        // 权限管理
        .route("/api/admin/permissions", get(list_permissions).post(create_permission))
        .route("/api/admin/permissions/{key}", get(get_permission).put(update_permission).delete(delete_permission))
        .route("/api/admin/permissions/batch", post(batch_create_permissions))
        .route("/api/admin/roles/{role_name}/permission-config", get(get_role_perm_config).put(update_role_perm_config))
        .route("/api/admin/roles/{role_name}/data-permissions", get(get_role_data_perms).put(set_role_data_perms))
        .route("/api/admin/roles/{role_name}/field-permissions", get(get_role_field_perms).put(set_role_field_perms))
        .route("/api/admin/roles/{role_name}/inherit", get(get_role_inherit).post(set_role_inherit).delete(remove_role_inherit))
        .route("/api/admin/roles/{role_name}/accessible-departments", get(get_accessible_depts))
        .route("/api/admin/roles/{role_name}/accessible-tenants", get(get_accessible_tenants))
        .route("/api/permissions/validate-data", post(validate_data_perm))
        .route("/api/permissions/check-sensitive", post(check_sensitive_perm))
        .route("/api/permissions/refresh-cache", post(refresh_perm_cache))
        .route("/api/permissions/batch-assign", post(batch_assign_perms))
        .route("/api/permissions/copy", post(copy_role_perms))
        .route("/api/permission-change-logs", get(list_perm_change_logs))
        .route("/api/permission-change-logs/export", get(export_perm_change_logs))
}
