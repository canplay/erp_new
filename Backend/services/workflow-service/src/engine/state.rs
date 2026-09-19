use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 工作流定义解析后的结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 起始节点ID
    pub start_node_id: Option<String>,
    /// 结束节点ID列表
    pub end_node_ids: Vec<String>,
    /// 节点定义
    pub nodes: Vec<NodeDefinition>,
    /// 边定义
    pub edges: Vec<EdgeDefinition>,
}

/// 节点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub config: serde_json::Value,
    pub timeout: Option<i32>,
    pub auto_complete: bool,
}

/// 边定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefinition {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub condition: Option<String>,
}

/// 执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub instance_id: String,
    pub workflow_id: String,
    pub current_node_id: Option<String>,
    pub variables: serde_json::Value,
    pub started_by: String,
}

impl ExecutionContext {
    #[must_use]
    pub fn new(workflow_id: String, started_by: String) -> Self {
        Self {
            instance_id: Uuid::new_v4().to_string(),
            workflow_id,
            current_node_id: None,
            variables: serde_json::json!({}),
            started_by,
        }
    }
}

/// 工作流状态与定义（调度触发用）
#[derive(sqlx::FromRow)]
pub struct WorkflowMeta {
    pub status: String,
    pub definition: serde_json::Value,
}
