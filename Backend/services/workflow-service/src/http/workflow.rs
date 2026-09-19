use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use super::{AppState, ListQuery, PaginatedData, WorkflowResponse};
use crate::repository::workflow_repository::WorkflowRepository;
use crate::repository::Workflow;

/// 获取工作流列表
pub async fn list_workflows(
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
pub async fn get_workflow(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
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
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<super::CreateWorkflowRequest>,
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
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<super::CreateWorkflowRequest>,
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
pub async fn delete_workflow(
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
pub async fn execute_workflow(
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
