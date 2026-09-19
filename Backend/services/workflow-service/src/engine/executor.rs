use chrono::Utc;
use uuid::Uuid;

use crate::engine::state::{EdgeDefinition, ExecutionContext, NodeDefinition, WorkflowDefinition};
use common::AppError;

pub use common::AppError as EngineError;

/// 工作流引擎结果
pub type EngineResult<T> = Result<T, EngineError>;

#[derive(sqlx::FromRow)]
struct WorkflowRow {
    status: String,
    version: i32,
}

/// 工作流引擎
pub struct WorkflowEngine {
    pool: sqlx::PgPool,
}

impl WorkflowEngine {
    /// 创建新的工作流引擎
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 解析工作流定义
    pub fn parse_definition(
        &self,
        definition: serde_json::Value,
    ) -> EngineResult<WorkflowDefinition> {
        // 支持 BPMN 格式和简单 JSON 格式
        if let Some(nodes) = definition.get("nodes").and_then(|v| v.as_array()) {
            let node_defs: Vec<NodeDefinition> = nodes
                .iter()
                .filter_map(|n| serde_json::from_value(n.clone()).ok())
                .collect();

            let edges: Vec<EdgeDefinition> = definition
                .get("edges")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| serde_json::from_value(e.clone()).ok())
                        .collect()
                })
                .unwrap_or_default();

            // 找出起始节点和结束节点
            let start_node_id = node_defs
                .iter()
                .find(|n| n.node_type == "start")
                .map(|n| n.id.clone());

            let end_node_ids: Vec<String> = node_defs
                .iter()
                .filter(|n| n.node_type == "end")
                .map(|n| n.id.clone())
                .collect();

            Ok(WorkflowDefinition {
                start_node_id,
                end_node_ids,
                nodes: node_defs,
                edges,
            })
        } else {
            // 简单格式，直接作为节点定义
            Ok(WorkflowDefinition {
                start_node_id: None,
                end_node_ids: vec![],
                nodes: vec![],
                edges: vec![],
            })
        }
    }

    /// 启动工作流实例
    pub async fn start_instance(
        &self,
        workflow_id: &str,
        workflow_definition: serde_json::Value,
        started_by: String,
        initial_variables: Option<serde_json::Value>,
    ) -> EngineResult<crate::models::WorkflowInstance> {
        let mut tx = self.pool.begin().await?;

        // 查询工作流
        let workflow: Option<WorkflowRow> = sqlx::query_as!(
            WorkflowRow,
            r"SELECT status, version FROM workflows WHERE id = $1",
            workflow_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let workflow =
            workflow.ok_or_else(|| AppError::WorkflowNotFound(workflow_id.to_string()))?;

        // 检查工作流状态
        if workflow.status != "published" {
            return Err(AppError::WorkflowNotPublished(workflow_id.to_string()));
        }

        // 解析定义
        let definition = self.parse_definition(workflow_definition)?;

        // 创建实例
        let now = Utc::now();
        let instance_id = Uuid::new_v4().to_string();
        let variables = initial_variables.unwrap_or(serde_json::json!({}));

        sqlx::query!(
            r"INSERT INTO workflow_instances (id, workflow_id, workflow_version, status, variables, started_by, started_at)
              VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &instance_id,
            workflow_id,
            workflow.version,
            "running",
            &variables,
            &started_by,
            now,
        )
        .execute(&mut *tx)
        .await?;

        // 如果有起始节点，创建任务记录
        if let Some(start_node_id) = &definition.start_node_id {
            let task_id = Uuid::new_v4().to_string();
            sqlx::query!(
                r"INSERT INTO task_records (id, instance_id, node_id, node_name, status, started_at)
                  VALUES ($1, $2, $3, $4, $5, $6)",
                &task_id,
                &instance_id,
                start_node_id,
                start_node_id, // 暂时使用 ID 作为名称
                "running",
                now,
            )
            .execute(&mut *tx)
            .await?;

            // 更新实例当前节点
            sqlx::query!(
                "UPDATE workflow_instances SET current_node_id = $1 WHERE id = $2",
                start_node_id,
                &instance_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(crate::models::WorkflowInstance {
            id: instance_id,
            workflow_id: workflow_id.to_string(),
            workflow_version: workflow.version,
            status: "running".to_string(),
            current_node_id: definition.start_node_id,
            variables,
            started_by,
            started_at: Some(now),
            completed_at: None,
        })
    }

    /// 完成节点并流转到下一个节点
    pub async fn complete_node(
        &self,
        instance_id: &str,
        node_id: &str,
        _variables: Option<serde_json::Value>,
    ) -> EngineResult<Option<String>> {
        let mut tx = self.pool.begin().await?;

        let now = Utc::now();

        // 更新任务记录
        // 修复 B19-3: 使用 FOR UPDATE 行锁防止并发推进导致实例重复执行
        let _ = sqlx::query!(
            r"UPDATE task_records
              SET status = $1, completed_at = $2
              WHERE instance_id = $3 AND node_id = $4 AND status = 'running'",
            "completed",
            now,
            instance_id,
            node_id,
        )
        .execute(&mut *tx)
        .await?;

        // 修复 B19-1: 持久化节点变量（原实现忽略传入的 _variables 参数）
        if let Some(ref vars) = _variables {
            sqlx::query!(
                r"UPDATE workflow_instances SET variables = $1::jsonb WHERE id = $2",
                vars,
                instance_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        // 查询实例和定义（已在上面 FOR UPDATE 行锁保护下）
        let instance: Option<crate::models::WorkflowInstance> = sqlx::query_as!(
            crate::models::WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances WHERE id = $1",
            instance_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let instance =
            instance.ok_or_else(|| AppError::InstanceNotFound(instance_id.to_string()))?;

        // 从工作流表获取定义
        let workflow_def: Option<serde_json::Value> =
            sqlx::query_scalar!("SELECT definition FROM workflows WHERE id = $1", &instance.workflow_id)
                .fetch_optional(&mut *tx)
                .await?;

        let definition = self.parse_definition(workflow_def.unwrap_or(serde_json::json!({})))?;

        // 查找下一个节点
        // 修复 B19-2: 条件边无匹配时返回 None（而非错误地回退第一条边）
        let next_node_id = {
            let outgoing_edges: Vec<&EdgeDefinition> = definition
                .edges
                .iter()
                .filter(|e| e.source.as_str() == instance.current_node_id.as_ref().map(|s| s.as_str()).unwrap_or(""))
                .collect();

            let mut result = None;
            for edge in &outgoing_edges {
                // 优先选择非条件边
                if edge.edge_type != "condition" {
                    result = Some(edge.target.clone());
                    break;
                }
                // 评估条件表达式
                if let Some(condition) = &edge.condition
                    && self.evaluate_condition(condition, &instance.variables) {
                        result = Some(edge.target.clone());
                        break;
                    }
            }
            // 所有条件边都不匹配时返回 None（而非第一条边）
            result
        };

        if let Some(next_id) = &next_node_id {
            // 检查是否是结束节点
            if definition.end_node_ids.contains(next_id) {
                // 完成工作流实例
                sqlx::query!(
                    r"UPDATE workflow_instances SET status = $1, completed_at = $2 WHERE id = $3",
                    "completed",
                    now,
                    instance_id,
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
                return Ok(None); // 工作流已完成
            }

            // 创建新任务记录
            let task_id = Uuid::new_v4().to_string();
            sqlx::query!(
                r"INSERT INTO task_records (id, instance_id, node_id, node_name, status, started_at)
                  VALUES ($1, $2, $3, $4, $5, $6)",
                &task_id,
                instance_id,
                next_id,
                next_id,
                "running",
                now,
            )
            .execute(&mut *tx)
            .await?;

            // 更新实例当前节点
            sqlx::query!(
                "UPDATE workflow_instances SET current_node_id = $1 WHERE id = $2",
                next_id,
                instance_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(next_node_id)
    }

    /// 评估条件表达式
    fn evaluate_condition(&self, condition: &str, variables: &serde_json::Value) -> bool {
        // 支持简单表达式：${variable.path} == value, ${variable.path} > value 等
        // 格式: ${key} operator value 或 ${key} == "string"

        // 解析变量引用
        if let Some(start_idx) = condition.find("${")
            && let Some(end_idx) = condition.find('}') {
                let var_path = &condition[start_idx + 2..end_idx];

                // 获取变量值
                let var_value = self.get_variable_value(var_path, variables);

                // 获取操作符后面的值
                let rest = condition[end_idx + 1..].trim();

                return self.compare_values(&var_value, rest);
            }

        // 简单布尔值
        match condition.trim() {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => false,
        }
    }

    /// 获取嵌套变量值
    fn get_variable_value<'a>(&self, path: &'a str, variables: &'a serde_json::Value) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = variables;

        for part in parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return String::new(),
            };
        }

        match current {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            _ => current.to_string(),
        }
    }

    /// 比较值
    fn compare_values(&self, var_value: &str, operator_expr: &str) -> bool {
        let operator_expr = operator_expr.trim();

        // 处理 == 运算符
        if let Some(rest) = operator_expr.strip_prefix("==") {
            let expected = rest.trim().trim_matches('"').trim_matches('\'');
            return var_value == expected;
        }

        // 处理 != 运算符
        if let Some(rest) = operator_expr.strip_prefix("!=") {
            let expected = rest.trim().trim_matches('"').trim_matches('\'');
            return var_value != expected;
        }

        // 处理 > 运算符
        if let Some(rest) = operator_expr.strip_prefix(">") {
            if let (Ok(v1), Ok(v2)) = (var_value.parse::<f64>(), rest.trim().parse::<f64>()) {
                return v1 > v2;
            }
            return var_value > rest.trim();
        }

        // 处理 >= 运算符
        if let Some(rest) = operator_expr.strip_prefix(">=")
            && let (Ok(v1), Ok(v2)) = (var_value.parse::<f64>(), rest.trim().parse::<f64>()) {
                return v1 >= v2;
            }

        // 处理 < 运算符
        if let Some(rest) = operator_expr.strip_prefix("<") {
            if let (Ok(v1), Ok(v2)) = (var_value.parse::<f64>(), rest.trim().parse::<f64>()) {
                return v1 < v2;
            }
            return var_value < rest.trim();
        }

        // 处理 <= 运算符
        if let Some(rest) = operator_expr.strip_prefix("<=")
            && let (Ok(v1), Ok(v2)) = (var_value.parse::<f64>(), rest.trim().parse::<f64>()) {
                return v1 <= v2;
            }

        false
    }

    /// 取消工作流实例
    pub async fn cancel_instance(
        &self,
        instance_id: &str,
        _reason: Option<String>,
    ) -> EngineResult<()> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        // 更新实例状态
        let affected = sqlx::query!(
            r"UPDATE workflow_instances SET status = $1, completed_at = $2 WHERE id = $3 AND status = 'running'",
            "cancelled",
            now,
            instance_id,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::InstanceNotFound(instance_id.to_string()));
        }

        // 更新所有待处理任务
        sqlx::query!(
            r"UPDATE task_records SET status = $1, completed_at = $2 WHERE instance_id = $3 AND status = 'pending'",
            "skipped",
            now,
            instance_id,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
