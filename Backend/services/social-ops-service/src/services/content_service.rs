//! Content service
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use sha2::{Sha256, Digest};
use sqlx::FromRow;

fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[derive(Clone)]
pub struct ContentService {
    db: PgPool,
}

#[derive(FromRow)]
struct ContentListItemRow {
    id: Uuid,
    source_type: String,
    content_type: String,
    title: Option<String>,
    body: Option<String>,
    status: String,
    created_str: String,
}

#[derive(FromRow)]
struct ContentItemRow {
    id: Uuid,
    title: String,
    body: String,
    content_type: String,
    status: String,
}

impl ContentService {
    #[must_use]
    pub const fn new(db: PgPool) -> Self { Self { db } }

    pub async fn list(&self, status: Option<&str>, page: i64, page_size: i64) -> Result<(Vec<Value>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;
        let rows = sqlx::query_as::<_, ContentListItemRow>(r#"SELECT id, source_type, content_type, title, body, status,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_str"
             FROM socialops.content_items
             WHERE ($1::text IS NULL OR status = $1)
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"#).bind(status).bind(page_size).bind(offset)
        .fetch_all(&self.db).await?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.content_items WHERE ($1::text IS NULL OR status = $1)" ).bind(status)
        .fetch_one(&self.db).await
        .unwrap_or(Some(0))
        .unwrap_or(0);

        let items = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.id, "source_type": row.source_type, "content_type": row.content_type,
                "title": row.title, "body_preview": row.body.as_deref().unwrap_or(" ").chars().take(200).collect::<String>(),
                "status": row.status, "created_at": row.created_str
            })
        }).collect();

        Ok((items, total))
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Value>, sqlx::Error> {
        let row = sqlx::query_as::<_, ContentListItemRow>(r#"SELECT id, source_type, content_type, title, body, status,
               to_char(created_at, 'YYYY-MM-DD HH24:MI:SS') AS "created_str"
             FROM socialops.content_items WHERE id = $1"#).bind(id)
        .fetch_optional(&self.db).await?;

        Ok(row.map(|row| {
            serde_json::json!({
                "id": row.id, "source_type": row.source_type, "content_type": row.content_type,
                "title": row.title, "body": row.body, "status": row.status, "created_at": row.created_str
            })
        }))
    }

    pub async fn create(&self, title: &str, body: &str, content_type: &str, source_url: Option<&str>) -> Result<Value, sqlx::Error> {
        let source_hash = source_url.map(hash_url);
        let row = sqlx::query_as::<_, ContentItemRow>(r#"INSERT INTO socialops.content_items (title, body, content_type, source_url, source_hash, source_type, status)
             VALUES ($1, $2, $3, $4, $5, 'manual', 'draft')
             RETURNING id, title AS "title!" , body AS "body!" , content_type, status"#).bind(title).bind(body).bind(content_type).bind(source_url).bind(source_hash.as_deref())
        .fetch_one(&self.db).await?;

        Ok(serde_json::json!({
            "id": row.id, "title": row.title, "body": row.body,
            "content_type": row.content_type, "status": row.status
        }))
    }

    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<bool, sqlx::Error> {
        let r = sqlx::query("UPDATE socialops.content_items SET status = $1, updated_at = NOW() WHERE id = $2" ).bind(status).bind(id)
        .execute(&self.db).await?;
        Ok(r.rows_affected() > 0)
    }
}
