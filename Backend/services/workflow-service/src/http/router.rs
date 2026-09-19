use axum::routing::{delete, get, post, put};
use axum::Router;

use super::{health, AppState, WorkflowState};
use crate::http::report::{create_report, download_report, generate_report, get_report, list_reports};
use crate::http::scheduled_task::{
    create_scheduled_task, delete_scheduled_task, get_scheduled_task, list_scheduled_tasks,
    trigger_scheduled_task,
};
use crate::http::workflow::{
    create_workflow, delete_workflow, execute_workflow, get_workflow, list_workflows, update_workflow,
};

/// 创建 HTTP 路由
pub fn create_http_router() -> Router {
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
        .route("/api/scheduled-tasks/:id/trigger", post(trigger_scheduled_task))
        .route("/api/reports", get(list_reports))
        .route("/api/reports", post(create_report))
        .route("/api/reports/:id", get(get_report))
        .route("/api/reports/:id/generate", post(generate_report))
        .route("/api/reports/:id/download", get(download_report))
        .route("/health", get(health))
        .with_state(state)
}
