//! Workflow 数据仓储层
//!
//! 实现工作流、实例、任务的数据库 CRUD 操作
//! 包含 `PostgreSQL` 实现和内存实现

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use parking_lot::RwLock;

// ============================================================================
// 数据模型
// ============================================================================

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
// Workflow Repository trait
// ============================================================================

/// Workflow Repository trait
pub trait WorkflowRepository: Send + Sync {
    // 工作流基本操作
    fn create(
        &self,
        wf: &Workflow,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        wf: &Workflow,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<Workflow>, sqlx::Error>> + Send;
    fn list(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<Workflow>, i64), sqlx::Error>> + Send;
    fn publish(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn save_nodes(
        &self,
        workflow_id: &str,
        nodes: &[WorkflowNode],
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn save_edges(
        &self,
        workflow_id: &str,
        edges: &[WorkflowEdge],
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn get_nodes(
        &self,
        workflow_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<WorkflowNode>, sqlx::Error>> + Send;
    fn get_edges(
        &self,
        workflow_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<WorkflowEdge>, sqlx::Error>> + Send;

    // 实例操作（grpc_handlers 需要）
    fn create_instance(
        &self,
        inst: &WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update_instance(
        &self,
        inst: &WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn get_instance(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<WorkflowInstance>, sqlx::Error>> + Send;
    fn list_instances(
        &self,
        workflow_id: Option<&str>,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<WorkflowInstance>, i64), sqlx::Error>> + Send;

    // 任务操作（grpc_handlers 需要）
    fn create_task(
        &self,
        task: &TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update_task(
        &self,
        task: &TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn get_task(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<TaskRecord>, sqlx::Error>> + Send;
    fn list_tasks(
        &self,
        instance_id: Option<&str>,
        assignee: Option<&str>,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<TaskRecord>, i64), sqlx::Error>> + Send;
    fn list_tasks_by_assignee(
        &self,
        assignee: &str,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<TaskRecord>, i64), sqlx::Error>> + Send;
    fn get_task_history(
        &self,
        instance_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TaskRecord>, sqlx::Error>> + Send;
}

/// `InMemory` Workflow Repository
pub struct InMemoryWorkflowRepository {
    workflows: RwLock<HashMap<String, Workflow>>,
    instances: RwLock<HashMap<String, WorkflowInstance>>,
    tasks: RwLock<HashMap<String, TaskRecord>>,
}

impl InMemoryWorkflowRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            workflows: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
            tasks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryWorkflowRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRepository for InMemoryWorkflowRepository {
    // 工作流基本操作
    async fn create(&self, wf: &Workflow) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        workflows.insert(wf.id.clone(), wf.clone());
        Ok(())
    }

    async fn update(&self, wf: &Workflow) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        workflows.insert(wf.id.clone(), wf.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        workflows.remove(id);
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Workflow>, sqlx::Error> {
        let workflows = self.workflows.read();
        Ok(workflows.get(id).cloned())
    }

    async fn list(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Workflow>, i64), sqlx::Error> {
        let workflows = self.workflows.read();
        let mut filtered: Vec<_> = workflows.values().cloned().collect();

        if let Some(s) = status {
            filtered.retain(|w| w.status == s);
        }

        let total = filtered.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, filtered.len());
        filtered.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok((filtered[offset..end].to_vec(), total))
    }

    async fn publish(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        if let Some(wf) = workflows.get_mut(id) {
            wf.status = "published".to_string();
        }
        Ok(())
    }

    async fn save_nodes(
        &self,
        _workflow_id: &str,
        _nodes: &[WorkflowNode],
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn save_edges(
        &self,
        _workflow_id: &str,
        _edges: &[WorkflowEdge],
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn get_nodes(&self, _workflow_id: &str) -> Result<Vec<WorkflowNode>, sqlx::Error> {
        Ok(vec![])
    }

    async fn get_edges(&self, _workflow_id: &str) -> Result<Vec<WorkflowEdge>, sqlx::Error> {
        Ok(vec![])
    }

    // 实例操作
    async fn create_instance(&self, inst: &WorkflowInstance) -> Result<(), sqlx::Error> {
        let mut instances = self.instances.write();
        instances.insert(inst.id.clone(), inst.clone());
        Ok(())
    }

    async fn update_instance(&self, inst: &WorkflowInstance) -> Result<(), sqlx::Error> {
        let mut instances = self.instances.write();
        instances.insert(inst.id.clone(), inst.clone());
        Ok(())
    }

    async fn get_instance(&self, id: &str) -> Result<Option<WorkflowInstance>, sqlx::Error> {
        let instances = self.instances.read();
        Ok(instances.get(id).cloned())
    }

    async fn list_instances(
        &self,
        workflow_id: Option<&str>,
        _status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<WorkflowInstance>, i64), sqlx::Error> {
        let instances = self.instances.read();
        let mut filtered: Vec<_> = instances.values().cloned().collect();

        if let Some(wid) = workflow_id {
            filtered.retain(|i| i.workflow_id == wid);
        }

        let total = filtered.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, filtered.len());
        filtered.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        Ok((filtered[offset..end].to_vec(), total))
    }

    // 任务操作
    async fn create_task(&self, task: &TaskRecord) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn update_task(&self, task: &TaskRecord) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn get_task(&self, id: &str) -> Result<Option<TaskRecord>, sqlx::Error> {
        let tasks = self.tasks.read();
        Ok(tasks.get(id).cloned())
    }

    async fn list_tasks(
        &self,
        instance_id: Option<&str>,
        assignee: Option<&str>,
        _status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<TaskRecord>, i64), sqlx::Error> {
        let tasks = self.tasks.read();
        let mut filtered: Vec<_> = tasks.values().cloned().collect();

        if let Some(iid) = instance_id {
            filtered.retain(|t| t.instance_id == iid);
        }
        if let Some(assign) = assignee {
            let assign_str = assign.to_string();
            filtered.retain(|t| t.assignee.as_ref() == Some(&assign_str));
        }

        let total = filtered.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, filtered.len());
        filtered.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        Ok((filtered[offset..end].to_vec(), total))
    }

    async fn list_tasks_by_assignee(
        &self,
        assignee: &str,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<TaskRecord>, i64), sqlx::Error> {
        self.list_tasks(None, Some(assignee), None, page, page_size)
            .await
    }

    async fn get_task_history(&self, instance_id: &str) -> Result<Vec<TaskRecord>, sqlx::Error> {
        let tasks = self.tasks.read();
        let mut filtered: Vec<_> = tasks.values().cloned().collect();
        filtered.retain(|t| t.instance_id == instance_id);
        filtered.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        Ok(filtered)
    }
}

/// `PostgreSQL` Workflow Repository
pub struct PostgresWorkflowRepository {
    pool: PgPool,
}

impl PostgresWorkflowRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WorkflowRepository for PostgresWorkflowRepository {
    async fn create(&self, wf: &Workflow) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO workflows (id, name, description, definition, status, version, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &wf.id,
            &wf.name,
            wf.description.as_deref(),
            &wf.definition,
            &wf.status,
            wf.version,
            &wf.created_by,
            wf.created_at,
            wf.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, wf: &Workflow) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE workflows SET name = $2, description = $3, definition = $4, status = $5, updated_at = NOW() WHERE id = $1",
            &wf.id,
            &wf.name,
            wf.description.as_deref(),
            &wf.definition,
            &wf.status,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM workflows WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Workflow>, sqlx::Error> {
        let row = sqlx::query_as!(
            Workflow,
            r"SELECT id, name, description, definition, status, version, created_by, created_at, updated_at
             FROM workflows WHERE id = $1",
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Workflow>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflows WHERE ($1::text IS NULL OR status = $1)",
            status,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            Workflow,
            r"SELECT id, name, description, definition, status, version, created_by, created_at, updated_at
             FROM workflows
             WHERE ($1::text IS NULL OR status = $1)
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            status,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let workflows: Vec<Workflow> = rows.into_iter().collect();

        Ok((workflows, count))
    }

    async fn publish(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE workflows SET status = 'published', updated_at = NOW() WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_nodes(
        &self,
        workflow_id: &str,
        nodes: &[WorkflowNode],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!("DELETE FROM workflow_nodes WHERE workflow_id = $1", workflow_id)
            .execute(&mut *tx)
            .await?;

        for node in nodes {
            sqlx::query!(
                r"INSERT INTO workflow_nodes (id, workflow_id, name, node_type, position_x, position_y, config, timeout, auto_complete, created_at)
                  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &node.id,
                workflow_id,
                &node.name,
                &node.node_type,
                node.position_x,
                node.position_y,
                &node.config,
                node.timeout,
                node.auto_complete,
                node.created_at,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }


    async fn save_edges(
        &self,
        workflow_id: &str,
        edges: &[WorkflowEdge],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!("DELETE FROM workflow_edges WHERE workflow_id = $1", workflow_id)
            .execute(&mut *tx)
            .await?;

        for edge in edges {
            sqlx::query!(
                r"INSERT INTO workflow_edges (id, workflow_id, source_node_id, target_node_id, edge_type, condition, label, priority, created_at)
                  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &edge.id,
                workflow_id,
                &edge.source_node_id,
                &edge.target_node_id,
                &edge.edge_type,
                edge.condition.as_deref(),
                edge.label.as_deref(),
                edge.priority,
                edge.created_at,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }


    async fn get_nodes(&self, workflow_id: &str) -> Result<Vec<WorkflowNode>, sqlx::Error> {
        let rows = sqlx::query_as!(
            WorkflowNode,
            r#"SELECT id, workflow_id, name, node_type, position_x, position_y,
                      COALESCE(config, '{}'::jsonb) AS config,
                      timeout,
                      COALESCE(auto_complete, false) AS "auto_complete!",
                      COALESCE(created_at, NOW()) AS "created_at!"
             FROM workflow_nodes WHERE workflow_id = $1 ORDER BY position_x, position_y"#,
            workflow_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn get_edges(&self, workflow_id: &str) -> Result<Vec<WorkflowEdge>, sqlx::Error> {
        let rows = sqlx::query_as!(
            WorkflowEdge,
            r#"SELECT id, workflow_id, source_node_id, target_node_id, edge_type,
                      condition, label,
                      COALESCE(priority, 0) AS "priority!",
                      COALESCE(created_at, NOW()) AS "created_at!"
             FROM workflow_edges WHERE workflow_id = $1 ORDER BY priority"#,
            workflow_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    // 实例操作
    async fn create_instance(&self, inst: &WorkflowInstance) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO workflow_instances (id, workflow_id, workflow_version, status, current_node_id, variables, started_by, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &inst.id,
            &inst.workflow_id,
            inst.workflow_version,
            &inst.status,
            inst.current_node_id.as_deref(),
            &inst.variables,
            &inst.started_by,
            inst.started_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_instance(&self, inst: &WorkflowInstance) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE workflow_instances SET status = $2, current_node_id = $3, variables = $4, completed_at = $5 WHERE id = $1",
            &inst.id,
            &inst.status,
            inst.current_node_id.as_deref(),
            &inst.variables,
            inst.completed_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_instance(&self, id: &str) -> Result<Option<WorkflowInstance>, sqlx::Error> {
        let row = sqlx::query_as!(
            WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances WHERE id = $1",
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_instances(
        &self,
        workflow_id: Option<&str>,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<WorkflowInstance>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflow_instances WHERE ($1::text IS NULL OR workflow_id = $1) AND ($2::text IS NULL OR status = $2)",
            workflow_id,
            status,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances
             WHERE ($1::text IS NULL OR workflow_id = $1) AND ($2::text IS NULL OR status = $2)
             ORDER BY started_at DESC LIMIT $3 OFFSET $4",
            workflow_id,
            status,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let instances: Vec<WorkflowInstance> = rows.into_iter().collect();

        Ok((instances, count))
    }

    // 任务操作
    async fn create_task(&self, task: &TaskRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO task_records (id, instance_id, node_id, node_name, assignee, status, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &task.id,
            &task.instance_id,
            &task.node_id,
            &task.node_name,
            task.assignee.as_deref(),
            &task.status,
            task.started_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_task(&self, task: &TaskRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE task_records SET assignee = $2, status = $3, comment = $4, completed_at = $5, result = $6, form_data = $7 WHERE id = $1",
            &task.id,
            task.assignee.as_deref(),
            &task.status,
            task.comment.as_deref(),
            task.completed_at,
            task.result.as_ref(),
            task.form_data.as_ref(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_task(&self, id: &str) -> Result<Option<TaskRecord>, sqlx::Error> {
        let row = sqlx::query_as!(
            TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!",
                      completed_at, result, form_data, timeout_at
             FROM task_records WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_tasks(
        &self,
        instance_id: Option<&str>,
        assignee: Option<&str>,
        _status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<TaskRecord>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM task_records WHERE ($1::text IS NULL OR instance_id = $1) AND ($2::text IS NULL OR assignee = $2)",
            instance_id,
            assignee,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!",
                      completed_at, result, form_data, timeout_at
             FROM task_records
             WHERE ($1::text IS NULL OR instance_id = $1) AND ($2::text IS NULL OR assignee = $2)
             ORDER BY started_at DESC LIMIT $3 OFFSET $4"#,
            instance_id,
            assignee,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let tasks: Vec<TaskRecord> = rows.into_iter().collect();

        Ok((tasks, count))
    }

    async fn list_tasks_by_assignee(
        &self,
        assignee: &str,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<TaskRecord>, i64), sqlx::Error> {
        self.list_tasks(None, Some(assignee), None, page, page_size)
            .await
    }

    async fn get_task_history(&self, instance_id: &str) -> Result<Vec<TaskRecord>, sqlx::Error> {
        let rows = sqlx::query_as!(
            TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!",
                      completed_at, result, form_data, timeout_at
             FROM task_records WHERE instance_id = $1 ORDER BY started_at DESC"#,
            instance_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

// ============================================================================
// Instance Repository trait
// ============================================================================

pub trait InstanceRepository: Send + Sync {
    fn create(
        &self,
        inst: &WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        inst: &WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<WorkflowInstance>, sqlx::Error>> + Send;
    fn list_by_user(
        &self,
        user_id: &str,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<WorkflowInstance>, i64), sqlx::Error>> + Send;
    fn update_current_node(
        &self,
        id: &str,
        node_id: &str,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn complete(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
}

pub struct PostgresInstanceRepository {
    pool: PgPool,
}

impl PostgresInstanceRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl InstanceRepository for PostgresInstanceRepository {
    async fn create(&self, inst: &WorkflowInstance) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO workflow_instances (id, workflow_id, workflow_version, status, current_node_id, variables, started_by, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &inst.id,
            &inst.workflow_id,
            inst.workflow_version,
            &inst.status,
            inst.current_node_id.as_deref(),
            &inst.variables,
            &inst.started_by,
            inst.started_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, inst: &WorkflowInstance) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE workflow_instances SET status = $2, current_node_id = $3, variables = $4, completed_at = $5 WHERE id = $1",
            &inst.id,
            &inst.status,
            inst.current_node_id.as_deref(),
            &inst.variables,
            inst.completed_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<WorkflowInstance>, sqlx::Error> {
        let row = sqlx::query_as!(
            WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances WHERE id = $1",
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_by_user(
        &self,
        user_id: &str,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<WorkflowInstance>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflow_instances WHERE started_by = $1 AND ($2::text IS NULL OR status = $2)",
            user_id,
            status,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances
             WHERE started_by = $1 AND ($2::text IS NULL OR status = $2)
             ORDER BY started_at DESC LIMIT $3 OFFSET $4",
            user_id,
            status,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let instances: Vec<WorkflowInstance> = rows.into_iter().collect();

        Ok((instances, count))
    }

    async fn update_current_node(&self, id: &str, node_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE workflow_instances SET current_node_id = $2 WHERE id = $1",
            id,
            node_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn complete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE workflow_instances SET status = 'completed', completed_at = NOW() WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ============================================================================
// Task Repository trait
// ============================================================================

pub trait TaskRepository: Send + Sync {
    fn create(
        &self,
        task: &TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        task: &TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<TaskRecord>, sqlx::Error>> + Send;
    fn list_by_instance(
        &self,
        instance_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TaskRecord>, sqlx::Error>> + Send;
    fn list_pending(
        &self,
        assignee: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TaskRecord>, sqlx::Error>> + Send;
    fn complete_task(
        &self,
        id: &str,
        result: Option<&serde_json::Value>,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
}

pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl PostgresTaskRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TaskRepository for PostgresTaskRepository {
    async fn create(&self, task: &TaskRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO task_records (id, instance_id, node_id, node_name, assignee, status, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &task.id,
            &task.instance_id,
            &task.node_id,
            &task.node_name,
            task.assignee.as_deref(),
            &task.status,
            task.started_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, task: &TaskRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE task_records SET assignee = $2, status = $3, comment = $4, completed_at = $5, result = $6, form_data = $7 WHERE id = $1",
            &task.id,
            task.assignee.as_deref(),
            &task.status,
            task.comment.as_deref(),
            task.completed_at,
            task.result.as_ref(),
            task.form_data.as_ref(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<TaskRecord>, sqlx::Error> {
        let row = sqlx::query_as!(
            TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!",
                      completed_at, result, form_data, timeout_at
             FROM task_records WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_by_instance(&self, instance_id: &str) -> Result<Vec<TaskRecord>, sqlx::Error> {
        let rows = sqlx::query_as!(
            TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!",
                      completed_at, result, form_data, timeout_at
             FROM task_records WHERE instance_id = $1 ORDER BY started_at"#,
            instance_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }


    async fn list_pending(&self, assignee: &str) -> Result<Vec<TaskRecord>, sqlx::Error> {
        let rows = sqlx::query_as!(
            TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!",
                      completed_at, result, form_data, timeout_at
             FROM task_records WHERE assignee = $1 AND status = 'pending' ORDER BY started_at"#,
            assignee,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }


    async fn complete_task(&self, id: &str, result: Option<&serde_json::Value>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE task_records SET status = 'completed', completed_at = NOW(), result = $2 WHERE id = $1",
            id,
            result,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

}

// ============================================================================
// Report Repository
// ============================================================================

pub trait ReportRepository: Send + Sync {
    fn create(
        &self,
        report: &Report,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        report: &Report,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<Report>, sqlx::Error>> + Send;
    fn list(
        &self,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<Report>, i64), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
}

pub struct InMemoryReportRepository {
    reports: RwLock<HashMap<String, Report>>,
}

impl InMemoryReportRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            reports: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryReportRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportRepository for InMemoryReportRepository {
    async fn create(&self, report: &Report) -> Result<(), sqlx::Error> {
        let mut reports = self.reports.write();
        reports.insert(report.id.clone(), report.clone());
        Ok(())
    }

    async fn update(&self, report: &Report) -> Result<(), sqlx::Error> {
        let mut reports = self.reports.write();
        reports.insert(report.id.clone(), report.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Report>, sqlx::Error> {
        let reports = self.reports.read();
        Ok(reports.get(id).cloned())
    }

    async fn list(&self, page: i64, page_size: i64) -> Result<(Vec<Report>, i64), sqlx::Error> {
        let reports = self.reports.read();
        let mut all: Vec<_> = reports.values().cloned().collect();
        let total = all.len() as i64;
        all.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, all.len());
        Ok((all[offset..end].to_vec(), total))
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut reports = self.reports.write();
        reports.remove(id);
        Ok(())
    }
}

// ============================================================================
// ScheduledTask Repository
// ============================================================================

pub trait ScheduledTaskRepository: Send + Sync {
    fn create(
        &self,
        task: &ScheduledTask,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        task: &ScheduledTask,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<ScheduledTask>, sqlx::Error>> + Send;
    fn list(
        &self,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<ScheduledTask>, i64), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
}

pub struct InMemoryScheduledTaskRepository {
    tasks: RwLock<HashMap<String, ScheduledTask>>,
}

impl InMemoryScheduledTaskRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryScheduledTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduledTaskRepository for InMemoryScheduledTaskRepository {
    async fn create(&self, task: &ScheduledTask) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn update(&self, task: &ScheduledTask) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ScheduledTask>, sqlx::Error> {
        let tasks = self.tasks.read();
        Ok(tasks.get(id).cloned())
    }

    async fn list(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ScheduledTask>, i64), sqlx::Error> {
        let tasks = self.tasks.read();
        let mut all: Vec<_> = tasks.values().cloned().collect();
        let total = all.len() as i64;
        all.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, all.len());
        Ok((all[offset..end].to_vec(), total))
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.remove(id);
        Ok(())
    }
}
