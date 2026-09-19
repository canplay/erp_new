//! HTTP Handlers for Workflow Service
//!
//! 提供工作流、报表和调度的 HTTP 接口

mod report;
mod router;
mod scheduled_task;
mod workflow;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::repository::{
    InMemoryReportRepository, InMemoryScheduledTaskRepository, InMemoryWorkflowRepository,
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

/// 健康检查
use axum::{Json, response::IntoResponse};
pub async fn health() -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({
            "status": "healthy",
            "service": "workflow-service"
        })),
    )
}
