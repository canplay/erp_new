//! LLM Provider service
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use sqlx::FromRow;

#[derive(Clone)]
pub struct LlmService {
    db: PgPool,
}

#[derive(FromRow)]
struct LlmProviderRow {
    id: Uuid,
    provider_name: String,
    api_endpoint: String,
    #[allow(dead_code)] // 保留列映射供未来 LLM 调用实现使用
    api_key_enc: String,
    model_name: String,
    is_active: bool,
}

impl LlmService {
    #[must_use]
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_providers(&self) -> Result<Vec<Value>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LlmProviderRow>(r#"SELECT id, provider_name, api_endpoint, model_name, api_key_enc, is_active
             FROM socialops.llm_providers ORDER BY created_at DESC"#)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.id, "provider_name": row.provider_name, "api_endpoint": row.api_endpoint,
                "model_name": row.model_name, "is_active": row.is_active
            })
        }).collect())
    }

    pub async fn add_provider(&self, name: &str, endpoint: &str, api_key: &str, model: &str) -> Result<Value, sqlx::Error> {
        let row = sqlx::query_as::<_, LlmProviderRow>(r#"INSERT INTO socialops.llm_providers (provider_name, api_endpoint, api_key_enc, model_name)
             VALUES ($1, $2, $3, $4)
             RETURNING id, provider_name, api_endpoint, model_name, api_key_enc, is_active"#).bind(name).bind(endpoint).bind(api_key).bind(model)
        .fetch_one(&self.db)
        .await?;

        Ok(serde_json::json!({
            "id": row.id, "provider_name": row.provider_name, "api_endpoint": row.api_endpoint,
            "model_name": row.model_name, "is_active": row.is_active
        }))
    }

    pub async fn delete_provider(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let r = sqlx::query("DELETE FROM socialops.llm_providers WHERE id = $1" ).bind(id)
            .execute(&self.db).await?;
        Ok(r.rows_affected() > 0)
    }
}
