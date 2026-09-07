//! 车辆分类 Repository

use serde::{Deserialize, Serialize};

/// 车辆分类
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarClass {
    pub id: i64,
    pub cpt: String,
    pub remark: String,
    pub free_time: i64,
    pub hm10: String,
    pub hm24: String,
    pub gratis_day: i64,
    pub cost_day: String,
}

pub type CarClassListItem = CarClass;

/// 车辆分类 Repository
pub struct CarClassRepository {
    pool: sqlx::PgPool,
}

impl CarClassRepository {
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有车辆分类
    pub async fn list(&self) -> Result<Vec<CarClass>, Box<dyn std::error::Error>> {
        let items: Vec<CarClass> = sqlx::query_as!(
            CarClass,
            r#"SELECT id,
                COALESCE(cpt, '') AS "cpt!",
                COALESCE(remark, '') AS "remark!",
                COALESCE(free_time, 0) AS "free_time!",
                COALESCE(hm10, '') AS "hm10!",
                COALESCE(hm24, '') AS "hm24!",
                COALESCE(gratis_day, 0) AS "gratis_day!",
                COALESCE(cost_day, '') AS "cost_day!"
            FROM tow_car_class"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<CarClass>, Box<dyn std::error::Error>> {
        let item: Option<CarClass> = sqlx::query_as!(
            CarClass,
            r#"SELECT id,
                COALESCE(cpt, '') AS "cpt!",
                COALESCE(remark, '') AS "remark!",
                COALESCE(free_time, 0) AS "free_time!",
                COALESCE(hm10, '') AS "hm10!",
                COALESCE(hm24, '') AS "hm24!",
                COALESCE(gratis_day, 0) AS "gratis_day!",
                COALESCE(cost_day, '') AS "cost_day!"
            FROM tow_car_class
            WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }
}
