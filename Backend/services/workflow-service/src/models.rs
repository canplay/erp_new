//!
//! 工作流数据模型
//!
//! 定义工作流、节点、边、实例等核心数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 工作流定义状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum WorkflowStatus {
    #[default]
    Draft, // 草稿
    Published, // 已发布
    Disabled,  // 已禁用
}

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum NodeType {
    Start, // 开始节点
    End,   // 结束节点
    #[default]
    Task, // 任务节点
    Approval, // 审批节点
    Condition, // 条件节点
    Parallel, // 并行节点
    Merge, // 合并节点
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start => write!(f, "start"),
            Self::End => write!(f, "end"),
            Self::Task => write!(f, "task"),
            Self::Approval => write!(f, "approval"),
            Self::Condition => write!(f, "condition"),
            Self::Parallel => write!(f, "parallel"),
            Self::Merge => write!(f, "merge"),
        }
    }
}

/// 节点状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum NodeStatus {
    #[default]
    Pending, // 待处理
    Running,   // 执行中
    Completed, // 已完成
    Rejected,  // 已拒绝
    Skipped,   // 已跳过
}

/// 边类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum EdgeType {
    #[default]
    Normal, // 普通连线
    Condition, // 条件连线
    Default,   // 默认连线
}

/// 实例状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InstanceStatus {
    #[default]
    Pending, // 待启动
    Running,   // 运行中
    Completed, // 已完成
    Cancelled, // 已取消
    Rejected,  // 已拒绝
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: serde_json::Value, // BPMN 或 JSON 格式定义
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
            id: Uuid::new_v4().to_string(),
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
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowNode {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub node_type: String,
    pub position_x: f64,
    pub position_y: f64,
    pub config: serde_json::Value,
    pub timeout: Option<i32>,
    pub auto_complete: Option<bool>,       // 数据库中为 nullable
    pub created_at: Option<DateTime<Utc>>, // 数据库中为 nullable
}

impl WorkflowNode {
    #[must_use]
    pub fn new(
        workflow_id: String,
        name: String,
        node_type: NodeType,
        position_x: f64,
        position_y: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            name,
            node_type: node_type.to_string(),
            position_x,
            position_y,
            config: serde_json::json!({}),
            timeout: None,
            auto_complete: None,
            created_at: None,
        }
    }
}

/// 工作流边（连线）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowEdge {
    pub id: String,
    pub workflow_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: String,
    pub condition: Option<String>,
    pub label: Option<String>,
    pub priority: Option<i32>,             // 数据库中为 nullable
    pub created_at: Option<DateTime<Utc>>, // 数据库中为 nullable
}

impl WorkflowEdge {
    #[must_use]
    pub fn new(workflow_id: String, source_node_id: String, target_node_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            source_node_id,
            target_node_id,
            edge_type: "normal".to_string(),
            condition: None,
            label: None,
            priority: None,
            created_at: None,
        }
    }
}

/// 工作流实例
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: i32,
    pub status: String,
    pub current_node_id: Option<String>,
    pub variables: serde_json::Value,
    pub started_by: String,
    pub started_at: Option<DateTime<Utc>>, // 数据库中为 nullable
    pub completed_at: Option<DateTime<Utc>>,
}

impl WorkflowInstance {
    #[must_use]
    pub fn new(workflow_id: String, workflow_version: i32, started_by: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            workflow_version,
            status: "pending".to_string(),
            current_node_id: None,
            variables: serde_json::json!({}),
            started_by,
            started_at: None,
            completed_at: None,
        }
    }
}

/// 任务记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRecord {
    pub id: String,
    pub instance_id: String,
    pub node_id: String,
    pub node_name: String,
    pub assignee: Option<String>, // 数据库中为 nullable
    pub status: String,
    pub comment: Option<String>,
    pub form_data: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>, // 数据库中为 nullable
    pub completed_at: Option<DateTime<Utc>>,
    pub timeout_at: Option<DateTime<Utc>>,
}

impl TaskRecord {
    #[must_use]
    pub fn new(
        instance_id: String,
        node_id: String,
        node_name: String,
        assignee: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            instance_id,
            node_id,
            node_name,
            assignee,
            status: "pending".to_string(),
            comment: None,
            form_data: None,
            started_at: None,
            completed_at: None,
            timeout_at: None,
        }
    }
}

/// 报表状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ReportStatus {
    #[default]
    Draft,
    Generated,
    Failed,
}

impl std::fmt::Display for ReportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Generated => write!(f, "generated"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// 报表定义
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub report_type: String,
    pub query_params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub status: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub generated_at: Option<DateTime<Utc>>,
}

impl Report {
    #[must_use]
    pub fn new(
        name: String,
        report_type: String,
        query_params: Option<serde_json::Value>,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            report_type,
            query_params,
            result: None,
            status: "draft".to_string(),
            created_by,
            created_at: now,
            generated_at: None,
        }
    }
}

/// 报表任务
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportTask {
    pub id: String,
    pub report_id: String,
    pub task_type: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ReportTask {
    #[must_use]
    pub fn new(report_id: String, task_type: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            report_id,
            task_type,
            status: "pending".to_string(),
            result: None,
            error_message: None,
            started_at: None,
            completed_at: None,
        }
    }
}

/// 定时任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScheduledTaskStatus {
    #[default]
    Active,
    Paused,
    Stopped,
}

impl std::fmt::Display for ScheduledTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// 定时任务
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub cron_expression: String,
    pub action_type: String,
    pub action_params: Option<serde_json::Value>,
    pub status: String,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
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
            id: Uuid::new_v4().to_string(),
            name,
            cron_expression,
            action_type,
            action_params,
            status: "active".to_string(),
            last_run_at: None,
            next_run_at: None,
            created_by,
            created_at: Utc::now(),
        }
    }
}

/// 创建工作流请求
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 更新工作流请求
#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub definition: Option<serde_json::Value>,
    pub status: Option<String>,
}

/// 创建节点请求
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub name: String,
    pub node_type: String,
    pub position_x: f64,
    pub position_y: f64,
    pub config: Option<serde_json::Value>,
    pub timeout: Option<i32>,
    pub auto_complete: Option<bool>,
}

/// 更新节点请求
#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub timeout: Option<i32>,
    pub auto_complete: Option<bool>,
}

/// 创建边请求
#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: Option<String>,
    pub condition: Option<String>,
    pub label: Option<String>,
}

/// 执行动作请求
#[derive(Debug, Deserialize)]
pub struct ExecuteActionRequest {
    pub action: String, // approve, reject, reassign, cancel
    pub comment: Option<String>,
    pub assignee: Option<String>,
    pub variables: Option<serde_json::Value>,
}

/// 更新边请求
#[derive(Debug, Deserialize)]
pub struct UpdateEdgeRequest {
    pub edge_type: Option<String>,
    pub condition: Option<String>,
    pub label: Option<String>,
    pub priority: Option<i32>,
}

/// 启动实例请求
#[derive(Debug, Deserialize)]
pub struct StartInstanceRequest {
    pub variables: Option<serde_json::Value>,
    pub start_node_id: Option<String>,
}

/// 列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

impl<T> PaginatedResponse<T> {
    #[must_use]
    pub const fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        Self {
            items,
            total,
            page,
            page_size,
        }
    }
}
