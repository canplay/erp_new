use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use super::{AppState, CreateScheduledTaskRequest, ListQuery, PaginatedData};
use crate::repository::event_repository::ScheduledTaskRepository;
use crate::repository::ScheduledTask;

/// 定时任务响应
#[derive(Debug, serde::Serialize)]
pub struct ScheduledTaskResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub cron_expression: Option<String>,
    pub interval_seconds: Option<i64>,
    pub task_handler: String,
    pub status: String,
    pub next_run_time: Option<String>,
    pub last_run_time: Option<String>,
}

/// 获取定时任务列表
pub async fn list_scheduled_tasks(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let page = i64::from(params.page.unwrap_or(1));
    let page_size = i64::from(params.page_size.unwrap_or(10));

    let Ok((tasks, total)) = state.scheduled_task_repo.list(page, page_size).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": "查询失败"
            })),
        );
    };

    let list: Vec<ScheduledTaskResponse> = tasks
        .into_iter()
        .map(|t| ScheduledTaskResponse {
            id: t.id,
            name: t.name.clone(),
            description: t.description,
            task_type: t.task_type,
            cron_expression: t.cron_expression.clone(),
            interval_seconds: t.interval_seconds,
            task_handler: t.task_handler.clone(),
            status: t.status.clone(),
            next_run_time: t.next_run_time.map(|dt| dt.to_rfc3339()),
            last_run_time: t.last_run_time.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": PaginatedData {
                list,
                total,
                page,
                page_size,
            }
        })),
    )
}

/// 获取单个定时任务
pub async fn get_scheduled_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.scheduled_task_repo.find_by_id(&id).await {
        Ok(Some(task)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": ScheduledTaskResponse {
                    id: task.id,
                    name: task.name.clone(),
                    description: task.description,
                    task_type: task.task_type,
                    cron_expression: task.cron_expression.clone(),
                    interval_seconds: task.interval_seconds,
                    task_handler: task.task_handler.clone(),
                    status: task.status.clone(),
                    next_run_time: task.next_run_time.map(|dt| dt.to_rfc3339()),
                    last_run_time: task.last_run_time.map(|dt| dt.to_rfc3339()),
                }
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "定时任务不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("查询定时任务失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "查询失败"
                })),
            )
        }
    }
}

/// 创建定时任务
pub async fn create_scheduled_task(
    State(state): State<AppState>,
    Json(req): Json<CreateScheduledTaskRequest>,
) -> impl IntoResponse {
    let task = ScheduledTask::new(
        req.name,
        req.cron_expression,
        req.action_type,
        req.action_params,
        "system".to_string(),
    );

    match state.scheduled_task_repo.create(&task).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "id": task.id,
                    "name": task.name
                }
            })),
        ),
        Err(e) => {
            tracing::error!("创建定时任务失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "创建失败"
                })),
            )
        }
    }
}

/// 删除定时任务
pub async fn delete_scheduled_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.scheduled_task_repo.delete(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "定时任务删除成功"
            })),
        ),
        Err(e) => {
            tracing::error!("删除定时任务失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "删除失败"
                })),
            )
        }
    }
}

/// 触发定时任务
pub async fn trigger_scheduled_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("【HTTP】触发定时任务: {id}");

    match state.scheduled_task_repo.find_by_id(&id).await {
        Ok(Some(task)) => {
            if task.status != "active" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "message": "定时任务未启用，无法触发"
                    })),
                );
            }

            tracing::info!(
                "【HTTP】定时任务已触发: {}, task_handler={}",
                task.name,
                task.task_handler
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "定时任务触发成功",
                    "data": {
                        "task_id": task.id,
                        "task_handler": task.task_handler
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "定时任务不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("【HTTP】获取定时任务失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "触发失败"
                })),
            )
        }
    }
}
