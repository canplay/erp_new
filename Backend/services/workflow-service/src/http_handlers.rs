//!
//! HTTP Handlers for Workflow Service
//!
//! 提供工作流、报表和调度的 HTTP 接口

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::repository::{
    InMemoryReportRepository, InMemoryScheduledTaskRepository, InMemoryWorkflowRepository, Report,
    ReportRepository, ScheduledTask, ScheduledTaskRepository, Workflow, WorkflowRepository,
};

/// 应用状态类型
pub type AppState = Arc<WorkflowState>;

/// 工作流状态结构 - 使用具体类型简化
pub struct WorkflowState {
    pub workflow_repo: InMemoryWorkflowRepository,
    pub report_repo: InMemoryReportRepository,
    pub scheduled_task_repo: InMemoryScheduledTaskRepository,
}

impl WorkflowState {
    /// 创建带内存仓库的状态
    #[must_use]
    pub fn new() -> Self {
        Self {
            workflow_repo: InMemoryWorkflowRepository::new(),
            report_repo: InMemoryReportRepository::new(),
            scheduled_task_repo: InMemoryScheduledTaskRepository::new(),
        }
    }
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self::new()
    }
}

/// 列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
}

/// 创建工作流请求
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 创建定时任务请求
#[derive(Debug, Deserialize)]
pub struct CreateScheduledTaskRequest {
    pub name: String,
    pub cron_expression: String,
    pub action_type: String,
    pub action_params: Option<serde_json::Value>,
}

/// 创建报表请求
#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub name: String,
    pub report_type: String,
    pub query_params: Option<serde_json::Value>,
}

/// 工作流响应
#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub version: i32,
    pub created_at: String,
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PaginatedData<T> {
    pub list: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

// ============================================================================
// 工作流处理器
// ============================================================================

/// 获取工作流列表
async fn list_workflows(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let page = i64::from(params.page.unwrap_or(1));
    let page_size = i64::from(params.page_size.unwrap_or(10));

    let Ok((workflows, total)) = state
        .workflow_repo
        .list(params.keyword.as_deref(), page, page_size)
        .await
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": "查询失败"
            })),
        );
    };

    let list: Vec<WorkflowResponse> = workflows
        .into_iter()
        .map(|w| WorkflowResponse {
            id: w.id,
            name: w.name,
            description: w.description,
            status: w.status,
            version: w.version,
            created_at: w.created_at.to_rfc3339(),
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

/// 获取单个工作流
async fn get_workflow(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.workflow_repo.find_by_id(&id).await {
        Ok(Some(workflow)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": WorkflowResponse {
                    id: workflow.id,
                    name: workflow.name,
                    description: workflow.description,
                    status: workflow.status,
                    version: workflow.version,
                    created_at: workflow.created_at.to_rfc3339(),
                }
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "工作流不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("查询工作流失败: {e}");
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

/// 创建工作流
async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    let workflow = Workflow::new(req.name, req.description, "system".to_string());

    match state.workflow_repo.create(&workflow).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "id": workflow.id,
                    "name": workflow.name
                }
            })),
        ),
        Err(e) => {
            tracing::error!("创建工作流失败: {e}");
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

/// 更新工作流
async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    match state.workflow_repo.find_by_id(&id).await {
        Ok(Some(mut workflow)) => {
            workflow.name = req.name;
            workflow.description = req.description;

            match state.workflow_repo.update(&workflow).await {
                Ok(()) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "message": "工作流更新成功"
                    })),
                ),
                Err(e) => {
                    tracing::error!("更新工作流失败: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "message": "更新失败"
                        })),
                    )
                }
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "工作流不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("获取工作流失败: {e}");
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

/// 删除工作流
async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.workflow_repo.delete(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "工作流删除成功"
            })),
        ),
        Err(e) => {
            tracing::error!("删除工作流失败: {e}");
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

/// 执行工作流
async fn execute_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("【HTTP】执行工作流: {id}");

    match state.workflow_repo.find_by_id(&id).await {
        Ok(Some(workflow)) => {
            if workflow.status != "published" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "message": "工作流未发布，无法执行"
                    })),
                );
            }

            let instance_id = uuid::Uuid::new_v4().to_string();
            tracing::info!("【HTTP】工作流实例已创建: {instance_id}");

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "工作流执行启动",
                    "data": {
                        "instance_id": instance_id,
                        "workflow_id": id
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "工作流不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("【HTTP】获取工作流失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "执行失败"
                })),
            )
        }
    }
}

// ============================================================================
// 定时任务处理器
// ============================================================================

/// 定时任务响应
#[derive(Debug, Serialize)]
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
async fn list_scheduled_tasks(
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
async fn get_scheduled_task(
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
async fn create_scheduled_task(
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
async fn delete_scheduled_task(
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
async fn trigger_scheduled_task(
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

// ============================================================================
// 报表处理器
// ============================================================================

/// 报表响应
#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub id: String,
    pub name: String,
    pub report_type: String,
    pub status: String,
    pub created_at: String,
    pub generated_at: Option<String>,
}

/// 获取报表列表
async fn list_reports(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let page = i64::from(params.page.unwrap_or(1));
    let page_size = i64::from(params.page_size.unwrap_or(10));

    let Ok((reports, total)) = state.report_repo.list(page, page_size).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": "查询失败"
            })),
        );
    };

    let list: Vec<ReportResponse> = reports
        .into_iter()
        .map(|r| ReportResponse {
            id: r.id,
            name: r.name,
            report_type: r.report_type,
            status: r.status,
            created_at: r.created_at.to_rfc3339(),
            generated_at: r.generated_at.map(|dt| dt.to_rfc3339()),
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

/// 获取单个报表
async fn get_report(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.report_repo.find_by_id(&id).await {
        Ok(Some(report)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": ReportResponse {
                    id: report.id,
                    name: report.name,
                    report_type: report.report_type,
                    status: report.status,
                    created_at: report.created_at.to_rfc3339(),
                    generated_at: report.generated_at.map(|dt| dt.to_rfc3339()),
                }
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "报表不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("查询报表失败: {e}");
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

/// 创建报表
async fn create_report(
    State(state): State<AppState>,
    Json(req): Json<CreateReportRequest>,
) -> impl IntoResponse {
    let report = Report::new(
        req.name,
        req.report_type,
        req.query_params,
        "system".to_string(),
    );

    match state.report_repo.create(&report).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "id": report.id,
                    "name": report.name
                }
            })),
        ),
        Err(e) => {
            tracing::error!("创建报表失败: {e}");
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

/// 生成报表
async fn generate_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("【HTTP】生成报表: {id}");

    match state.report_repo.find_by_id(&id).await {
        Ok(Some(report)) => {
            tracing::info!(
                "【HTTP】报表生成中: {}, type={}",
                report.name,
                report.report_type
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "报表生成中",
                    "data": {
                        "report_id": report.id,
                        "report_type": report.report_type,
                        "status": "generating"
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "报表不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("【HTTP】获取报表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "生成失败"
                })),
            )
        }
    }
}

/// 下载报表
async fn download_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("【HTTP】下载报表: {id}");

    match state.report_repo.find_by_id(&id).await {
        Ok(Some(report)) => {
            if report.result.is_none() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "message": "报表尚未生成"
                    })),
                );
            }

            tracing::info!(
                "【HTTP】报表下载准备: {}, generated_at={:?}",
                report.name,
                report.generated_at
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "报表准备就绪",
                    "data": {
                        "report_id": report.id,
                        "name": report.name,
                        "report_type": report.report_type,
                        "result": report.result,
                        "generated_at": report.generated_at.map(|dt| dt.to_rfc3339())
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "报表不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("【HTTP】获取报表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "下载失败"
                })),
            )
        }
    }
}

/// 健康检查
async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "healthy",
            "service": "workflow-service"
        })),
    )
}

// ============================================================================
// 路由创建
// ============================================================================

/// 创建 HTTP 路由
pub fn create_http_router() -> Router {
    use axum::routing::{delete, get, post, put};

    let state = AppState::new(WorkflowState::new());

    Router::new()
        .route("/api/workflows", get(list_workflows))
        .route("/api/workflows", post(create_workflow))
        .route("/api/workflows/:id", get(get_workflow))
        .route("/api/workflows/:id", put(update_workflow))
        .route("/api/workflows/:id", delete(delete_workflow))
        .route("/api/workflows/:id/execute", post(execute_workflow))
        .route("/api/scheduled-tasks", get(list_scheduled_tasks))
        .route("/api/scheduled-tasks", post(create_scheduled_task))
        .route("/api/scheduled-tasks/:id", get(get_scheduled_task))
        .route("/api/scheduled-tasks/:id", delete(delete_scheduled_task))
        .route(
            "/api/scheduled-tasks/:id/trigger",
            post(trigger_scheduled_task),
        )
        .route("/api/reports", get(list_reports))
        .route("/api/reports", post(create_report))
        .route("/api/reports/:id", get(get_report))
        .route("/api/reports/:id/generate", post(generate_report))
        .route("/api/reports/:id/download", get(download_report))
        .route("/health", get(health))
        .with_state(state)
}
