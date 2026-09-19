use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::extension::types::{NodeState, NodeType};
use crate::extension::config::generate_id;

/// 节点类型扩展
pub trait NodeExtension: Send + Sync {
    fn node_type(&self) -> NodeType;
    fn display_name(&self) -> String;
    fn description(&self) -> String;
    fn validate_config(&self, config: &HashMap<String, String>) -> Result<(), String>;
    fn default_timeout(&self) -> Option<u64> {
        None
    }
    fn icon(&self) -> String {
        "default".to_string()
    }
}

/// 工作流节点管理器
pub struct WorkflowNodeManager {
    nodes: HashMap<String, crate::extension::types::WorkflowNode>,
    extensions: HashMap<NodeType, Box<dyn NodeExtension>>,
}

impl WorkflowNodeManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    /// 注册节点扩展
    pub fn register_extension(&mut self, ext: Box<dyn NodeExtension>) -> bool {
        self.extensions.insert(ext.node_type(), ext);
        true
    }

    /// 获取节点扩展
    pub fn get_extension(&self, node_type: NodeType) -> Option<&dyn NodeExtension> {
        self.extensions.get(&node_type).map(|b| b.as_ref())
    }

    /// 创建工作流节点
    pub fn create_node(
        &mut self,
        workflow_id: &str,
        name: &str,
        node_type: NodeType,
        config: HashMap<String, String>,
        position_x: f64,
        position_y: f64,
    ) -> crate::extension::types::WorkflowNode {
        let node_id = format!("{}_{}", workflow_id, generate_id());

        let node = crate::extension::types::WorkflowNode {
            node_id: node_id.clone(),
            workflow_id: workflow_id.to_string(),
            name: name.to_string(),
            node_type,
            state: NodeState::Draft,
            config,
            inputs: Vec::new(),
            outputs: Vec::new(),
            timeout_seconds: None,
            retry_count: 0,
            retry_interval: 60,
            properties: HashMap::new(),
            position_x,
            position_y,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.nodes.insert(node_id, node.clone());
        node
    }

    /// 获取节点
    pub fn get_node(&self, node_id: &str) -> Option<&crate::extension::types::WorkflowNode> {
        self.nodes.get(node_id)
    }

    /// 更新节点
    pub fn update_node(
        &mut self,
        node_id: &str,
        name: Option<String>,
        config: Option<HashMap<String, String>>,
        state: Option<NodeState>,
    ) -> bool {
        if let Some(node) = self.nodes.get_mut(node_id) {
            if let Some(n) = name {
                node.name = n;
            }
            if let Some(c) = config {
                node.config.extend(c);
            }
            if let Some(s) = state {
                node.state = s;
            }
            node.updated_at = Utc::now();
            return true;
        }
        false
    }

    /// 删除节点
    pub fn delete_node(&mut self, node_id: &str) -> bool {
        self.nodes.remove(node_id).is_some()
    }

    /// 获取工作流的所有节点
    pub fn get_workflow_nodes(&self, workflow_id: &str) -> Vec<&crate::extension::types::WorkflowNode> {
        self.nodes
            .values()
            .filter(|n| n.workflow_id == workflow_id)
            .collect()
    }

    /// 验证节点配置
    pub fn validate_node(&self, node_id: &str) -> Vec<crate::extension::types::ValidationError> {
        let mut errors = Vec::new();

        if let Some(node) = self.nodes.get(node_id) {
            if let Some(ext) = self.extensions.get(&node.node_type) {
                if let Err(e) = ext.validate_config(&node.config) {
                    errors.push(crate::extension::types::ValidationError {
                        field: "config".to_string(),
                        message: e,
                    });
                }
            }
        } else {
            errors.push(crate::extension::types::ValidationError {
                field: "node".to_string(),
                message: "节点不存在".to_string(),
            });
        }

        errors
    }
}

impl Default for WorkflowNodeManager {
    fn default() -> Self {
        Self::new()
    }
}
