//! 拖车原因类型 Repository

use serde::{Deserialize, Serialize};

/// 拖车原因类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DcType {
    pub id: i64,
    pub cpt: String,
    pub unit_id: String,
    pub py: String,
    pub index: i32,
}

pub type DcTypeListItem = DcType;

/// 拖车原因类型 Repository
pub struct DcTypeRepository {
    pool: sqlx::PgPool,
}

impl DcTypeRepository {
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有拖车原因类型
    pub async fn list(&self) -> Result<Vec<DcType>, Box<dyn std::error::Error>> {
        let items: Vec<DcType> = sqlx::query_as!(
            DcType,
            r#"SELECT id,
                COALESCE(cpt, '') AS "cpt!",
                COALESCE(unit_id, '') AS "unit_id!",
                COALESCE(py, '') AS "py!",
                COALESCE("index"::int, 0) AS "index!"
            FROM tow_dc_type"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(&self, id: i64) -> Result<Option<DcType>, Box<dyn std::error::Error>> {
        let item: Option<DcType> = sqlx::query_as!(
            DcType,
            r#"SELECT id,
                COALESCE(cpt, '') AS "cpt!",
                COALESCE(unit_id, '') AS "unit_id!",
                COALESCE(py, '') AS "py!",
                COALESCE("index"::int, 0) AS "index!"
            FROM tow_dc_type
            WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }
}
