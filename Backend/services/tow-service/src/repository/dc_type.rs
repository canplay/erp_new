use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DcType {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub remark: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub update_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub type DcTypeListItem = DcType;

pub struct DcTypeRepository {
    pool: sqlx::PgPool,
}

impl DcTypeRepository {
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<DcType>, Box<dyn std::error::Error>> {
        let items: Vec<DcType> = sqlx::query_as!(
            DcType,
            r#"SELECT id, COALESCE(name, '') AS "name!", code, remark, sort_order, status, create_date, update_date
            FROM tow_dc_type"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<DcType>, Box<dyn std::error::Error>> {
        let item: Option<DcType> = sqlx::query_as!(
            DcType,
            r#"SELECT id, COALESCE(name, '') AS "name!", code, remark, sort_order, status, create_date, update_date
            FROM tow_dc_type WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }
}
