//! Workflow 数据仓储层
//!
//! 工作流、实例、任务、报表、定时任务的数据库 CRUD 操作
//!
//! 包含 `PostgreSQL` 实现和内存实现

// ============================================================================
// 子模块声明
// ============================================================================

pub mod workflow_repository;
pub mod instance_repository;
pub mod task_repository;
pub mod event_repository;

// ============================================================================
// 共享数据模型
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: serde_json::Value,
    pub status: String,
    pub version: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    #[must_use]
    pub fn new(name: String, description: Option<String>, created_by: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            definition: serde_json::json!({}),
            status: "draft".to_string(),
            version: 1,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub node_type: String,
    pub position_x: i32,
    pub position_y: i32,
    pub config: serde_json::Value,
    pub timeout: Option<i32>,
    pub auto_complete: bool,
    pub created_at: DateTime<Utc>,
}

/// 工作流连线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub id: String,
    pub workflow_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: String,
    pub condition: Option<String>,
    pub label: Option<String>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
}

/// 工作流实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: i32,
    pub status: String,
    pub current_node_id: Option<String>,
    pub variables: serde_json::Value,
    pub started_by: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 任务记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub instance_id: String,
    pub node_id: String,
    pub node_name: String,
    pub assignee: Option<String>,
    pub status: String,
    pub comment: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub form_data: Option<serde_json::Value>,
    pub timeout_at: Option<DateTime<Utc>>,
}

/// 定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub cron_expression: Option<String>,
    pub interval_seconds: Option<i64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub task_handler: String,
    pub task_params: Option<serde_json::Value>,
    pub status: String,
    pub next_run_time: Option<DateTime<Utc>>,
    pub last_run_time: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl ScheduledTask {
    #[must_use]
    pub fn new(
        name: String,
        cron_expression: String,
        action_type: String,
        action_params: Option<serde_json::Value>,
        created_by: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            task_type: "cron".to_string(),
            cron_expression: Some(cron_expression),
            interval_seconds: None,
            start_time: None,
            end_time: None,
            task_handler: action_type,
            task_params: action_params,
            status: "active".to_string(),
            next_run_time: None,
            last_run_time: None,
            created_by,
            created_at: Utc::now(),
        }
    }
}

/// 报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: serde_json::Value,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub generated_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl Report {
    #[must_use]
    pub fn new(
        name: String,
        report_type: String,
        query_params: Option<serde_json::Value>,
        created_by: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            report_type,
            config: query_params.unwrap_or(serde_json::json!({})),
            status: "draft".to_string(),
            result: None,
            generated_at: None,
            created_by,
            created_at: Utc::now(),
        }
    }
}

// ============================================================================
// 重新导出所有 trait 和实现
// ============================================================================

// Workflow Repository
pub use workflow_repository::{
    PostgresWorkflowRepository, WorkflowRepository,
};

// Instance Repository
pub use instance_repository::{InstanceRepository, PostgresInstanceRepository};

// Task Repository
pub use task_repository::{PostgresTaskRepository, TaskRepository};

// Event Repository (Report + ScheduledTask)
pub use event_repository::{
    ReportRepository,
    ScheduledTaskRepository,
};
