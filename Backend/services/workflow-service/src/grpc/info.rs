use std::sync::Arc;

use crate::repository::{
    InMemoryWorkflowRepository, TaskRecord, Workflow, WorkflowInstance,
};

/// Workflow 应用状态（gRPC 层）
#[derive(Default)]
pub struct WorkflowAppState {
    pub repository: InMemoryWorkflowRepository,
}

impl WorkflowAppState {
    pub const fn new(repository: InMemoryWorkflowRepository) -> Self {
        Self { repository }
    }
}

// ============== Info 结构体 ==============

/// 工作流信息
#[derive(Debug, Clone)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub version: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Workflow> for WorkflowInfo {
    fn from(w: Workflow) -> Self {
        Self {
            id: w.id,
            name: w.name,
            description: w.description,
            status: w.status,
            version: w.version,
            created_by: w.created_by,
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

/// 工作流实例信息
#[derive(Debug, Clone)]
pub struct WorkflowInstanceInfo {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: i32,
    pub status: String,
    pub current_node_id: Option<String>,
    pub started_by: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl From<WorkflowInstance> for WorkflowInstanceInfo {
    fn from(i: WorkflowInstance) -> Self {
        Self {
            id: i.id,
            workflow_id: i.workflow_id,
            workflow_version: i.workflow_version,
            status: i.status,
            current_node_id: i.current_node_id,
            started_by: i.started_by,
            started_at: Some(i.started_at.to_rfc3339()),
            completed_at: i.completed_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// 任务记录信息
#[derive(Debug, Clone)]
pub struct TaskRecordInfo {
    pub id: String,
    pub instance_id: String,
    pub node_id: String,
    pub node_name: String,
    pub assignee: Option<String>,
    pub status: String,
    pub comment: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl From<TaskRecord> for TaskRecordInfo {
    fn from(t: TaskRecord) -> Self {
        Self {
            id: t.id,
            instance_id: t.instance_id,
            node_id: t.node_id,
            node_name: t.node_name,
            assignee: t.assignee,
            status: t.status,
            comment: t.comment,
            started_at: Some(t.started_at.to_rfc3339()),
            completed_at: t.completed_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Workflow gRPC 服务实现
#[derive(Clone)]
pub struct WorkflowGrpcService {
    state: Arc<WorkflowAppState>,
}

impl WorkflowGrpcService {
    pub const fn new(state: Arc<WorkflowAppState>) -> Self {
        Self { state }
    }

    #[must_use]
    pub const fn state(&self) -> &Arc<WorkflowAppState> {
        &self.state
    }
}
