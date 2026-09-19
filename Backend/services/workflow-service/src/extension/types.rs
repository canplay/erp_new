use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NodeType {
    /// 开始节点
    Start,
    /// 结束节点
    End,
    /// 任务节点
    #[default]
    Task,
    /// 条件节点
    Condition,
    /// 并行分支
    Parallel,
    /// 串行分支
    Serial,
    /// 循环节点
    Loop,
    /// 子流程节点
    SubProcess,
    /// 人工审批节点
    Approval,
    /// 自动脚本节点
    Script,
    /// 通知节点
    Notification,
    /// 自定义节点
    Custom,
}

/// 节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NodeState {
    /// 草稿
    #[default]
    Draft,
    /// 已激活
    Active,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已暂停
    Paused,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 工作流节点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub node_id: String,
    pub workflow_id: String,
    pub name: String,
    pub node_type: NodeType,
    pub state: NodeState,
    /// 节点配置（JSON）
    pub config: HashMap<String, String>,
    /// 输入参数
    pub inputs: Vec<Port>,
    /// 输出参数
    pub outputs: Vec<Port>,
    /// 执行超时（秒）
    pub timeout_seconds: Option<u64>,
    /// 重试次数
    pub retry_count: u32,
    /// 重试间隔（秒）
    pub retry_interval: u32,
    /// 自定义属性
    pub properties: HashMap<String, String>,
    /// 位置信息
    pub position_x: f64,
    pub position_y: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 节点端口（输入/输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub port_id: String,
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// 节点连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub connection_id: String,
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
    /// 条件表达式（用于条件分支）
    pub condition: Option<String>,
    pub priority: u32,
}

/// 节点执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionContext {
    pub execution_id: String,
    pub workflow_id: String,
    pub node_id: String,
    pub inputs: HashMap<String, String>,
    pub outputs: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// 节点扩展配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExtensionConfig {
    pub node_type: NodeType,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    /// 输入端口定义
    pub input_ports: Vec<PortDefinition>,
    /// 输出端口定义
    pub output_ports: Vec<PortDefinition>,
    /// 配置参数定义
    pub config_params: Vec<ConfigParam>,
    /// 验证规则
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDefinition {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigParam {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub options: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: String,
    pub expression: String,
    pub error_message: String,
}

/// 节点执行器接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutor {
    pub executor_id: String,
    pub node_type: NodeType,
    pub name: String,
    pub handler: String,
    pub config: HashMap<String, String>,
    pub enabled: bool,
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// 节点统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub total_nodes: usize,
    pub nodes_by_type: HashMap<String, usize>,
    pub total_executions: usize,
    pub running_executions: usize,
    pub completed_executions: usize,
    pub failed_executions: usize,
}
