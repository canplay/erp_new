//! 车辆类型 Repository

use serde::{Deserialize, Serialize};

/// 车辆类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarType {
    pub id: i64,
    pub type_name: String,
    pub ccid: i64,
    pub py: String,
    pub cost: String,
    pub index: i64,
    pub dc_start_m: String,
    pub dc_start_k: i64,
    pub dc_bstart_m: String,
    pub p_start_m: String,
    pub p_start_h: i64,
    pub p_bstart_m: String,
    pub p_bstart_dm: String,
}

pub type CarTypeListItem = CarType;

/// 车辆类型 Repository
pub struct CarTypeRepository {
    pool: sqlx::PgPool,
}

impl CarTypeRepository {
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有车辆类型
    pub async fn list(&self) -> Result<Vec<CarType>, Box<dyn std::error::Error>> {
        let items: Vec<CarType> = sqlx::query_as!(
            CarType,
            r#"SELECT id,
                COALESCE(type_name, '') AS "type_name!" ,
                COALESCE(ccid, 0) AS "ccid!" ,
                COALESCE(py, '') AS "py!" ,
                COALESCE(cost, '') AS "cost!" ,
                COALESCE("index" , 0) AS "index!" ,
                COALESCE(dc_start_m, '') AS "dc_start_m!" ,
                COALESCE(dc_start_k, 0) AS "dc_start_k!" ,
                COALESCE(dc_bstart_m, '') AS "dc_bstart_m!" ,
                COALESCE(p_start_m, '') AS "p_start_m!" ,
                COALESCE(p_start_h, 0) AS "p_start_h!" ,
                COALESCE(p_bstart_m, '') AS "p_bstart_m!" ,
                COALESCE(p_bstart_dm, '') AS "p_bstart_dm!"
            FROM tow_car_type"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(&self, id: i64) -> Result<Option<CarType>, Box<dyn std::error::Error>> {
        let item: Option<CarType> = sqlx::query_as!(
            CarType,
            r#"SELECT id,
                COALESCE(type_name, '') AS "type_name!" ,
                COALESCE(ccid, 0) AS "ccid!" ,
                COALESCE(py, '') AS "py!" ,
                COALESCE(cost, '') AS "cost!" ,
                COALESCE("index" , 0) AS "index!" ,
                COALESCE(dc_start_m, '') AS "dc_start_m!" ,
                COALESCE(dc_start_k, 0) AS "dc_start_k!" ,
                COALESCE(dc_bstart_m, '') AS "dc_bstart_m!" ,
                COALESCE(p_start_m, '') AS "p_start_m!" ,
                COALESCE(p_start_h, 0) AS "p_start_h!" ,
                COALESCE(p_bstart_m, '') AS "p_bstart_m!" ,
                COALESCE(p_bstart_dm, '') AS "p_bstart_dm!"
            FROM tow_car_type
            WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }
}
