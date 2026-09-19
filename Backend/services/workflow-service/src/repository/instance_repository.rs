//! Workflow 数据仓储层 - 工作流实例
//!
//! 工作流实例的 CRUD 操作

use sqlx::PgPool;

// ============================================================================
// Instance Repository trait
// ============================================================================

pub trait InstanceRepository: Send + Sync {
    fn create(
        &self,
        inst: &super::WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        inst: &super::WorkflowInstance,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::WorkflowInstance>, sqlx::Error>> + Send;
    fn list_by_user(
        &self,
        user_id: &str,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::WorkflowInstance>, i64), sqlx::Error>> + Send;
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
    async fn create(&self, inst: &super::WorkflowInstance) -> Result<(), sqlx::Error> {
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

    async fn update(&self, inst: &super::WorkflowInstance) -> Result<(), sqlx::Error> {
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

    async fn find_by_id(&self, id: &str) -> Result<Option<super::WorkflowInstance>, sqlx::Error> {
        let row = sqlx::query_as!(
            super::WorkflowInstance,
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
                      COALESCE(variables, '{}'::jsonb) AS variables,
                      started_by, started_at, completed_at
             FROM workflow_instances WHERE id = $1" ,
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
    ) -> Result<(Vec<super::WorkflowInstance>, i64), sqlx::Error> {
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
            super::WorkflowInstance,
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

        let instances: Vec<super::WorkflowInstance> = rows.into_iter().collect();

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
