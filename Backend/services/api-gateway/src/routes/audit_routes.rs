//! 审计日志路由 — 真实 gRPC 调用 audit-service

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::get, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct LogQuery { page: Option<i32>, page_size: Option<i32>, user_id: Option<i64>, keyword: Option<String>, start_date: Option<String>, end_date: Option<String>, operation_type: Option<String>, status: Option<i32>, period: Option<String> }

/// 获取 audit-service gRPC 客户端
async fn get_audit_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::AuditGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.audit_client().await
        .map_err(|e| json_error(&format!("audit-service 不可用: {e}")))
}

/// proto AuditLog → JSON
fn audit_log_to_json(l: &grpc_proto::audit::AuditLog) -> Value {
    json!({
        "id": l.id,
        "user_id": l.user_id,
        "username": l.username,
        "ip_address": l.ip_address,
        "user_agent": l.user_agent,
        "action": l.action,
        "resource_type": l.resource_type,
        "resource_id": l.resource_id,
        "description": l.description,
        "created_at": l.created_at,
    })
}

/// proto LoginLogInfo → JSON
fn login_log_to_json(l: &grpc_proto::audit::LoginLogInfo) -> Value {
    json!({
        "id": l.id,
        "user_id": l.user_id,
        "username": l.username,
        "ip_address": l.ip_address,
        "user_agent": l.user_agent,
        "success": l.success,
        "fail_reason": l.fail_reason,
        "login_method": l.login_method,
        "created_at": l.created_at,
        // 兼容前端 LoginLog 类型字段（status=1 成功 / 0 失败, ip, error_msg）
        "status": if l.success { 1 } else { 0 },
        "ip": l.ip_address,
        "error_msg": l.fail_reason,
        "login_time": l.created_at,
    })
}

/// proto ApiCallLogInfo → JSON
fn api_call_log_to_json(l: &grpc_proto::audit::ApiCallLogInfo) -> Value {
    json!({
        "id": l.id,
        "user_id": l.user_id,
        "username": l.username,
        "method": l.method,
        "path": l.path,
        "status_code": l.status_code,
        "duration_ms": l.duration_ms,
        "ip_address": l.ip_address,
        "created_at": l.created_at,
    })
}

async fn list_operation_logs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LogQuery>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_logs(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.user_id.unwrap_or(0),
        q.keyword.clone().unwrap_or_default(),
        String::new(),
        q.operation_type.clone().unwrap_or_default(),
        q.start_date.clone().unwrap_or_default(),
        q.end_date.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.logs.iter().map(audit_log_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_operation_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_log(id).await {
        Ok(resp) => {
            if let Some(log) = resp.log {
                json_success(audit_log_to_json(&log))
            } else {
                json_error("日志不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn list_login_logs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LogQuery>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_login_logs(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.user_id.unwrap_or(0),
        q.keyword.clone().unwrap_or_default(),
        q.status.unwrap_or(-1),  // -1 = 不过滤
        q.start_date.clone().unwrap_or_default(),
        q.end_date.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.logs.iter().map(login_log_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_login_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_login_log(id).await {
        Ok(resp) => {
            if let Some(log) = resp.log {
                json_success(login_log_to_json(&log))
            } else {
                json_error("日志不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn list_api_call_logs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LogQuery>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_api_call_logs(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.user_id.unwrap_or(0),
        String::new(),
        q.keyword.clone().unwrap_or_default(),
        q.status.unwrap_or(0),
        q.start_date.clone().unwrap_or_default(),
        q.end_date.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.logs.iter().map(api_call_log_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_api_call_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_api_call_log(id).await {
        Ok(resp) => {
            if let Some(log) = resp.log {
                json_success(api_call_log_to_json(&log))
            } else {
                json_error("日志不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_operation_stats(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LogQuery>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_stats(q.period.clone().unwrap_or("today".to_string())).await {
        Ok(resp) => {
            if let Some(s) = resp.stats {
                json_success(json!({
                    "total_calls": s.total_count,
                    "today_count": s.today_count,
                    "week_count": s.week_count,
                    "by_action": s.by_action,
                    "by_resource_type": s.by_resource_type,
                }))
            } else {
                json_success(json!({"total_calls": 0, "today_count": 0, "week_count": 0}))
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_api_call_statistics(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LogQuery>,
) -> Json<Value> {
    let mut client = match get_audit_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_stats(q.period.clone().unwrap_or("today".to_string())).await {
        Ok(resp) => {
            if let Some(s) = resp.stats {
                json_success(json!({
                    "total_requests": s.total_count,
                    "today_count": s.today_count,
                    "week_count": s.week_count,
                }))
            } else {
                json_success(json!({"total_requests": 0, "today_count": 0, "week_count": 0}))
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

// 以下接口 audit.proto 暂未提供对应 RPC，保留空实现（待 proto 扩展）
async fn get_api_endpoint_statistics() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn get_api_trend() -> Json<Value> {
    json_success(json!({"list": []}))
}
async fn get_api_response_distribution() -> Json<Value> {
    json_success(json!({"list": []}))
}
async fn get_api_category_statistics() -> Json<Value> {
    json_success(json!({"list": []}))
}
async fn get_api_performance_baseline() -> Json<Value> {
    json_success(json!({"list": []}))
}

async fn export_logs_fn() -> Json<Value> {
    json_ok()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 操作日志
        .route("/api/audit/operation-logs", get(list_operation_logs))
        .route("/api/audit/operation-logs/{id}", get(get_operation_log))
        .route("/api/audit/operation-logs/export", get(export_logs_fn))
        .route("/api/audit/operation-stats", get(get_operation_stats))
        // 登录日志
        .route("/api/audit/login-logs", get(list_login_logs))
        .route("/api/audit/login-logs/{id}", get(get_login_log))
        .route("/api/audit/login-logs/export", get(export_logs_fn))
        // API调用日志
        .route("/api/audit/api-call-logs", get(list_api_call_logs))
        .route("/api/audit/api-call-logs/{id}", get(get_api_call_log))
        .route("/api/audit/api-call-logs/export", get(export_logs_fn))
        // API治理统计
        .route("/api/audit/api-call-statistics", get(get_api_call_statistics))
        .route("/api/audit/api-endpoint-statistics", get(get_api_endpoint_statistics))
        .route("/api/audit/api-trend", get(get_api_trend))
        .route("/api/audit/api-response-distribution", get(get_api_response_distribution))
        .route("/api/audit/api-category-statistics", get(get_api_category_statistics))
        .route("/api/audit/api-performance-baseline", get(get_api_performance_baseline))
}
