use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::extension::config::generate_id;
use crate::extension::manager::NodeExtension;
use crate::extension::types::*;

/// 节点管理器 (async version with in-memory state)
pub struct NodeManager {
    /// 节点定义缓存
    nodes: Arc<RwLock<HashMap<String, WorkflowNode>>>,
    /// 节点连接
    connections: Arc<RwLock<HashMap<String, Vec<NodeConnection>>>>,
    /// 节点扩展配置
    extensions: Arc<RwLock<HashMap<NodeType, NodeExtensionConfig>>>,
    /// 节点执行器
    executors: Arc<RwLock<HashMap<NodeType, NodeExecutor>>>,
    /// 节点执行记录
    executions: Arc<RwLock<HashMap<String, NodeExecutionContext>>>,
}

impl NodeManager {
    /// 创建新的 `NodeManager`
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            extensions: Arc::new(RwLock::new(HashMap::new())),
            executors: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认管理器
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new()
    }

    /// 注册节点扩展配置
    pub async fn register_extension(&self, config: NodeExtensionConfig) -> bool {
        let mut extensions = self.extensions.write().await;
        extensions.insert(config.node_type, config);
        true
    }

    /// 获取节点扩展配置
    pub async fn get_extension(&self, node_type: NodeType) -> Option<NodeExtensionConfig> {
        let extensions = self.extensions.read().await;
        extensions.get(&node_type).cloned()
    }

    /// 注册节点执行器
    pub async fn register_executor(&self, executor: NodeExecutor) -> bool {
        let mut executors = self.executors.write().await;
        executors.insert(executor.node_type, executor);
        true
    }

    /// 获取节点执行器
    pub async fn get_executor(&self, node_type: NodeType) -> Option<NodeExecutor> {
        let executors = self.executors.read().await;
        executors.get(&node_type).cloned()
    }

    /// 创建工作流节点
    pub async fn create_node(
        &self,
        workflow_id: &str,
        name: &str,
        node_type: NodeType,
        config: HashMap<String, String>,
        position_x: f64,
        position_y: f64,
    ) -> WorkflowNode {
        let mut nodes = self.nodes.write().await;

        let node_id = format!("{}_{}", workflow_id, generate_id());

        let extension = self.extensions.read().await.get(&node_type).cloned();

        let (inputs, outputs) = if let Some(ext) = extension {
            (
                ext.input_ports
                    .iter()
                    .map(|p| Port {
                        port_id: format!("{}_in_{}", node_id, p.name),
                        name: p.name.clone(),
                        data_type: p.data_type.clone(),
                        required: p.required,
                        default_value: None,
                    })
                    .collect(),
                ext.output_ports
                    .iter()
                    .map(|p| Port {
                        port_id: format!("{}_out_{}", node_id, p.name),
                        name: p.name.clone(),
                        data_type: p.data_type.clone(),
                        required: p.required,
                        default_value: None,
                    })
                    .collect(),
            )
        } else {
            (Vec::new(), Vec::new())
        };

        let node = WorkflowNode {
            node_id: node_id.clone(),
            workflow_id: workflow_id.to_string(),
            name: name.to_string(),
            node_type,
            state: NodeState::Draft,
            config,
            inputs,
            outputs,
            timeout_seconds: None,
            retry_count: 0,
            retry_interval: 60,
            properties: HashMap::new(),
            position_x,
            position_y,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        nodes.insert(node_id, node.clone());
        node
    }

    /// 获取节点
    pub async fn get_node(&self, node_id: &str) -> Option<WorkflowNode> {
        let nodes = self.nodes.read().await;
        nodes.get(node_id).cloned()
    }

    /// 更新节点
    pub async fn update_node(
        &self,
        node_id: &str,
        name: Option<String>,
        config: Option<HashMap<String, String>>,
        state: Option<NodeState>,
    ) -> bool {
        let mut nodes = self.nodes.write().await;

        if let Some(node) = nodes.get_mut(node_id) {
            if let Some(n) = name {
                node.name = n;
            }
            if let Some(c) = config {
                node.config.extend(c);
            }
            if let Some(s) = state {
                node.state = s;
            }
            node.updated_at = chrono::Utc::now();
            return true;
        }
        false
    }

    /// 删除节点
    pub async fn delete_node(&self, node_id: &str) -> bool {
        let mut nodes = self.nodes.write().await;
        nodes.remove(node_id).is_some()
    }

    /// 获取工作流的所有节点
    pub async fn get_workflow_nodes(&self, workflow_id: &str) -> Vec<WorkflowNode> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| n.workflow_id == workflow_id)
            .cloned()
            .collect()
    }

    /// 创建节点连接
    pub async fn create_connection(
        &self,
        source_node_id: &str,
        source_port_id: &str,
        target_node_id: &str,
        target_port_id: &str,
        condition: Option<String>,
    ) -> Option<NodeConnection> {
        let nodes = self.nodes.read().await;

        if !nodes.contains_key(source_node_id) || !nodes.contains_key(target_node_id) {
            return None;
        }

        let connection = NodeConnection {
            connection_id: generate_id(),
            source_node_id: source_node_id.to_string(),
            source_port_id: source_port_id.to_string(),
            target_node_id: target_node_id.to_string(),
            target_port_id: target_port_id.to_string(),
            condition,
            priority: 0,
        };

        let mut connections = self.connections.write().await;
        let node_connections = connections
            .entry(source_node_id.to_string())
            .or_insert_with(Vec::new);
        node_connections.push(connection.clone());

        Some(connection)
    }

    /// 获取节点的输出连接
    pub async fn get_outgoing_connections(&self, node_id: &str) -> Vec<NodeConnection> {
        let connections = self.connections.read().await;
        connections.get(node_id).cloned().unwrap_or_default()
    }

    /// 获取节点的后继节点
    pub async fn get_successor_nodes(&self, node_id: &str) -> Vec<String> {
        let connections = self.connections.read().await;
        connections
            .get(node_id)
            .map(|conns| conns.iter().map(|c| c.target_node_id.clone()).collect())
            .unwrap_or_default()
    }

    /// 获取节点的先驱节点
    pub async fn get_predecessor_nodes(&self, node_id: &str) -> Vec<String> {
        let connections = self.connections.read().await;
        let mut predecessors = Vec::new();

        for (source, conns) in connections.iter() {
            for conn in conns {
                if conn.target_node_id == node_id {
                    predecessors.push(source.clone());
                }
            }
        }

        predecessors
    }

    /// 验证节点配置
    pub async fn validate_node(&self, node_id: &str) -> Vec<ValidationError> {
        let nodes = self.nodes.read().await;
        let mut errors = Vec::new();

        if let Some(node) = nodes.get(node_id) {
            let extensions = self.extensions.read().await;
            if let Some(ext) = extensions.get(&node.node_type) {
                for param in &ext.config_params {
                    if param.required && !node.config.contains_key(&param.name) {
                        errors.push(ValidationError {
                            field: param.name.clone(),
                            message: format!("缺少必需参数: {}", param.name),
                        });
                    }
                }

                for rule in &ext.validation_rules {
                    if let Some(value) = node.config.get(&format!("{}_expr", rule.rule_type))
                        && *value != rule.expression {
                            errors.push(ValidationError {
                                field: rule.rule_type.clone(),
                                message: rule.error_message.clone(),
                            });
                        }
                }
            }
        } else {
            errors.push(ValidationError {
                field: "node".to_string(),
                message: "节点不存在".to_string(),
            });
        }

        errors
    }

    /// 记录节点执行
    pub async fn start_execution(
        &self,
        execution_id: &str,
        workflow_id: &str,
        node_id: &str,
        inputs: HashMap<String, String>,
    ) -> Option<NodeExecutionContext> {
        let nodes = self.nodes.read().await;
        if !nodes.contains_key(node_id) {
            return None;
        }

        let context = NodeExecutionContext {
            execution_id: execution_id.to_string(),
            workflow_id: workflow_id.to_string(),
            node_id: node_id.to_string(),
            inputs,
            outputs: HashMap::new(),
            variables: HashMap::new(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            error: None,
        };

        let mut executions = self.executions.write().await;
        executions.insert(execution_id.to_string(), context.clone());

        Some(context)
    }

    /// 完成节点执行
    pub async fn complete_execution(
        &self,
        execution_id: &str,
        outputs: HashMap<String, String>,
    ) -> bool {
        let mut executions = self.executions.write().await;

        if let Some(context) = executions.get_mut(execution_id) {
            context.outputs = outputs;
            context.completed_at = Some(chrono::Utc::now());
            return true;
        }
        false
    }

    /// 获取节点执行记录
    pub async fn get_node_executions(&self, node_id: &str) -> Vec<NodeExecutionContext> {
        let executions = self.executions.read().await;
        executions
            .values()
            .filter(|e| e.node_id == node_id)
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> NodeStats {
        let nodes = self.nodes.read().await;
        let executions = self.executions.read().await;

        let total = nodes.len();
        let by_type: HashMap<String, usize> = nodes.values().fold(HashMap::new(), |mut acc, n| {
            *acc.entry(format!("{:?}", n.node_type)).or_insert(0) += 1;
            acc
        });

        let running = executions
            .values()
            .filter(|e| e.started_at.is_some() && e.completed_at.is_none())
            .count();

        let completed = executions
            .values()
            .filter(|e| e.completed_at.is_some() && e.error.is_none())
            .count();

        let failed = executions.values().filter(|e| e.error.is_some()).count();

        NodeStats {
            total_nodes: total,
            nodes_by_type: by_type,
            total_executions: executions.len(),
            running_executions: running,
            completed_executions: completed,
            failed_executions: failed,
        }
    }
}

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_extension() {
        let manager = NodeManager::default_manager();

        let config = NodeExtensionConfig {
            node_type: NodeType::Custom,
            display_name: "自定义节点".to_string(),
            description: "支持自定义逻辑".to_string(),
            icon: "custom".to_string(),
            input_ports: vec![PortDefinition {
                name: "input".to_string(),
                data_type: "string".to_string(),
                required: true,
                description: None,
            }],
            output_ports: vec![PortDefinition {
                name: "output".to_string(),
                data_type: "string".to_string(),
                required: true,
                description: None,
            }],
            config_params: vec![ConfigParam {
                name: "handler".to_string(),
                param_type: "string".to_string(),
                required: true,
                default_value: None,
                options: None,
                description: Some("处理函数".to_string()),
            }],
            validation_rules: vec![],
        };

        let result = manager.register_extension(config).await;
        assert!(result);

        let ext = manager.get_extension(NodeType::Custom).await;
        assert!(ext.is_some());
        assert_eq!(ext.expect("test assertion").display_name, "自定义节点");
    }

    #[tokio::test]
    async fn test_create_node() {
        let manager = NodeManager::default_manager();

        let node = manager
            .create_node(
                "workflow_001",
                "测试节点",
                NodeType::Task,
                HashMap::new(),
                100.0,
                200.0,
            )
            .await;

        assert_eq!(node.workflow_id, "workflow_001");
        assert_eq!(node.name, "测试节点");
        assert_eq!(node.node_type, NodeType::Task);
        assert_eq!(node.state, NodeState::Draft);

        let fetched = manager.get_node(&node.node_id).await;
        assert!(fetched.is_some());
    }

    #[tokio::test]
    async fn test_update_node() {
        let manager = NodeManager::default_manager();

        let node = manager
            .create_node(
                "workflow_001",
                "节点1",
                NodeType::Task,
                HashMap::new(),
                0.0,
                0.0,
            )
            .await;

        let mut config = HashMap::new();
        config.insert("timeout".to_string(), "60".to_string());

        let updated = manager
            .update_node(
                &node.node_id,
                Some("更新后节点".to_string()),
                Some(config),
                Some(NodeState::Active),
            )
            .await;

        assert!(updated);

        let fetched = manager.get_node(&node.node_id).await;
        let fetched_node = fetched.as_ref().expect("test assertion");
        assert_eq!(fetched_node.name, "更新后节点");
        assert_eq!(fetched_node.state, NodeState::Active);
    }

    #[tokio::test]
    async fn test_connections() {
        let manager = NodeManager::default_manager();

        let node1 = manager
            .create_node("wf1", "开始", NodeType::Start, HashMap::new(), 0.0, 0.0)
            .await;
        let node2 = manager
            .create_node("wf1", "任务", NodeType::Task, HashMap::new(), 100.0, 0.0)
            .await;
        let node3 = manager
            .create_node("wf1", "结束", NodeType::End, HashMap::new(), 200.0, 0.0)
            .await;

        let conn1 = manager
            .create_connection(&node1.node_id, "out1", &node2.node_id, "in1", None)
            .await;
        let conn2 = manager
            .create_connection(&node2.node_id, "out1", &node3.node_id, "in1", None)
            .await;

        assert!(conn1.is_some());
        assert!(conn2.is_some());

        let successors = manager.get_successor_nodes(&node1.node_id).await;
        assert_eq!(successors.len(), 1);
        assert_eq!(successors[0], node2.node_id);

        let predecessors = manager.get_predecessor_nodes(&node3.node_id).await;
        assert_eq!(predecessors.len(), 1);
        assert_eq!(predecessors[0], node2.node_id);
    }

    #[tokio::test]
    async fn test_execution() {
        let manager = NodeManager::default_manager();

        let node = manager
            .create_node("wf1", "任务", NodeType::Task, HashMap::new(), 0.0, 0.0)
            .await;

        let mut inputs = HashMap::new();
        inputs.insert("param1".to_string(), "value1".to_string());

        let context = manager
            .start_execution("exec_001", "wf1", &node.node_id, inputs)
            .await;
        assert!(context.is_some());

        let mut outputs = HashMap::new();
        outputs.insert("result".to_string(), "success".to_string());

        let completed = manager.complete_execution("exec_001", outputs).await;
        assert!(completed);

        let executions = manager.get_node_executions(&node.node_id).await;
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].execution_id, "exec_001");
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = NodeManager::default_manager();

        manager
            .create_node("wf1", "开始", NodeType::Start, HashMap::new(), 0.0, 0.0)
            .await;
        manager
            .create_node("wf1", "任务1", NodeType::Task, HashMap::new(), 100.0, 0.0)
            .await;
        manager
            .create_node("wf1", "任务2", NodeType::Task, HashMap::new(), 200.0, 0.0)
            .await;
        manager
            .create_node("wf1", "结束", NodeType::End, HashMap::new(), 300.0, 0.0)
            .await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_nodes, 4);
    }
}
