use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;

#[derive(Clone)]
pub struct PublishService {
    db: PgPool,
}

impl PublishService {
    #[must_use]
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn publish(
        &self,
        content_id: Uuid,
        account_id: Uuid,
        version_id: Option<Uuid>,
    ) -> Result<Value> {
        let content_exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.content_items WHERE id = $1" ).bind(content_id)
        .fetch_one(&self.db)
        .await?
        .unwrap_or(0);

        if content_exists == 0 {
            anyhow::bail!("内容不存在: {content_id}" );
        }

        let account_exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.social_accounts WHERE id = $1 AND is_active = true" ).bind(account_id)
        .fetch_one(&self.db)
        .await?
        .unwrap_or(0);

        if account_exists == 0 {
            anyhow::bail!("社交账号不存在或未激活: {account_id}" );
        }

        if let Some(vid) = version_id {
            let version_exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.rewrite_versions WHERE id = $1" ).bind(vid)
            .fetch_one(&self.db)
            .await?
            .unwrap_or(0);

            if version_exists == 0 {
                anyhow::bail!("改写版本不存在: {vid}" );
            }
        }

        let row = sqlx::query(r#"INSERT INTO socialops.publish_tasks (content_id, target_account, status, publish_mode)
             VALUES ($1, $2, 'pending', 'manual')
             RETURNING id, content_id, target_account AS "account_id" , status"#).bind(content_id).bind(account_id)
        .fetch_one(&self.db)
        .await?;

        Ok(serde_json::json!({
            "id": row.id,
            "content_id": row.content_id,
            "account_id": row.account_id,
            "version_id": version_id,
            "status": row.status,
        }))
    }

    pub async fn process_task(&self, task_id: Uuid) -> Result<Value> {
        let task = sqlx::query(r#"SELECT id, content_id, target_account, status
             FROM socialops.publish_tasks WHERE id = $1"#).bind(task_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("发布任务不存在: {task_id}" ))?;

        if task.status != "pending" {
            anyhow::bail!("任务状态不是 pending，当前状态: {}" , task.status);
        }

        sqlx::query("UPDATE socialops.publish_tasks SET status = 'running' WHERE id = $1" ).bind(task_id)
        .execute(&self.db)
        .await?;

        // 占位实现：模拟发布，更新为 completed
        // NOTE: 实际生产环境需根据 account_id 获取平台凭证，调用对应平台 API 发布
        // 当前返回模拟成功响应，避免编译警告
        sqlx::query(r#"INSERT INTO socialops.publish_results (task_id, response_body)
             VALUES ($1, $2::jsonb)"#).bind(task_id).bind(&serde_json::json!({
                "message": "模拟发布成功（占位实现）" ,
                "platform": "placeholder" ,
                "external_id": null,
            }))
        .execute(&self.db)
        .await?;

        sqlx::query("UPDATE socialops.publish_tasks SET status = 'completed', published_at = NOW() WHERE id = $1" ).bind(task_id)
        .execute(&self.db)
        .await?;

        Ok(serde_json::json!({
            "task_id": task_id,
            "status": "completed" ,
        }))
    }

    pub async fn list_tasks(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Value>, i64)> {
        let offset = (page - 1) * page_size;

        let rows = sqlx::query(r#"SELECT t.id, t.content_id, t.target_account AS "account_id" , t.status,
               to_char(t.created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at" ,
               to_char(t.published_at, 'YYYY-MM-DD HH24:MI:SS') AS "completed_at"
             FROM socialops.publish_tasks t
             WHERE ($1::text IS NULL OR t.status = $1)
             ORDER BY t.created_at DESC
             LIMIT $2 OFFSET $3"#).bind(status).bind(page_size).bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.publish_tasks WHERE ($1::text IS NULL OR status = $1)" ).bind(status)
        .fetch_one(&self.db)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "content_id": row.content_id,
                    "account_id": row.account_id,
                    "status": row.status,
                    "created_at": row.created_at,
                    "completed_at": row.completed_at,
                })
            })
            .collect();

        Ok((items, total))
    }

    pub async fn get_task(&self, task_id: Uuid) -> Result<Option<Value>> {
        let task = sqlx::query(r#"SELECT t.id, t.content_id, t.target_account AS "account_id" , t.status,
               to_char(t.created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at" ,
               to_char(t.published_at, 'YYYY-MM-DD HH24:MI:SS') AS "completed_at"
             FROM socialops.publish_tasks t
             WHERE t.id = $1"#).bind(task_id)
        .fetch_optional(&self.db)
        .await?;

        match task {
            Some(row) => {
                let results = self.get_results(row.id).await?;
                Ok(Some(serde_json::json!({
                    "id": row.id,
                    "content_id": row.content_id,
                    "account_id": row.account_id,
                    "status": row.status,
                    "created_at": row.created_at,
                    "completed_at": row.completed_at,
                    "results": results,
                })))
            }
            None => Ok(None),
        }
    }

    async fn get_results(&self, task_id: Uuid) -> Result<Vec<Value>> {
        let rows = sqlx::query(r#"SELECT id, response_body, to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at"
             FROM socialops.publish_results
             WHERE task_id = $1
             ORDER BY created_at"#).bind(task_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "platform_response": row.response_body,
                    "created_at": row.created_at,
                })
            })
            .collect())
    }

    pub async fn create_schedule(
        &self,
        content_id: Uuid,
        account_id: Uuid,
        cron_expr: &str,
    ) -> Result<Value> {
        let content_exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.content_items WHERE id = $1" ).bind(content_id)
        .fetch_one(&self.db)
        .await?
        .unwrap_or(0);

        if content_exists == 0 {
            anyhow::bail!("内容不存在: {content_id}" );
        }

        let account_exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.social_accounts WHERE id = $1 AND is_active = true" ).bind(account_id)
        .fetch_one(&self.db)
        .await?
        .unwrap_or(0);

        if account_exists == 0 {
            anyhow::bail!("社交账号不存在或未激活: {account_id}" );
        }

        let row = sqlx::query(r#"INSERT INTO socialops.publish_schedules (content_id, target_account, cron_expression)
             VALUES ($1, $2, $3)
             RETURNING id, content_id, target_account AS "account_id" , cron_expression AS "cron_expr" ,
               is_recurring, status,
               to_char(scheduled_at, 'YYYY-MM-DD HH24:MI:SS') AS "scheduled_at" ,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at" "#).bind(content_id).bind(account_id).bind(cron_expr)
        .fetch_one(&self.db)
        .await?;

        Ok(serde_json::json!({
            "id": row.id,
            "content_id": row.content_id,
            "account_id": row.account_id,
            "version_id": Value::Null,
            "cron_expr": row.cron_expr,
            "is_recurring": row.is_recurring,
            "is_active": row.is_recurring,
            "status": row.status,
            "scheduled_at": row.scheduled_at,
            "created_at": row.created_at,
        }))
    }

    pub async fn list_schedules(&self) -> Result<Vec<Value>> {
        let rows = sqlx::query(r#"SELECT id, content_id, target_account AS "account_id" , rewrite_version_id AS "version_id" ,
               cron_expression AS "cron_expr" , is_recurring, status,
               to_char(scheduled_at, 'YYYY-MM-DD HH24:MI:SS') AS "scheduled_at" ,
               to_char(next_run_at, 'YYYY-MM-DD HH24:MI:SS') AS "next_run_at" ,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at"
             FROM socialops.publish_schedules
             ORDER BY created_at DESC"#)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "content_id": row.content_id,
                    "account_id": row.account_id,
                    "version_id": row.version_id,
                    "cron_expr": row.cron_expr,
                    "is_recurring": row.is_recurring,
                    "is_active": row.is_recurring,
                    "status": row.status,
                    "scheduled_at": row.scheduled_at,
                    "next_run_at": row.next_run_at,
                    "created_at": row.created_at,
                })
            })
            .collect())
    }

    pub async fn update_schedule(
        &self,
        id: Uuid,
        cron_expr: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<bool> {
        let r = sqlx::query(r#"UPDATE socialops.publish_schedules
             SET cron_expression = COALESCE($2, cron_expression),
                 is_recurring = COALESCE($3, is_recurring)
             WHERE id = $1"#).bind(id).bind(cron_expr).bind(is_active)
        .execute(&self.db)
        .await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn get_schedule(&self, id: Uuid) -> Result<Option<Value>> {
        let row = sqlx::query(r#"SELECT s.id, s.content_id, s.target_account AS "account_id" , s.rewrite_version_id AS "version_id" ,
               s.cron_expression AS "cron_expr" , s.is_recurring, s.status,
               to_char(s.scheduled_at, 'YYYY-MM-DD HH24:MI:SS') AS "scheduled_at" ,
               to_char(s.next_run_at, 'YYYY-MM-DD HH24:MI:SS') AS "next_run_at" ,
               to_char(s.created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at"
             FROM socialops.publish_schedules s
             WHERE s.id = $1"#).bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| {
            serde_json::json!({
                "id": row.id,
                "content_id": row.content_id,
                "account_id": row.account_id,
                "version_id": row.version_id,
                "cron_expr": row.cron_expr,
                "is_recurring": row.is_recurring,
                "is_active": row.is_recurring,
                "status": row.status,
                "scheduled_at": row.scheduled_at,
                "next_run_at": row.next_run_at,
                "created_at": row.created_at,
            })
        }))
    }

    pub async fn delete_schedule(&self, id: Uuid) -> Result<bool> {
        let r = sqlx::query("DELETE FROM socialops.publish_schedules WHERE id = $1" ).bind(id)
            .execute(&self.db)
            .await?;
        Ok(r.rows_affected() > 0)
    }
}
