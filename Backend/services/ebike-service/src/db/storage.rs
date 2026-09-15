//! 仓储数据仓库
//!
//! 提供仓储信息的分表 CRUD、历史轨迹及批量导入功能。

use chrono::{Datelike, Local};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Error;

use crate::db::{safe_table_name, GenericRepository};
use crate::model::StorageInfo;

pub struct StorageRepository {
    pool: PgPool,
}

impl GenericRepository for StorageRepository {
    type Model = StorageInfo;

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl StorageRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add(&self, info: &StorageInfo) -> Result<bool, Error> {
        self.add_history(info).await?;

        let rows: Vec<String> = sqlx::query_scalar!(
            "SELECT code FROM public.storage WHERE code = $1",
            &info.code,
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            sqlx::query!(
                "INSERT INTO public.storage (code, status, provide, gps, create_date, update_date, \
                 delete, alert, remark, sum, cur, points, gps_type) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                &info.code,
                info.status,
                &info.provide,
                info.gps.as_ref(),
                Local::now().naive_local(),
                Local::now().naive_local(),
                false,
                info.alert.as_deref(),
                info.remark.as_deref(),
                info.sum,
                info.cur,
                info.points.as_deref(),
                info.gps_type,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE public.storage SET provide = $1, gps = $2, update_date = $3, delete = $4, \
                 alert = $5, remark = $6, sum = $7, cur = $8, gps_type = $9, status = $10 WHERE code = $11",
                &info.provide,
                info.gps.as_ref(),
                Local::now().naive_local(),
                false,
                info.alert.as_deref(),
                info.remark.as_deref(),
                info.sum,
                info.cur,
                info.gps_type,
                info.status,
                &info.code,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(true)
    }

    pub async fn add_batch(&self, items: &[StorageInfo]) -> Result<bool, Error> {
        for item in items {
            self.add(item).await?;
        }
        Ok(true)
    }

    pub async fn del(&self, code: &str) -> Result<bool, Error> {
        sqlx::query!(
            "UPDATE public.storage SET update_date = $1, delete = $2 WHERE code = $3",
            Local::now().naive_local(),
            true,
            code,
        )
        .execute(&self.pool)
        .await?;
        Ok(true)
    }

    pub async fn query(
        &self,
        code: &str,
        provide: &str,
        status: i64,
    ) -> Result<Vec<StorageInfo>, Error> {
        let rows = sqlx::query_as!(
            StorageInfo,
            r#"SELECT code,
                      COALESCE(status, 0) AS "status!",
                      COALESCE(provide, '') AS "provide!",
                      gps,
                      COALESCE(type, 0) AS "type!",
                      COALESCE(sum, 0) AS "sum!",
                      COALESCE(cur, 0) AS "cur!",
                      alert, remark, points, delete, create_date, update_date,
                      COALESCE(gps_type, 0) AS "gps_type!"
               FROM public.storage
               WHERE delete = false
                 AND ($1 = '' OR code = $1)
                 AND ($2 = '' OR provide = $2)
                 AND ($3::BIGINT = -1 OR status = $3::BIGINT)"#,
            code,
            provide,
            status,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn history(&self, code: &str) -> Result<Vec<StorageInfo>, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = safe_table_name("storage_history", year, month)
            .map_err(|e| Error::Protocol(format!("Invalid table name: {e}").into()))?;
        let query = format!(
            "SELECT code, status, provide, gps, type, sum, cur, alert, remark, points, delete, \
             create_date, update_date, gps_type FROM public.{table_name} \
             WHERE code = $1 AND delete = false"
        );
        let rows = sqlx::query_as::<_, StorageInfo>(&query)
            .bind(code)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn add_history(&self, info: &StorageInfo) -> Result<bool, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = safe_table_name("storage_history", year, month)
            .map_err(|e| Error::Protocol(format!("Invalid table name: {e}").into()))?;
        // FIX [SQL-INJ-002]: 表名已通过 safe_table_name 白名单校验（格式：[a-zA-Z0-9_]+_YYYY_MM）
        // O4 修复: 确保动态表存在（月初首次写入必炸）
        let create_sql = format!(
            "CREATE TABLE IF NOT EXISTS public.{table_name} (
                id TEXT PRIMARY KEY,
                code TEXT,
                status BIGINT,
                provide TEXT,
                gps JSONB,
                create_date TIMESTAMP,
                update_date TIMESTAMP,
                delete BOOLEAN,
                alert TEXT,
                remark TEXT,
                sum NUMERIC,
                cur NUMERIC,
                points TEXT
            )"
        );
        let _ = sqlx::query(&create_sql)
            .execute(&self.pool)
            .await;

        let insert_sql = format!(
            "INSERT INTO public.{table_name} (id, code, status, provide, gps, create_date, update_date, \
             delete, alert, remark, sum, cur, points) \
             VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        );
        sqlx::query(&insert_sql)
            .bind(&info.code)
            .bind(info.status)
            .bind(&info.provide)
            .bind(info.gps.as_ref().unwrap_or(&json!({ "lng": 0, "lat": 0 })))
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .bind(false)
            .bind(info.alert.as_ref().unwrap_or(&String::new()))
            .bind(info.remark.as_ref().unwrap_or(&String::new()))
            .bind(info.sum)
            .bind(info.cur)
            .bind(info.points.as_ref().unwrap_or(&String::new()))
            .execute(&self.pool).await?;
        Ok(true)
    }
}

// ============ 数据库索引（幂等，首次启动自动创建） ============
/// 为 storage 表添加复合索引：provide + status（用于条件查询）
pub async fn ensure_storage_indexes(pool: &PgPool) -> Result<(), Error> {
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_storage_provide_status ON public.storage(provide, status)",
    )
    .execute(pool)
    .await;

    Ok(())
}
