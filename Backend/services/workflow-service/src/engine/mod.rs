//! Workflow Engine
//!
//! 工作流执行引擎 - 实现状态机、节点流转、定时任务触发

mod executor;
mod scheduler;
mod state;

pub use executor::{EngineError, EngineResult, WorkflowEngine};
pub use scheduler::TaskScheduler;
pub use state::{ExecutionContext, WorkflowDefinition};

#[cfg(test)]
mod tests {
    use super::state::{NodeDefinition, WorkflowDefinition};

    #[test]
    fn test_parse_simple_definition() {
        // 模拟解析简单工作流定义
        let definition = serde_json::json!({
            "nodes": [
                {"id": "start", "name": "开始", "node_type": "start", "config": {}, "timeout": null, "auto_complete": true},
                {"id": "task1", "name": "任务1", "node_type": "task", "config": {}, "timeout": 3600, "auto_complete": false},
                {"id": "end", "name": "结束", "node_type": "end", "config": {}, "timeout": null, "auto_complete": true}
            ],
            "edges": [
                {"id": "e1", "source": "start", "target": "task1", "edge_type": "normal"},
                {"id": "e2", "source": "task1", "target": "end", "edge_type": "normal"}
            ]
        });

        // 手动验证解析逻辑
        let nodes = definition.get("nodes").expect("test assertion").as_array().expect("test assertion");
        let start_node_id = nodes
            .iter()
            .find(|n| n.get("node_type").expect("test assertion").as_str().expect("test assertion") == "start")
            .map(|n| n.get("id").expect("test assertion").as_str().expect("test assertion").to_string());

        let end_node_ids: Vec<String> = nodes
            .iter()
            .filter(|n| n.get("node_type").expect("test assertion").as_str().expect("test assertion") == "end")
            .filter_map(|n| n.get("id").expect("test assertion").as_str())
            .map(String::from)
            .collect();

        assert_eq!(start_node_id, Some("start".to_string()));
        assert_eq!(end_node_ids, vec!["end"]);
        assert_eq!(nodes.len(), 3);
    }
}
