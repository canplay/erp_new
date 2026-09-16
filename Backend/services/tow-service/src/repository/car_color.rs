//! 车辆颜色 Repository

use serde::{Deserialize, Serialize};

/// 车辆颜色
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarColor {
    pub id: i64,
    pub name: String,
    pub remark: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
}

pub type CarColorListItem = CarColor;

/// 车辆颜色 Repository
pub struct CarColorRepository {
    pool: sqlx::PgPool,
}

impl CarColorRepository {
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有车辆颜色
    pub async fn list(&self) -> Result<Vec<CarColor>, Box<dyn std::error::Error>> {
        let items: Vec<CarColor> = sqlx::query_as!(
            CarColor,
            r#"SELECT id,
                COALESCE(name, '') AS "name!" ,
                remark,
                sort_order,
                status
            FROM tow_car_color"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<CarColor>, Box<dyn std::error::Error>> {
        let item: Option<CarColor> = sqlx::query_as!(
            CarColor,
            r#"SELECT id,
                COALESCE(name, '') AS "name!" ,
                remark,
                sort_order,
                status
            FROM tow_car_color
            WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }
}
