import os

filepath = r'D:\Workspace\erp_new\Backend\services\tow-service\src\repository\car_type.rs'

content = '''use serde::{Deserialize, Serialize};

/// 车辆类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarType {
    pub id: i64,
    pub name: String,
    pub remark: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub update_date: Option<chrono::DateTime<chrono::Utc>>,
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
            r#"SELECT id, name, remark, sort_order, status, create_date, update_date
            FROM tow_car_type"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(&self, id: i64) -> Result<Option<CarType>, Box<dyn std::error::Error>> {
        let item: Option<CarType> = sqlx::query_as!(
            CarType,
            r#"SELECT id, name, remark, sort_order, status, create_date, update_date
            FROM tow_car_type
            WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(item)
    }
}
'''

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Fixed car_type.rs")
