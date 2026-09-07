//! 车辆数据仓库
//!
//! 提供车辆信息的分表 CRUD、历史轨迹及告警记录存储。

// ============ 启动时自动创建的索引摘要 ============
// 以下索引均在首次启动时幂等创建（CREATE INDEX IF NOT EXISTS），无需手动建表：
//
// - idx_car_provide_status_delete (provide, status, delete)
//   加速运营方过滤的车辆条件查询：按 provide 数据隔离 + status 筛选 + 软删除过滤
//   （car.query、del 的 where 条件、check_quota 的计数）。
// - idx_car_code (UNIQUE, code)
//   唯一约束 code 并加速按 code 的精确查询/去重
//   （car.add 的 INSERT/UPDATE 判定、car.query、car.history）。
// - idx_violation_provide_status (provide, status)
//   加速违停记录按运营方 + 状态联合筛选（list_violations、resolve_violation）。
// - idx_operators_provide (UNIQUE, provide)
//   运营方唯一约束并加速配额 LEFT JOIN 关联查询（check_quota 去重）。

use chrono::{Datelike, Local};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Error;

use crate::db::GenericRepository;
use crate::model::CarInfo;

/// 违停记录
#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct ViolationRecord {
    pub id: i64,
    pub car_code: String,
    pub provide: String,
    pub lng: String,
    pub lat: String,
    pub violation_type: String,
    pub status: i32,
    pub created_at: chrono::NaiveDateTime,
    pub resolved_at: Option<chrono::NaiveDateTime>,
    pub remark: Option<String>,
}

pub struct CarRepository {
    pool: PgPool,
}

impl GenericRepository for CarRepository {
    type Model = CarInfo;

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl CarRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add(&self, info: &CarInfo) -> Result<bool, Error> {
        // Insert history
        self.add_history(info).await?;

        let rows: Vec<String> = sqlx::query_scalar!(
            "SELECT code FROM public.car WHERE code = $1",
            &info.code,
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            sqlx::query!(
                "INSERT INTO public.car (code, status, provide, speed, gps, time, create_date, update_date, delete, alert, remark, type, gps_type) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                &info.code,
                info.status,
                &info.provide,
                info.speed,
                info.gps.as_ref(),
                info.time.as_ref(),
                Local::now().naive_local(),
                Local::now().naive_local(),
                false,
                info.alert.as_deref(),
                info.remark.as_deref(),
                info.r#type,
                info.gps_type,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE public.car SET status = $1, provide = $2, speed = $3, gps = $4, time = $5, \
                 update_date = $6, delete = $7, alert = $8, remark = $9, type = $10, gps_type = $11 WHERE code = $12",
                info.status,
                &info.provide,
                info.speed,
                info.gps.as_ref(),
                info.time.as_ref(),
                Local::now().naive_local(),
                false,
                info.alert.as_deref(),
                info.remark.as_deref(),
                info.r#type,
                info.gps_type,
                &info.code,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(true)
    }

    pub async fn add_batch(&self, items: &[CarInfo]) -> Result<bool, Error> {
        for item in items {
            self.add(item).await?;
        }
        Ok(true)
    }

    pub async fn del(&self, code: &str, provide: &str) -> Result<bool, Error> {
        // provide 非空时按运营方过滤，保证数据隔离（运营方只能删自己的车）
        if provide.is_empty() {
            sqlx::query!(
                "UPDATE public.car SET update_date = $1, delete = $2 WHERE code = $3",
                Local::now().naive_local(),
                true,
                code,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE public.car SET update_date = $1, delete = $2 WHERE code = $3 AND provide = $4",
                Local::now().naive_local(),
                true,
                code,
                provide,
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(true)
    }

    pub async fn query(
        &self,
        code: &str,
        provide: &str,
        status: i64,
        time_start: &str,
        time_end: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CarInfo>, Error> {
        let rows = sqlx::query_as!(
            CarInfo,
            r#"SELECT code,
                      COALESCE(status, 0) AS "status!",
                      COALESCE(provide, '') AS "provide!",
                      COALESCE(speed, 0) AS "speed!",
                      gps,
                      COALESCE(type, 0) AS "type!",
                      time, alert, remark, delete, create_date, update_date,
                      COALESCE(gps_type, 0) AS "gps_type!"
               FROM public.car
               WHERE delete = false
                 AND ($1 = '' OR code = $1)
                 AND ($2 = '' OR provide = $2)
                 AND ($3::BIGINT = -1 OR status = $3::BIGINT)
                 AND ($4 = '' OR (time::json->'start')::TEXT LIKE $4)
                 AND ($5 = '' OR (time::json->'end')::TEXT LIKE $5)
               LIMIT $6 OFFSET $7"#,
            code,
            provide,
            status,
            &format!("%{time_start}%"),
            &format!("%{time_end}%"),
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn history(&self, code: &str) -> Result<Vec<CarInfo>, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = format!("car_history_{year}_{month}");
        let rows = sqlx::query_as::<_, CarInfo>(
            &format!(
                "SELECT code, status, provide, speed, gps, COALESCE(type, 0) AS type, time, alert, remark, delete, create_date, \
                 update_date, gps_type FROM public.{} WHERE code = $1 AND delete = false",
                table_name
            )
        )
        .bind(code)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn alert(
        &self,
        code: &str,
        provide: &str,
        status: i64,
        time: &str,
        alert: &str,
        remark: &str,
    ) -> Result<Vec<CarInfo>, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = format!("car_history_{year}_{month}");
        let rows = sqlx::query_as::<_, CarInfo>(
            &format!(
                "SELECT code, status, provide, speed, gps, COALESCE(type, 0) AS type, time, alert, remark, delete, \
                 create_date, update_date, gps_type FROM public.{} \
                 WHERE delete = false \
                 AND ($1 = '' OR code = $1) \
                 AND ($2 = '' OR provide = $2) \
                 AND ($3 = -1 OR status = $3) \
                 AND ($4 = '' OR create_date::TEXT LIKE $4) \
                 AND (($5 = '' AND alert != '') OR ($5 != '' AND alert LIKE $5)) \
                 AND ($6 = '' OR remark LIKE $6)",
                table_name
            )
        )
        .bind(code)
        .bind(provide)
        .bind(status)
        .bind(format!("%{time}%"))
        .bind(format!("%{alert}%"))
        .bind(format!("%{remark}%"))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// 保存违停记录
    pub async fn save_violation(
        &self,
        car_code: &str,
        provide: &str,
        lng: &str,
        lat: &str,
        violation_type: &str,
    ) -> Result<(), Error> {
        // 同车辆同类型未处理的违停不重复创建
        let existing = sqlx::query!(
            "SELECT id FROM violations WHERE car_code = $1 AND violation_type = $2 AND status = 0",
            car_code,
            violation_type,
        )
        .fetch_optional(&self.pool)
        .await?;
        if existing.is_some() {
            return Ok(());
        }
        sqlx::query!(
            "INSERT INTO violations (car_code, provide, lng, lat, violation_type, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, 0, NOW())",
            car_code,
            provide,
            lng,
            lat,
            violation_type,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 查询违停记录（按运营商和状态筛选）
    pub async fn list_violations(
        &self,
        provide: &str,
        status: i64,
    ) -> Result<Vec<ViolationRecord>, Error> {
        // 确保索引已创建（幂等）
        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_violation_provide_status ON violations(provide, status)",
        )
        .execute(&self.pool)
        .await;

        let rows = sqlx::query_as!(
            ViolationRecord,
            r#"SELECT id::BIGINT AS "id!",
                      car_code, provide,
                      COALESCE(lng, '') AS "lng!",
                      COALESCE(lat, '') AS "lat!",
                      COALESCE(violation_type, '') AS "violation_type!",
                      COALESCE(status, 0) AS "status!",
                      COALESCE(created_at, NOW()::timestamp) AS "created_at!",
                      resolved_at, remark
               FROM violations
               WHERE ($1 = '' OR provide = $1)
                 AND ($2::BIGINT = -1 OR status = $2::BIGINT)
               ORDER BY created_at DESC
               LIMIT 200"#,
            provide,
            status,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// 标记违停已处理
    pub async fn resolve_violation(&self, id: i64, remark: &str) -> Result<bool, Error> {
        // 先创建索引（如果尚未存在），加速 provide + status 的联合查询
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_violation_provide_status ON violations(provide, status)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "UPDATE violations SET status = 2, resolved_at = NOW(), remark = $1 WHERE id::BIGINT = $2",
            remark,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(true)
    }

    async fn add_history(&self, info: &CarInfo) -> Result<bool, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = format!("car_history_{year}_{month}");
        // B11 豁免: 按月分表 car_history_{year}_{month}, 表名运行时动态
        // 修复 (2026-08-10): 补 type 列——历史表 type 曾为 NULL 导致 query_as 解码失败
        sqlx::query(
            &format!(
                "INSERT INTO public.{} (id, code, status, provide, speed, gps, type, time, create_date, \
                 update_date, delete, alert, remark, gps_type) \
                 VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                table_name
            )
        )
        .bind(&info.code)
        .bind(info.status)
        .bind(&info.provide)
        .bind(info.speed)
        .bind(info.gps.as_ref().unwrap_or(&json!({ "lng": "", "lat": "" })))
        .bind(info.r#type)
        .bind(info.time.as_ref().unwrap_or(&json!({ "lng": "", "lat": "" })))
        .bind(Local::now().naive_local())
        .bind(Local::now().naive_local())
        .bind(false)
        .bind(info.alert.as_ref().unwrap_or(&String::new()))
        .bind(info.remark.as_ref().unwrap_or(&String::new()))
        .bind(info.gps_type)
        .execute(&self.pool).await?;
        Ok(true)
    }
}

// ============ 数据库索引（幂等，首次启动自动创建） ============
/// 为 car 表添加复合索引：provide + status + delete（用于运营方过滤和条件查询）
pub async fn ensure_car_indexes(pool: &PgPool) -> Result<(), Error> {
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_car_provide_status_delete ON public.car(provide, status, delete)",
    )
    .execute(pool)
    .await;

    // car.code 唯一索引（加速精确查询和去重）
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_car_code ON public.car(code)",
    )
    .execute(pool)
    .await;

    Ok(())
}

// ============ 数据库索引（幂等，首次启动自动创建） ============
/// 为 operators 表添加唯一索引：provide（加速配额查询和去重）
pub async fn ensure_operators_indexes(pool: &PgPool) -> Result<(), Error> {
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_operators_provide ON operators(provide)",
    )
    .execute(pool)
    .await;

    Ok(())
}
