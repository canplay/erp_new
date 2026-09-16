use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

#[derive(Clone)]
pub struct LlmService {
    db: PgPool,
}

impl LlmService {
    #[must_use]
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_providers(&self) -> Result<Vec<Value>, sqlx::Error> {
        sqlx::query(r#"SELECT id, provider_name, api_endpoint, model_name, api_key_enc, is_active
             FROM socialops.llm_providers ORDER BY created_at DESC"#)
        .fetch_all(&self.db)
        .await
        .map(|rows| {
            rows.into_iter().map(|row| {
                serde_json::json!({
                    "id": row.id, "provider_name": row.provider_name, "api_endpoint": row.api_endpoint,
                    "model_name": row.model_name, "is_active": row.is_active
                })
            }).collect()
        })
    }

    pub async fn add_provider(&self, name: &str, endpoint: &str, api_key: &str, model: &str) -> Result<Value, sqlx::Error> {
        sqlx::query(r#"INSERT INTO socialops.llm_providers (provider_name, api_endpoint, api_key_enc, model_name)
             VALUES ($1, $2, $3, $4)
             RETURNING id, provider_name, api_endpoint, model_name, is_active"#).bind(name).bind(endpoint).bind(api_key).bind(model).bind()
        .fetch_one(&self.db)
        .await
        .map(|row| {
            serde_json::json!({
                "id": row.id, "provider_name": row.provider_name, "api_endpoint": row.api_endpoint,
                "model_name": row.model_name, "is_active": row.is_active
            })
        })
    }

    pub async fn delete_provider(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let r = sqlx::query("DELETE FROM socialops.llm_providers WHERE id = $1" ).bind(id)
            .execute(&self.db).await?;
        Ok(r.rows_affected() > 0)
    }
}
