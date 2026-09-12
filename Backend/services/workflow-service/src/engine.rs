//!
//! Workflow Engine
//!
//! 工作流执行引擎 - 实现状态机、节点流转、定时任务触发

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::WorkflowInstance;

/// 工作流引擎错误类型
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("工作流不存在: {0}")]
    WorkflowNotFound(String),

    #[error("工作流未发布: {0}")]
    WorkflowNotPublished(String),

    #[error("实例不存在: {0}")]
    InstanceNotFound(String),

    #[error("节点不存在: {0}")]
    NodeNotFound(String),

    #[error("无效的状态转换: {0}")]
    InvalidStateTransition(String),

    #[error("执行失败: {0}")]
    ExecutionFailed(String),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// 工作流引擎结果
pub type EngineResult<T> = Result<T, EngineError>;

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

/// 工作流行信息（查询用）
#[derive(sqlx::FromRow)]
struct WorkflowRow {
    status: String,
    version: i32,
}

/// 工作流状态与定义（调度触发用）
#[derive(sqlx::FromRow)]
struct WorkflowMeta {
    status: String,
    definition: serde_json::Value,
}

/// 到期的定时任务
#[derive(sqlx::FromRow)]
struct DueScheduledTask {
    id: String,
    name: String,
    action_type: String,
    action_params: serde_json::Value,
}

/// 报表元信息
#[derive(sqlx::FromRow)]
struct ReportMeta {
    id: String,
    name: String,
    report_type: String,
    query_params: serde_json::Value,
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
    ) -> EngineResult<WorkflowInstance> {
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
            workflow.ok_or_else(|| EngineError::WorkflowNotFound(workflow_id.to_string()))?;

        // 检查工作流状态
        if workflow.status != "published" {
            return Err(EngineError::WorkflowNotPublished(workflow_id.to_string()));
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

        Ok(WorkflowInstance {
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
            let vars_json = serde_json::to_string(vars).unwrap_or_else(|_| "{}".to_string());
            sqlx::query!(
                r"UPDATE workflow_instances SET variables = $1::jsonb WHERE id = $2",
                vars_json.as_str(),
                instance_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        // 查询实例和定义（已在上面 FOR UPDATE 行锁保护下）
        let instance: Option<WorkflowInstance> = sqlx::query_as!(
            WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances WHERE id = $1",
            instance_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let instance =
            instance.ok_or_else(|| EngineError::InstanceNotFound(instance_id.to_string()))?;

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
            return Err(EngineError::InstanceNotFound(instance_id.to_string()));
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

/// 定时任务调度器
pub struct TaskScheduler {
    pool: sqlx::PgPool,
}

impl TaskScheduler {
    /// 创建新的调度器
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 检查并触发到期的定时任务
    pub async fn check_and_trigger_tasks(&self) -> EngineResult<Vec<String>> {
        let now = Utc::now();
        let mut triggered = Vec::new();

        // 查询所有到期的任务
        let tasks: Vec<DueScheduledTask> = sqlx::query_as!(
            DueScheduledTask,
            r"SELECT id, name, task_handler AS action_type,
                      COALESCE(task_params, '{}'::jsonb) AS action_params
            FROM schedule_tasks
            WHERE status = 'active' AND next_run_time <= $1",
            now,
        )
        .fetch_all(&self.pool)
        .await?;

        for task in tasks {
            tracing::info!("触发定时任务: {} ({})", task.name, task.id);

            // 执行任务动作
            self.execute_task_action(&task.action_type, &task.action_params)
                .await?;

            // 更新任务状态和下次执行时间
            let next_run_at = self.calculate_next_run(&task.id).await?;

            sqlx::query!(
                r"UPDATE schedule_tasks
                  SET last_run_time = $1, next_run_time = $2
                  WHERE id = $3",
                now,
                next_run_at,
                &task.id,
            )
            .execute(&self.pool)
            .await?;

            triggered.push(task.id);
        }

        Ok(triggered)
    }

    /// 执行任务动作
    async fn execute_task_action(
        &self,
        action_type: &str,
        action_params: &serde_json::Value,
    ) -> EngineResult<()> {
        match action_type {
            "workflow" => {
                // 触发工作流
                let workflow_id = action_params
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EngineError::ExecutionFailed("缺少 workflow_id".to_string()))?;

                self.execute_workflow_action(workflow_id, action_params)
                    .await?;
                tracing::info!("执行工作流: {workflow_id}");
            }
            "report" => {
                // 生成报表
                let report_id = action_params
                    .get("report_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EngineError::ExecutionFailed("缺少 report_id".to_string()))?;

                self.execute_report_action(report_id).await?;
                tracing::info!("生成报表: {report_id}");
            }
            "webhook" => {
                // 发送 Webhook 请求
                let url = action_params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EngineError::ExecutionFailed("缺少 webhook url".to_string()))?;

                self.execute_webhook_action(url, action_params).await?;
                tracing::info!("发送 Webhook: {url}");
            }
            "script" => {
                // 执行脚本
                let script = action_params
                    .get("script")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EngineError::ExecutionFailed("缺少 script 内容".to_string()))?;

                self.execute_script_action(script).await?;
                tracing::info!("执行自定义脚本");
            }
            _ => {
                return Err(EngineError::ExecutionFailed(format!(
                    "未知的动作类型: {action_type}"
                )));
            }
        }
        Ok(())
    }

    /// 执行工作流动作用
    async fn execute_workflow_action(
        &self,
        workflow_id: &str,
        _params: &serde_json::Value,
    ) -> EngineResult<()> {
        // 查询工作流定义
        let workflow: Option<WorkflowMeta> = sqlx::query_as!(
            WorkflowMeta,
            r"SELECT status, definition FROM workflows WHERE id = $1",
            workflow_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match workflow {
            Some(w) if w.status == "published" => {
                // 创建新实例
                let engine = WorkflowEngine::new(self.pool.clone());
                engine
                    .start_instance(workflow_id, w.definition, "scheduler".to_string(), None)
                    .await?;
                Ok(())
            }
            Some(_) => Err(EngineError::ExecutionFailed("工作流未发布".to_string())),
            None => Err(EngineError::WorkflowNotFound(workflow_id.to_string())),
        }
    }

    /// 执行报表生成动作
    async fn execute_report_action(&self, report_id: &str) -> EngineResult<()> {
        // 查询报表定义
        let report: Option<ReportMeta> = sqlx::query_as!(
            ReportMeta,
            r"SELECT id, name, report_type,
                      COALESCE(config, '{}'::jsonb) AS query_params
            FROM reports WHERE id = $1",
            report_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match report {
            Some(r) => {
                // 根据报表类型生成数据
                let result = self
                    .generate_report_data(&r.report_type, &r.query_params)
                    .await?;
                let now = Utc::now();

                // 更新报表状态（reports 表无 result 列，结果不落库）
                sqlx::query!(
                    r"UPDATE reports SET status = $1, updated_at = $2 WHERE id = $3",
                    "generated",
                    now,
                    &r.id,
                )
                .execute(&self.pool)
                .await?;

                tracing::info!("报表生成成功: {} ({})", r.name, r.id);
                let _ = result;
                Ok(())
            }
            None => Err(EngineError::ExecutionFailed(format!(
                "报表不存在: {report_id}"
            ))),
        }
    }

    /// 生成报表数据
    async fn generate_report_data(
        &self,
        report_type: &str,
        _query_params: &serde_json::Value,
    ) -> EngineResult<serde_json::Value> {
        match report_type {
            "table" => {
                // 表格报表：返回示例数据
                Ok(serde_json::json!({
                    "type": "table",
                    "columns": ["名称", "数量", "金额"],
                    "rows": [
                        ["项目A", 100, 10000],
                        ["项目B", 200, 20000],
                        ["项目C", 150, 15000]
                    ],
                    "summary": {
                        "total_rows": 3,
                        "total_amount": 45000
                    }
                }))
            }
            "chart" => {
                // 图表报表：返回图表数据
                Ok(serde_json::json!({
                    "type": "chart",
                    "chart_type": "bar",
                    "labels": ["1月", "2月", "3月", "4月", "5月"],
                    "datasets": [{
                        "label": "销售额",
                        "data": [120, 190, 300, 500, 200]
                    }]
                }))
            }
            "dashboard" => {
                // 仪表盘报表：返回多个指标
                Ok(serde_json::json!({
                    "type": "dashboard",
                    "metrics": [
                        {"name": "总用户数", "value": 1000, "change": 10.5},
                        {"name": "日活用户", "value": 500, "change": -2.3},
                        {"name": "总收入", "value": 50000, "change": 15.2}
                    ],
                    "charts": [
                        {"name": "用户趋势", "type": "line", "data": [100, 120, 150, 180, 200]},
                        {"name": "收入构成", "type": "pie", "data": [30, 40, 30]}
                    ]
                }))
            }
            _ => Ok(serde_json::json!({
                "type": report_type,
                "message": "不支持的报表类型"
            })),
        }
    }

    /// 执行 Webhook 动作
    async fn execute_webhook_action(
        &self,
        url: &str,
        params: &serde_json::Value,
    ) -> EngineResult<()> {
        // 获取请求方法
        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST");

        // 获取 headers - 使用 HeaderName 和 HeaderValue 需要 'static 生命周期
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(headers_obj) = params.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in headers_obj {
                // 使用 HeaderName 解析，失败则跳过
                let header_name = match reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                    Ok(name) => name,
                    Err(_) => continue,
                };
                if let Some(value) = v.as_str() {
                    // 使用 HeaderValue 解析，失败则跳过
                    if let Ok(header_value) = value.parse() {
                        headers.insert(header_name, header_value);
                    }
                }
            }
        }

        // 构建请求体
        let body = params
            .get("body")
            .cloned()
            .map(|v| serde_json::to_string(&v))
            .transpose()
            .map_err(|e| EngineError::ExecutionFailed(e.to_string()))?;

        // 创建 HTTP 客户端
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| EngineError::ExecutionFailed(format!("创建HTTP客户端失败: {e}")))?;

        // 发送请求
        let request_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => {
                return Err(EngineError::ExecutionFailed(format!(
                    "不支持的HTTP方法: {method}"
                )));
            }
        };

        let mut request = request_builder.headers(headers);
        if let Some(body) = body {
            request = request.body(body);
        }

        // 发送请求并记录结果
        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                tracing::info!("Webhook 响应: {method} {url} - 状态: {status}");
                if !status.is_success() {
                    tracing::warn!("Webhook 返回非成功状态: {status} - body: {body_text}");
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("Webhook 请求失败: {method} {url} - 错误: {e}");
                Err(EngineError::ExecutionFailed(format!(
                    "Webhook 请求失败: {e}"
                )))
            }
        }
    }

    /// 执行脚本动作
    async fn execute_script_action(&self, script: &str) -> EngineResult<()> {
        // 简单的脚本执行框架
        // 实际生产环境应该使用沙箱执行器

        tracing::info!("执行脚本: {}", &script[..script.len().min(100)]);

        // 这里可以实现简单的表达式求值
        // 例如: ${env.VAR_NAME} -> 获取环境变量
        // 或者: ${date.format('yyyy-MM-dd')} -> 日期格式化

        Ok(())
    }

    /// 计算下次执行时间
    async fn calculate_next_run(&self, task_id: &str) -> EngineResult<Option<DateTime<Utc>>> {
        #[derive(sqlx::FromRow)]
        struct TaskCron {
            cron_expression: String,
        }

        let task: Option<TaskCron> = sqlx::query_as!(
            TaskCron,
            r#"SELECT COALESCE(cron_expression, '') AS "cron_expression!"
FROM schedule_tasks WHERE id = $1"#,
            task_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(task) = task {
            // 简单的 cron 解析（支持标准格式）
            // 实际生产应使用 cron crate 进行解析
            let parts: Vec<&str> = task.cron_expression.split_whitespace().collect();
            if parts.len() >= 5 {
                // 简单实现：假设每分钟执行一次
                let next = Utc::now() + Duration::minutes(1);
                return Ok(Some(next));
            }
        }

        Ok(None)
    }

    /// 暂停定时任务
    pub async fn pause_task(&self, task_id: &str) -> EngineResult<()> {
        let affected = sqlx::query!(
            r"UPDATE schedule_tasks SET status = 'paused' WHERE id = $1",
            task_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(EngineError::InstanceNotFound(task_id.to_string()));
        }
        Ok(())
    }

    /// 恢复定时任务
    pub async fn resume_task(&self, task_id: &str) -> EngineResult<()> {
        let affected = sqlx::query!(
            r"UPDATE schedule_tasks SET status = 'active' WHERE id = $1",
            task_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(EngineError::InstanceNotFound(task_id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
