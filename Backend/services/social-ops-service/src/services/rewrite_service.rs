//! Rewrite service — LLM-based content rewriting
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;
use sqlx::FromRow;

#[derive(Clone)]
pub struct RewriteService {
    db: PgPool,
}

#[derive(FromRow)]
struct RewriteTaskRow {
    id: Uuid,
    content_id: Uuid,
    llm_provider_id: Uuid,
    rewrite_prompt: Option<String>,
    status: String,
    target_count: i32,
    created_at: String,
    completed_at: Option<String>,
}

#[derive(FromRow)]
struct RewriteTaskCreateRow {
    id: Uuid,
    content_id: Uuid,
    llm_provider_id: Uuid,
    rewrite_prompt: String,
    status: String,
    target_count: i32,
}

#[derive(FromRow)]
struct ContentRow {
    title: Option<String>,
    body: Option<String>,
}

#[derive(FromRow)]
struct LlmProviderRow {
    api_endpoint: String,
    #[allow(dead_code)] // 保留列映射供未来 LLM 调用实现使用
    api_key_enc: String,
    model_name: Option<String>,
    #[allow(dead_code)] // 保留列映射供未来 LLM 调用实现使用
    default_params: Option<String>,
}

#[derive(FromRow)]
struct RewriteVersionRow {
    id: Uuid,
    version_seq: i32,
    rewritten_title: String,
    rewritten_body: String,
}

#[derive(FromRow)]
struct RewriteVersionDetailRow {
    id: Uuid,
    version_seq: i32,
    rewritten_title: Option<String>,
    rewritten_body: Option<String>,
    status: String,
    created_at: String,
}

impl RewriteService {
    #[must_use]
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_task(
        &self,
        content_id: Uuid,
        llm_provider_id: Uuid,
        target_count: i32,
    ) -> Result<Value> {
        let content_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM socialops.content_items WHERE id = $1" ).bind(content_id)
        .fetch_one(&self.db)
        .await?;

        if content_exists == 0 {
            anyhow::bail!("内容不存在: {content_id}" );
        }

        let provider_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM socialops.llm_providers WHERE id = $1 AND is_active = true" ).bind(llm_provider_id)
        .fetch_one(&self.db)
        .await?;

        if provider_exists == 0 {
            anyhow::bail!("LLM 提供者不存在或未激活: {llm_provider_id}" );
        }

        let default_prompt = "请对以下内容进行改写，生成多个不同风格的版本。返回 JSON 数组，每个元素包含 title 和 body 字段。";

        let row = sqlx::query_as::<_, RewriteTaskCreateRow>(r#"INSERT INTO socialops.rewrite_tasks (content_id, llm_provider_id, rewrite_prompt, target_count, status)
             VALUES ($1, $2, $3, $4, 'pending')
             RETURNING id, content_id, llm_provider_id, rewrite_prompt AS "rewrite_prompt!" , status, target_count"#).bind(content_id).bind(llm_provider_id).bind(default_prompt).bind(target_count)
        .fetch_one(&self.db)
        .await?;

        Ok(serde_json::json!({
            "id": row.id,
            "content_id": row.content_id,
            "llm_provider_id": row.llm_provider_id,
            "rewrite_prompt": row.rewrite_prompt,
            "status": row.status,
            "target_count": row.target_count,
        }))
    }

    pub async fn process_task(&self, task_id: Uuid) -> Result<Value> {
        let task = sqlx::query_as::<_, RewriteTaskRow>(r#"SELECT id, content_id, llm_provider_id, rewrite_prompt, status, target_count,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at" ,
               to_char(completed_at, 'YYYY-MM-DD HH24:MI:SS') AS "completed_at"
             FROM socialops.rewrite_tasks WHERE id = $1"#).bind(task_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("任务不存在: {task_id}" ))?;

        if task.status != "pending" {
            anyhow::bail!("任务状态不是 pending，当前状态: {}" , task.status);
        }

        let (content_id, llm_provider_id, rewrite_prompt, target_count) =
            (task.content_id, task.llm_provider_id, task.rewrite_prompt, task.target_count);
        let prompt = rewrite_prompt.as_deref().unwrap_or(
            "请对以下内容进行改写，生成多个不同风格的版本。返回 JSON 数组，每个元素包含 title 和 body 字段。" ,
        );

        let content = sqlx::query_as::<_, ContentRow>(r#"SELECT title, body FROM socialops.content_items WHERE id = $1"#).bind(content_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("内容不存在: {content_id}" ))?;

        let original_text = format!(
            "标题：{}\n\n正文：{}" ,
            content.title.as_deref().unwrap_or("" ),
            content.body.as_deref().unwrap_or("" )
        );

        let provider = sqlx::query_as::<_, LlmProviderRow>(r#"SELECT api_endpoint, api_key_enc, model_name, default_params::text AS "default_params"
             FROM socialops.llm_providers WHERE id = $1 AND is_active = true"#).bind(llm_provider_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("LLM 提供者不存在或未激活: {llm_provider_id}" ))?;

        let (api_endpoint, api_key, model_name) =
            (provider.api_endpoint, provider.api_key_enc, provider.model_name);

        sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'running' WHERE id = $1" ).bind(task_id)
        .execute(&self.db)
        .await?;

        let endpoint = format!("{}/chat/completions" , api_endpoint.trim_end_matches('/'));
        let model = model_name.unwrap_or_else(|| "gpt-4o-mini".to_string());

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system" ,
                    "content": format!(
                        "你是一个专业的内容改写助手。{} 请严格按照 JSON 数组格式返回，不要包含任何其他文字。" ,
                        prompt
                    )
                },
                {
                    "role": "user" ,
                    "content": format!(
                        "请对以下内容进行改写，生成 {} 个不同风格的版本（如正式、轻松、营销、学术等风格）。\n\n原文：\n{}" ,
                        target_count, original_text
                    )
                }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.7
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&endpoint)
            .header("Authorization" , format!("Bearer {api_key}" ))
            .header("Content-Type" , "application/json" )
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("LLM API 请求失败: {e}" ))?;

        if !resp.status().is_success() {
            let status_code = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'failed' WHERE id = $1" ).bind(task_id)
            .execute(&self.db)
            .await?;
            anyhow::bail!("LLM API 返回错误: status={status_code}, body={error_body}" );
        }

        let llm_response: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("解析 LLM API 响应失败: {e}" ))?;

        let llm_content = llm_response["choices" ][0]["message" ]["content" ]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("LLM 响应结构异常: 缺少 choices[0].message.content" ))?;

        let usage = llm_response.get("usage" );

        let versions: Vec<Value> = serde_json::from_str(llm_content)
            .map_err(|e| anyhow::anyhow!("解析 LLM 返回的 JSON 失败: {e}" ))?;

        if versions.is_empty() {
            sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'failed' WHERE id = $1" ).bind(task_id)
            .execute(&self.db)
            .await?;
            anyhow::bail!("LLM 未返回有效的版本数据" );
        }

        let mut created_versions = Vec::new();
        for (i, version) in versions.iter().enumerate() {
            let seq = (i + 1) as i32;
            let v_title = version["title" ].as_str().unwrap_or("" );
            let v_body = version["body" ].as_str().unwrap_or("" );

            let row = sqlx::query_as::<_, RewriteVersionRow>(r#"INSERT INTO socialops.rewrite_versions (task_id, version_seq, rewritten_title, rewritten_body, llm_raw_response, status)
                 VALUES ($1, $2, $3, $4, $5::jsonb, 'draft')
                 RETURNING id, version_seq, rewritten_title AS "rewritten_title!" , rewritten_body AS "rewritten_body!\" "#).bind(task_id).bind(seq).bind(v_title).bind(v_body).bind(&llm_response)
            .fetch_one(&self.db)
            .await?;

            created_versions.push(serde_json::json!({
                "id": row.id,
                "version_seq": row.version_seq,
                "rewritten_title": row.rewritten_title,
                "rewritten_body": row.rewritten_body,
            }));
        }

        sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'completed' WHERE id = $1" ).bind(task_id)
        .execute(&self.db)
        .await?;

        Ok(serde_json::json!({
            "task_id": task_id,
            "status": "completed" ,
            "versions_count": created_versions.len(),
            "versions": created_versions,
            "usage": usage,
        }))
    }

    pub async fn list_tasks(
        &self,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Value>, i64)> {
        let offset = (page - 1) * page_size;

        #[derive(FromRow)]
        struct TaskListItemRow {
            id: Uuid,
            content_id: Uuid,
            status: String,
            rewrite_prompt: Option<String>,
            target_count: i32,
            created_at: String,
        }

        let rows = sqlx::query_as::<_, TaskListItemRow>(r#"SELECT t.id, t.content_id, t.status, t.rewrite_prompt, t.target_count,
               to_char(t.created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at"
             FROM socialops.rewrite_tasks t
             WHERE ($1::text IS NULL OR t.status = $1)
             ORDER BY t.created_at DESC
             LIMIT $2 OFFSET $3"#).bind(status).bind(page_size).bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM socialops.rewrite_tasks WHERE ($1::text IS NULL OR status = $1)" ).bind(status)
        .fetch_one(&self.db)
        .await?;

        let items = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "content_id": row.content_id,
                    "status": row.status,
                    "rewrite_prompt": row.rewrite_prompt,
                    "target_count": row.target_count,
                    "created_at": row.created_at,
                })
            })
            .collect();

        Ok((items, total))
    }

    pub async fn get_task(&self, task_id: Uuid) -> Result<Option<Value>> {
        let task = sqlx::query_as::<_, RewriteTaskRow>(r#"SELECT t.id, t.content_id, t.llm_provider_id, t.rewrite_prompt, t.status, t.target_count,
               to_char(t.created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at" ,
               to_char(t.completed_at, 'YYYY-MM-DD HH24:MI:SS') AS "completed_at"
             FROM socialops.rewrite_tasks t
             WHERE t.id = $1"#).bind(task_id)
        .fetch_optional(&self.db)
        .await?;

        match task {
            Some(row) => {
                let versions = self.get_versions_raw(row.id).await?;
                Ok(Some(serde_json::json!({
                    "id": row.id,
                    "content_id": row.content_id,
                    "llm_provider_id": row.llm_provider_id,
                    "rewrite_prompt": row.rewrite_prompt,
                    "status": row.status,
                    "target_count": row.target_count,
                    "created_at": row.created_at,
                    "completed_at": row.completed_at,
                    "versions": versions,
                })))
            }
            None => Ok(None),
        }
    }

    pub async fn get_versions(&self, task_id: Uuid) -> Result<Vec<Value>> {
        self.get_versions_raw(task_id).await
    }

    async fn get_versions_raw(&self, task_id: Uuid) -> Result<Vec<Value>> {
        let rows = sqlx::query_as::<_, RewriteVersionDetailRow>(r#"SELECT id, version_seq, rewritten_title, rewritten_body, status,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at"
             FROM socialops.rewrite_versions
             WHERE task_id = $1
             ORDER BY version_seq"#).bind(task_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "version_seq": row.version_seq,
                    "rewritten_title": row.rewritten_title,
                    "rewritten_body": row.rewritten_body,
                    "status": row.status,
                    "created_at": row.created_at,
                })
            })
            .collect())
    }

    pub async fn update_version_status(&self, version_id: Uuid, status: &str) -> Result<bool> {
        let r = sqlx::query("UPDATE socialops.rewrite_versions SET status = $1 WHERE id = $2" ).bind(status).bind(version_id)
        .execute(&self.db).await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn trigger_process(&self, task_id: Uuid) -> Result<Value> {
        let task = sqlx::query_as::<_, RewriteTaskRow>(r#"SELECT id, content_id, llm_provider_id, rewrite_prompt, status, target_count,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_at" ,
               to_char(completed_at, 'YYYY-MM-DD HH24:MI:SS') AS "completed_at"
             FROM socialops.rewrite_tasks WHERE id = $1"#).bind(task_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("task not found: {task_id}" ))?;

        if task.status != "pending" {
            anyhow::bail!("task status is not pending: {}" , task.status);
        }

        let (content_id, llm_provider_id, prompt, target_count) =
            (task.content_id, task.llm_provider_id, task.rewrite_prompt, task.target_count);
        let prompt = prompt.as_deref().unwrap_or(
            "请对以下内容进行改写，生成多个不同风格的版本。返回 JSON 数组，每个元素包含 title 和 body 字段。" ,
        );

        let content = sqlx::query_as::<_, ContentRow>(r#"SELECT title, body FROM socialops.content_items WHERE id = $1"#).bind(content_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("content not found: {content_id}" ))?;

        let original_text = format!(
            "标题：{}\n\n正文：{}" ,
            content.title.as_deref().unwrap_or("" ),
            content.body.as_deref().unwrap_or("" )
        );

        let provider = sqlx::query_as::<_, LlmProviderRow>(r#"SELECT api_endpoint, api_key_enc, model_name, default_params::text AS "default_params"
             FROM socialops.llm_providers WHERE id = $1 AND is_active = true"#).bind(llm_provider_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("LLM provider not found or inactive: {llm_provider_id}" ))?;

        let (api_endpoint, api_key, model_name) =
            (provider.api_endpoint, provider.api_key_enc, provider.model_name);
        let model = model_name.unwrap_or_else(|| "gpt-4o-mini".to_string());

        sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'running' WHERE id = $1" ).bind(task_id)
            .execute(&self.db)
            .await?;

        let endpoint = format!("{}/chat/completions" , api_endpoint.trim_end_matches('/'));
        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system" ,
                    "content": format!(
                        "你是一个专业的内容改写助手。{} 请严格按照 JSON 数组格式返回，不要包含任何其他文字。" ,
                        prompt
                    )
                },
                {
                    "role": "user" ,
                    "content": format!(
                        "请对以下内容进行改写，生成 {} 个不同风格的版本（如正式、轻松、营销、学术等风格）。\n\n原文：\n{}" ,
                        target_count, original_text
                    )
                }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.7
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&endpoint)
            .header("Authorization" , format!("Bearer {api_key}" ))
            .header("Content-Type" , "application/json" )
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("LLM API request failed: {e}" ))?;

        if !resp.status().is_success() {
            let status_code = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'failed' WHERE id = $1" ).bind(task_id)
                .execute(&self.db)
                .await?;
            anyhow::bail!("LLM API returned error: status={status_code}, body={error_body}" );
        }

        let llm_response: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM API response: {e}" ))?;

        let llm_content = llm_response["choices" ][0]["message" ]["content" ]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("LLM response missing choices[0].message.content" ))?;

        let versions: Vec<Value> = serde_json::from_str(llm_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM JSON response: {e}" ))?;

        if versions.is_empty() {
            sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'failed' WHERE id = $1" ).bind(task_id)
                .execute(&self.db)
                .await?;
            anyhow::bail!("LLM returned no valid versions" );
        }

        let mut created_versions = Vec::new();
        for (i, version) in versions.iter().enumerate() {
            let seq = (i + 1) as i32;
            let v_title = version["title" ].as_str().unwrap_or("" );
            let v_body = version["body" ].as_str().unwrap_or("" );

            let row = sqlx::query_as::<_, RewriteVersionRow>(r#"INSERT INTO socialops.rewrite_versions (task_id, version_seq, rewritten_title, rewritten_body, llm_raw_response, status)
                 VALUES ($1, $2, $3, $4, $5::jsonb, 'draft')
                 RETURNING id, version_seq, rewritten_title AS "rewritten_title!" , rewritten_body AS "rewritten_body!\" "#).bind(task_id).bind(seq).bind(v_title).bind(v_body).bind(&llm_response)
            .fetch_one(&self.db)
            .await?;

            created_versions.push(serde_json::json!({
                "id": row.id,
                "version_seq": row.version_seq,
                "rewritten_title": row.rewritten_title,
                "rewritten_body": row.rewritten_body,
            }));
        }

        sqlx::query("UPDATE socialops.rewrite_tasks SET status = 'completed' WHERE id = $1" ).bind(task_id)
            .execute(&self.db)
            .await?;

        Ok(serde_json::json!({
            "task_id": task_id,
            "status": "completed" ,
            "versions_count": created_versions.len(),
            "versions": created_versions,
        }))
    }
}
