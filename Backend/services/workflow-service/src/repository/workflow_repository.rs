//! Workflow 数据仓储层 - 工作流定义
//!
//! 工作流、节点、连线的 CRUD 操作

use parking_lot::RwLock;
use sqlx::PgPool;
use std::collections::HashMap;

// ============================================================================
// Workflow Repository trait
// ============================================================================

/// Workflow Repository trait
pub trait WorkflowRepository: Send + Sync {
    // 工作流基本操作
    fn create(
        &self,
        wf: &super::Workflow,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        wf: &super::Workflow,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::Workflow>, sqlx::Error>> + Send;
    fn list(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::Workflow>, i64), sqlx::Error>> + Send;
    fn publish(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn save_nodes(
        &self,
        workflow_id: &str,
        nodes: &[super::WorkflowNode],
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn save_edges(
        &self,
        workflow_id: &str,
        edges: &[super::WorkflowEdge],
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn get_nodes(
        &self,
        workflow_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<super::WorkflowNode>, sqlx::Error>> + Send;
    fn get_edges(
        &self,
        workflow_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<super::WorkflowEdge>, sqlx::Error>> + Send;

    // 实例操作（grpc_handlers 需要）
    fn create_instance(
        &self,
        inst: &super::WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update_instance(
        &self,
        inst: &super::WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn get_instance(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::WorkflowInstance>, sqlx::Error>> + Send;
    fn list_instances(
        &self,
        workflow_id: Option<&str>,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::WorkflowInstance>, i64), sqlx::Error>> + Send;

    // 任务操作（grpc_handlers 需要）
    fn create_task(
        &self,
        task: &super::TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update_task(
        &self,
        task: &super::TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn get_task(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::TaskRecord>, sqlx::Error>> + Send;
    fn list_tasks(
        &self,
        instance_id: Option<&str>,
        assignee: Option<&str>,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::TaskRecord>, i64), sqlx::Error>> + Send;
    fn list_tasks_by_assignee(
        &self,
        assignee: &str,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::TaskRecord>, i64), sqlx::Error>> + Send;
    fn get_task_history(
        &self,
        instance_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<super::TaskRecord>, sqlx::Error>> + Send;
}

/// `InMemory` Workflow Repository
pub struct InMemoryWorkflowRepository {
    workflows: RwLock<HashMap<String, super::Workflow>>,
    instances: RwLock<HashMap<String, super::WorkflowInstance>>,
    tasks: RwLock<HashMap<String, super::TaskRecord>>,
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
    async fn create(&self, wf: &super::Workflow) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        workflows.insert(wf.id.clone(), wf.clone());
        Ok(())
    }

    async fn update(&self, wf: &super::Workflow) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        workflows.insert(wf.id.clone(), wf.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut workflows = self.workflows.write();
        workflows.remove(id);
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<super::Workflow>, sqlx::Error> {
        let workflows = self.workflows.read();
        Ok(workflows.get(id).cloned())
    }

    async fn list(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<super::Workflow>, i64), sqlx::Error> {
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
        _nodes: &[super::WorkflowNode],
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn save_edges(
        &self,
        _workflow_id: &str,
        _edges: &[super::WorkflowEdge],
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn get_nodes(&self, _workflow_id: &str) -> Result<Vec<super::WorkflowNode>, sqlx::Error> {
        Ok(vec![])
    }

    async fn get_edges(&self, _workflow_id: &str) -> Result<Vec<super::WorkflowEdge>, sqlx::Error> {
        Ok(vec![])
    }

    async fn create_instance(&self, inst: &super::WorkflowInstance) -> Result<(), sqlx::Error> {
        let mut instances = self.instances.write();
        instances.insert(inst.id.clone(), inst.clone());
        Ok(())
    }

    async fn update_instance(&self, inst: &super::WorkflowInstance) -> Result<(), sqlx::Error> {
        let mut instances = self.instances.write();
        instances.insert(inst.id.clone(), inst.clone());
        Ok(())
    }

    async fn get_instance(&self, id: &str) -> Result<Option<super::WorkflowInstance>, sqlx::Error> {
        let instances = self.instances.read();
        Ok(instances.get(id).cloned())
    }

    async fn list_instances(
        &self,
        workflow_id: Option<&str>,
        _status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<super::WorkflowInstance>, i64), sqlx::Error> {
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

    async fn create_task(&self, task: &super::TaskRecord) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn update_task(&self, task: &super::TaskRecord) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn get_task(&self, id: &str) -> Result<Option<super::TaskRecord>, sqlx::Error> {
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
    ) -> Result<(Vec<super::TaskRecord>, i64), sqlx::Error> {
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
    ) -> Result<(Vec<super::TaskRecord>, i64), sqlx::Error> {
        self.list_tasks(None, Some(assignee), None, page, page_size).await
    }

    async fn get_task_history(&self, instance_id: &str) -> Result<Vec<super::TaskRecord>, sqlx::Error> {
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
    async fn create(&self, wf: &super::Workflow) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO workflows (id, name, description, definition, status, version, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            wf.id,
            wf.name,
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

    async fn update(&self, wf: &super::Workflow) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE workflows SET name = $2, description = $3, definition = $4, status = $5, updated_at = NOW() WHERE id = $1"#,
            wf.id,
            wf.name,
            wf.description.as_deref(),
            &wf.definition,
            &wf.status,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM workflows WHERE id = $1"#, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<super::Workflow>, sqlx::Error> {
        let row = sqlx::query_as!(
            super::Workflow,
            r#"SELECT id, name, description, definition, status, version, created_by, created_at, updated_at
             FROM workflows WHERE id = $1"#,
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
    ) -> Result<(Vec<super::Workflow>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM workflows WHERE ($1::text IS NULL OR status = $1)"#,
            status,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            super::Workflow,
            r#"SELECT id, name, description, definition, status, version, created_by, created_at, updated_at
             FROM workflows
             WHERE ($1::text IS NULL OR status = $1)
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
            status,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let workflows: Vec<super::Workflow> = rows.into_iter().collect();

        Ok((workflows, count))
    }

    async fn publish(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"UPDATE workflows SET status = 'published', updated_at = NOW() WHERE id = $1"#, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_nodes(
        &self,
        workflow_id: &str,
        nodes: &[super::WorkflowNode],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(r#"DELETE FROM workflow_nodes WHERE workflow_id = $1"#, workflow_id)
            .execute(&mut *tx)
            .await?;

        for node in nodes {
            sqlx::query!(
                r#"INSERT INTO workflow_nodes (id, workflow_id, name, node_type, position_x, position_y, config, timeout, auto_complete, created_at)
                  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
                node.id,
                workflow_id,
                node.name,
                node.node_type,
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
        edges: &[super::WorkflowEdge],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(r#"DELETE FROM workflow_edges WHERE workflow_id = $1"#, workflow_id)
            .execute(&mut *tx)
            .await?;

        for edge in edges {
            sqlx::query!(
                r#"INSERT INTO workflow_edges (id, workflow_id, source_node_id, target_node_id, edge_type, condition, label, priority, created_at)
                  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
                edge.id,
                workflow_id,
                edge.source_node_id,
                edge.target_node_id,
                edge.edge_type,
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

    async fn get_nodes(&self, workflow_id: &str) -> Result<Vec<super::WorkflowNode>, sqlx::Error> {
        let rows = sqlx::query_as!(
            super::WorkflowNode,
            r#"SELECT id, workflow_id, name, node_type, position_x, position_y,
                      COALESCE(config, '{}'::jsonb) AS config,
                      timeout,
                      COALESCE(auto_complete, false) AS "auto_complete!" ,
                      COALESCE(created_at, NOW()) AS "created_at!"
             FROM workflow_nodes WHERE workflow_id = $1 ORDER BY position_x, position_y"#,
            workflow_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn get_edges(&self, workflow_id: &str) -> Result<Vec<super::WorkflowEdge>, sqlx::Error> {
        let rows = sqlx::query_as!(
            super::WorkflowEdge,
            r#"SELECT id, workflow_id, source_node_id, target_node_id, edge_type,
                      condition, label,
                      COALESCE(priority, 0) AS "priority!" ,
                      COALESCE(created_at, NOW()) AS "created_at!"
             FROM workflow_edges WHERE workflow_id = $1 ORDER BY priority"#,
            workflow_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn create_instance(&self, inst: &super::WorkflowInstance) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO workflow_instances (id, workflow_id, workflow_version, status, current_node_id, variables, started_by, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            inst.id,
            inst.workflow_id,
            inst.workflow_version,
            inst.status,
            inst.current_node_id.as_deref(),
            &inst.variables,
            inst.started_by,
            inst.started_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_instance(&self, inst: &super::WorkflowInstance) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE workflow_instances SET status = $2, current_node_id = $3, variables = $4, completed_at = $5 WHERE id = $1"#,
            inst.id,
            inst.status,
            inst.current_node_id.as_deref(),
            &inst.variables,
            inst.completed_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_instance(&self, id: &str) -> Result<Option<super::WorkflowInstance>, sqlx::Error> {
        let row = sqlx::query_as!(
            super::WorkflowInstance,
            r#"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances WHERE id = $1"#,
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
    ) -> Result<(Vec<super::WorkflowInstance>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM workflow_instances WHERE ($1::text IS NULL OR workflow_id = $1) AND ($2::text IS NULL OR status = $2)"#,
            workflow_id,
            status,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            super::WorkflowInstance,
            r#"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances
             WHERE ($1::text IS NULL OR workflow_id = $1) AND ($2::text IS NULL OR status = $2)
             ORDER BY started_at DESC LIMIT $3 OFFSET $4"#,
            workflow_id,
            status,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let instances: Vec<super::WorkflowInstance> = rows.into_iter().collect();

        Ok((instances, count))
    }

    async fn create_task(&self, task: &super::TaskRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO task_records (id, instance_id, node_id, node_name, assignee, status, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            task.id,
            task.instance_id,
            task.node_id,
            task.node_name,
            task.assignee.as_deref(),
            &task.status,
            task.started_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_task(&self, task: &super::TaskRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE task_records SET assignee = $2, status = $3, comment = $4, completed_at = $5, result = $6, form_data = $7 WHERE id = $1"#,
            task.id,
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

    async fn get_task(&self, id: &str) -> Result<Option<super::TaskRecord>, sqlx::Error> {
        let row = sqlx::query_as!(
            super::TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!" ,
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
    ) -> Result<(Vec<super::TaskRecord>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM task_records WHERE ($1::text IS NULL OR instance_id = $1) AND ($2::text IS NULL OR assignee = $2)"#,
            instance_id,
            assignee,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            super::TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!" ,
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

        let tasks: Vec<super::TaskRecord> = rows.into_iter().collect();

        Ok((tasks, count))
    }

    async fn list_tasks_by_assignee(
        &self,
        assignee: &str,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<super::TaskRecord>, i64), sqlx::Error> {
        self.list_tasks(None, Some(assignee), None, page, page_size).await
    }

    async fn get_task_history(&self, instance_id: &str) -> Result<Vec<super::TaskRecord>, sqlx::Error> {
        let rows = sqlx::query_as!(
            super::TaskRecord,
            r#"SELECT id, instance_id, node_id, node_name, assignee, status, comment,
                      COALESCE(started_at, NOW()) AS "started_at!" ,
                      completed_at, result, form_data, timeout_at
             FROM task_records WHERE instance_id = $1 ORDER BY started_at DESC"#,
            instance_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
