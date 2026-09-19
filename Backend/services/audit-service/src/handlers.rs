//! HTTP 处理器

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
};

use common::AppError;
use common::AppResult;
use crate::models::{LoginLogQuery, LoginLogResponse, LoginStatistics, OperationLogQuery, OperationLogResponse, OperationLogDetailResponse, BatchDeleteRequest, ApiCallLogQuery, ApiCallLogResponse, ApiCallStatistics, ApiEndpointStatistics, ApiTrendPoint, ApiResponseTimeDistribution};
use crate::repository::{AuditRepository, FindOperationLogsParams};
use crate::helpers::{json_ok_msg, json_health};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub repository: AuditRepository,
}

/// 创建路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // 登录日志
        .route("/login" , axum::routing::get(get_login_logs))
        .route(
            "/login/statistics" ,
            axum::routing::get(get_login_statistics),
        )
        // 操作日志
        .route("/operation" , axum::routing::get(get_operation_logs))
        .route(
            "/operation/batch" ,
            axum::routing::delete(batch_delete_operation_logs),
        )
        .route(
            "/operation/{id}" ,
            axum::routing::get(get_operation_log_detail),
        )
        // API 调用日志（API Governance）
        .route("/api-call" , axum::routing::get(get_api_call_logs))
        .route(
            "/api-call/statistics" ,
            axum::routing::get(get_api_call_statistics),
        )
        .route(
            "/api-call/endpoints" ,
            axum::routing::get(get_api_endpoint_statistics),
        )
        .route("/api-call/trend" , axum::routing::get(get_api_call_trend))
        .route(
            "/api-call/distribution" ,
            axum::routing::get(get_api_response_distribution),
        )
        .with_state(state)
}

// ============ 登录日志 ============

/// 获取登录日志列表
async fn get_login_logs(
    State(state): State<AppState>,
    Query(query): Query<LoginLogQuery>,
) -> AppResult<Json<LoginLogResponse>> {
    let (logs, total) = state
        .repository
        .find_login_logs(
            query.page,
            query.page_size,
            query.username.as_deref(),
            query.status,
            query.start_date.as_deref(),
            query.end_date.as_deref(),
        )
        .await?;

    Ok(Json(LoginLogResponse {
        items: logs,
        total,
        page: query.page,
        page_size: query.page_size,
    }))
}

/// 获取登录统计
async fn get_login_statistics(State(state): State<AppState>) -> AppResult<Json<LoginStatistics>> {
    let stats = state.repository.get_login_statistics().await?;
    Ok(Json(stats))
}

// ============ 操作日志 ============

/// 获取操作日志列表
async fn get_operation_logs(
    State(state): State<AppState>,
    Query(query): Query<OperationLogQuery>,
) -> AppResult<Json<OperationLogResponse>> {
    let (logs, total) = state
        .repository
        .find_operation_logs(
            FindOperationLogsParams {
                page: query.page,
                page_size: query.page_size,
                username: query.username.as_deref(),
                module: query.module.as_deref(),
                business_type: query.business_type.as_deref(),
                status: query.status,
                start_date: query.start_date.as_deref(),
                end_date: query.end_date.as_deref(),
            }
        )
        .await?;

    Ok(Json(OperationLogResponse {
        items: logs,
        total,
        page: query.page,
        page_size: query.page_size,
    }))
}

/// 获取操作日志详情
async fn get_operation_log_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<OperationLogDetailResponse>> {
    let log = state
        .repository
        .find_operation_log_by_id(id)
        .await?
        .ok_or(AppError::AuditNotFound)?;

    Ok(Json(OperationLogDetailResponse { log }))
}

/// 批量删除操作日志
async fn batch_delete_operation_logs(
    State(state): State<AppState>,
    Json(req): Json<BatchDeleteRequest>,
) -> AppResult<impl IntoResponse> {
    let count = state
        .repository
        .batch_delete_operation_logs(&req.ids)
        .await?;

    Ok(json_ok_msg(&format!("已删除 {} 条日志" , count)))
}

// ============ API 调用日志（API Governance） ============

/// 获取 API 调用日志列表
async fn get_api_call_logs(
    State(state): State<AppState>,
    Query(query): Query<ApiCallLogQuery>,
) -> AppResult<Json<ApiCallLogResponse>> {
    let (logs, total) = state.repository.find_api_call_logs(&query).await?;
    Ok(Json(ApiCallLogResponse { list: logs, total }))
}

/// 获取 API 调用统计
async fn get_api_call_statistics(
    State(state): State<AppState>,
) -> AppResult<Json<ApiCallStatistics>> {
    let stats = state.repository.get_api_call_statistics().await?;
    Ok(Json(stats))
}

/// 获取 API 端点统计
async fn get_api_endpoint_statistics(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ApiEndpointStatistics>>> {
    let stats = state.repository.get_api_endpoint_statistics().await?;
    Ok(Json(stats))
}

/// 获取 API 调用趋势
async fn get_api_call_trend(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ApiTrendPoint>>> {
    let trend = state.repository.get_api_call_trend().await?;
    Ok(Json(trend))
}

/// 获取响应时间分布
async fn get_api_response_distribution(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ApiResponseTimeDistribution>>> {
    let dist = state.repository.get_api_response_distribution().await?;
    Ok(Json(dist))
}

/// 健康检查
pub fn health_check() -> Json<serde_json::Value> {
    json_health("audit-service" )
}
