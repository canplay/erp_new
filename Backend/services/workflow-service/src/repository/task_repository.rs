//! Workflow 数据仓储层 - 任务
//!
//! 任务的 CRUD 操作

use sqlx::PgPool;

// ============================================================================
// Task Repository trait
// ============================================================================

pub trait TaskRepository: Send + Sync {
    fn create(
        &self,
        task: &super::TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        task: &super::TaskRecord,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::TaskRecord>, sqlx::Error>> + Send;
    fn list_by_instance(
        &self,
        instance_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<super::TaskRecord>, sqlx::Error>> + Send;
    fn list_pending(
        &self,
        assignee: &str,
    ) -> impl std::future::Future<Output = Result<Vec<super::TaskRecord>, sqlx::Error>> + Send;
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
    async fn create(&self, task: &super::TaskRecord) -> Result<(), sqlx::Error> {
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

    async fn update(&self, task: &super::TaskRecord) -> Result<(), sqlx::Error> {
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

    async fn find_by_id(&self, id: &str) -> Result<Option<super::TaskRecord>, sqlx::Error> {
        let row = sqlx::query_as!(
            super::TaskRecord,
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

    async fn list_by_instance(&self, instance_id: &str) -> Result<Vec<super::TaskRecord>, sqlx::Error> {
        let rows = sqlx::query_as!(
            super::TaskRecord,
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

    async fn list_pending(&self, assignee: &str) -> Result<Vec<super::TaskRecord>, sqlx::Error> {
        let rows = sqlx::query_as!(
            super::TaskRecord,
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
