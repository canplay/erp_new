//! 拖车原因 Repository

use serde::{Deserialize, Serialize};

/// 拖车原因
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DcCauses {
    pub id: i64,
    pub name: String,
    pub type_id: Option<i64>,
    pub type_name: Option<String>,
    pub remark: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
}

pub type DcCausesListItem = DcCauses;

/// 拖车原因 Repository
pub struct DcCausesRepository {
    pool: sqlx::PgPool,
}

impl DcCausesRepository {
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有拖车原因
    pub async fn list(&self) -> Result<Vec<DcCauses>, Box<dyn std::error::Error>> {
        let items: Vec<DcCauses> = sqlx::query_as!(
            DcCauses,
            r#"SELECT id,
                COALESCE(name, '') AS "name!" ,
                type_id,
                type_name,
                remark,
                sort_order,
                status
            FROM tow_dc_causes"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<DcCauses>, Box<dyn std::error::Error>> {
        let item: Option<DcCauses> = sqlx::query_as!(
            DcCauses,
            r#"SELECT id,
                COALESCE(name, '') AS "name!" ,
                type_id,
                type_name,
                remark,
                sort_order,
                status
            FROM tow_dc_causes
            WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }

    /// 根据类型 ID 查询
    pub async fn find_by_type_id(
        &self,
        dct_id: i64,
    ) -> Result<Vec<DcCauses>, Box<dyn std::error::Error>> {
        let items: Vec<DcCauses> = sqlx::query_as!(
            DcCauses,
            r#"SELECT id,
                COALESCE(name, '') AS "name!" ,
                type_id,
                type_name,
                remark,
                sort_order,
                status
            FROM tow_dc_causes
            WHERE type_id = $1"#,
            dct_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }
}
