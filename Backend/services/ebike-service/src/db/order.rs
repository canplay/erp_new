//! 订单数据仓库
//!
//! 提供订单按月分表的 CRUD 操作及多维度条件查询。

use chrono::{Datelike, Local};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Error;

use crate::db::GenericRepository;
use crate::model::OrderInfo;

pub struct OrderRepository {
    pool: PgPool,
}

impl GenericRepository for OrderRepository {
    type Model = OrderInfo;

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl OrderRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add(&self, info: &OrderInfo) -> Result<String, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = format!("order_{year}_{month}");
        // 确保动态表存在（月初首次写入必炸，见 O4）
        let _ = sqlx::query(
            &format!(
                "CREATE TABLE IF NOT EXISTS public.{table_name} (
                    id TEXT PRIMARY KEY,
                    code TEXT,
                    status BIGINT,
                    provide TEXT,
                    speed BIGINT,
                    gps JSONB,
                    time JSONB,
                    create_date TIMESTAMP,
                    update_date TIMESTAMP,
                    delete BOOLEAN,
                    alert TEXT,
                    remark TEXT,
                    gps_type BIGINT,
                    hash TEXT,
                    payable NUMERIC,
                    pay NUMERIC,
                    refund NUMERIC,
                    coupon NUMERIC,
                    \"order\" TEXT,
                    pay_type BIGINT,
                    pay_time TIMESTAMP,
                    pay_status BIGINT,
                    paytype TEXT,
                    paytime TIMESTAMP
                )"
            )
        )
        .execute(&self.pool)
        .await;

        if info.hash.is_empty() {
            sqlx::query(
                &format!(
                    "INSERT INTO public.{} (id, code, status, provide, speed, gps, time, create_date, \
                     update_date, delete, alert, remark, gps_type, hash, payable, pay, refund, coupon, \
                     \"order\", pay_type, pay_time, pay_status, paytype, paytime) \
                     VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, \
                     $12, gen_random_uuid()::TEXT, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)",
                    table_name
                )
            )
            .bind(info.code.as_ref().unwrap_or(&String::new()))
            .bind(info.status)
            .bind(&info.provide)
            .bind(info.speed)
            .bind(info.gps.as_ref().unwrap_or(&json!({})))
            .bind(info.time.as_ref().unwrap_or(&json!({})))
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .bind(false)
            .bind(info.alert.as_ref().unwrap_or(&String::new()))
            .bind(info.remark.as_ref().unwrap_or(&String::new()))
            .bind(info.gps_type.unwrap_or(0))
            .bind(info.payable)
            .bind(info.pay)
            .bind(info.refund)
            .bind(info.coupon)
            .bind(info.order.as_ref().unwrap_or(&String::new()))
            .bind(info.pay_type)
            .bind(info.pay_time.unwrap_or(Local::now().naive_local()))
            .bind(info.pay_status)
            .bind(info.paytype)
            .bind(info.paytime.unwrap_or(Local::now().naive_local()))
            .execute(&self.pool).await?;

            // 查询生成的 hash
            let row = sqlx::query_as::<_, (String,)>(&format!(
                "SELECT hash FROM public.{} ORDER BY create_date DESC LIMIT 1", table_name
            ))
            .fetch_one(&self.pool).await?;
            Ok(row.0)
        } else {
            let exists = sqlx::query_scalar::<_, i64>(
                &format!(
                    "SELECT COUNT(*) FROM public.{} WHERE hash = $1 AND delete = false",
                    table_name
                )
            )
            .bind(&info.hash)
            .fetch_one(&self.pool)
            .await?;

            if exists == 0 {
                sqlx::query(
                    &format!(
                        "UPDATE public.{} SET code = $1, status = $2, provide = $3, speed = $4, \
                         gps = $5, time = $6, update_date = $7, delete = $8, alert = $9, remark = $10, \
                         gps_type = $11, payable = $12, pay = $13, refund = $14, coupon = $15, \
                         \"order\" = $16, pay_type = $17, pay_time = $18, pay_status = $19 \
                         WHERE hash = $20",
                        table_name
                    )
                )
                .bind(info.code.as_ref().unwrap_or(&String::new()))
                .bind(info.status)
                .bind(&info.provide)
                .bind(info.speed)
                .bind(info.gps.as_ref().unwrap_or(&json!({})))
                .bind(info.time.as_ref().unwrap_or(&json!({})))
                .bind(Local::now().naive_local())
                .bind(false)
                .bind(info.alert.as_ref().unwrap_or(&String::new()))
                .bind(info.remark.as_ref().unwrap_or(&String::new()))
                .bind(info.gps_type.unwrap_or(0))
                .bind(info.payable)
                .bind(info.pay)
                .bind(info.refund)
                .bind(info.coupon)
                .bind(info.order.as_ref().unwrap_or(&String::new()))
                .bind(info.pay_type)
                .bind(info.pay_time.unwrap_or(Local::now().naive_local()))
                .bind(info.pay_status)
                .bind(&info.hash)
                .execute(&self.pool).await?;

                Ok(info.hash.clone())
            } else {
                Ok("duplicate".to_string())
            }
        }
    }

    pub async fn add_batch(&self, items: &[OrderInfo]) -> Result<bool, Error> {
        for item in items {
            self.add(item).await?;
        }
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn query(
        &self,
        code: &str,
        provide: &str,
        status: i64,
        _time_start: &str,
        _time_end: &str,
        order: &str,
        paystatus: i64,
        paytype: i64,
        paytime: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OrderInfo>, Error> {
        let year = Local::now().year();
        let month = Local::now().month();
        let table_name = format!("order_{year}_{month}");

        let rows = sqlx::query_as::<_, OrderInfo>(
            &format!(
                "SELECT * FROM public.{table_name} \
                 WHERE delete = false \
                 AND ($1 = '' OR code = $1) \
                 AND ($2 = '' OR provide = $2) \
                 AND ($3 = -1 OR status = $3) \
                 AND ($4 = '' OR \"order\" = $4) \
                 AND ($5 = -1 OR paystatus = $5) \
                 AND ($6 = -1 OR paytype = $6) \
                 AND ($7 = '' OR paytime::text = $7) \
                 LIMIT $8 OFFSET $9",
            )
        )
        .bind(code)
        .bind(provide)
        .bind(status)
        .bind(order)
        .bind(paystatus)
        .bind(paytype)
        .bind(paytime)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            Ok(vec![])
        } else {
            Ok(rows)
        }
    }
}

// ============ 数据库索引（幂等，首次启动自动创建） ============
/// 为 order 表添加复合索引：provide + status + create_date（用于条件查询和排序）
pub async fn ensure_order_indexes(pool: &PgPool) -> Result<(), Error> {
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_order_provide_status_create ON public.order(provide, status, create_date)",
    )
    .execute(pool)
    .await;

    Ok(())
}
