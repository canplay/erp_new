// 支付服务
// Pay service

use redis::AsyncCommands;
use sqlx::PgPool;
use sqlx::types::BigDecimal;

use crate::error::{PayError, Result};
use crate::models::{PayCreateParams, PayOrder, PayQuery};

/// 支付服务
pub struct PayService {
    /// 数据库连接池（共享，由外部传入）
    pub pool: PgPool,
    /// `Redis连接URL`
    redis_url: String,
}

/// 数据库行类型
struct PayRow {
    id: String,
    order: String,
    status: String,
    pay_type: String,
    order_pay: Option<serde_json::Value>,
    amount: i32,
    remark: String,
    create_service: String,
    create_params: Option<serde_json::Value>,
    create_date: chrono::NaiveDateTime,
    update_date: chrono::NaiveDateTime,
}

impl PayService {
    /// 创建支付服务实例（接收外部传入的共享 PgPool）
    #[must_use]
    pub fn new(pool: PgPool, redis_url: String) -> Self {
        Self {
            pool,
            redis_url,
        }
    }

    /// 查询支付订单数量 (修复: SQL注入-参数绑定)
    pub async fn count(&self, query: &PayQuery) -> Result<i64> {
        let remark_like = query.remark.as_deref().map(|r| format!("%{r}%"));

        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM pay WHERE "id" IS NOT NULL
               AND ($1 = '' OR status = $1)
               AND ($2 = '' OR "type" = $2)
               AND ($3 = '' OR remark LIKE $3)"#,
            query.status.as_deref().unwrap_or(""),
            query.pay_type.as_deref().unwrap_or(""),
            remark_like.as_deref().unwrap_or(""),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(PayError::DatabaseError)?
        .unwrap_or(0);

        Ok(count)
    }

    /// 查询支付订单列表 (修复: SQL注入-参数绑定+ORDER BY白名单)
    pub async fn list(&self, query: &PayQuery) -> Result<Vec<PayOrder>> {
        let remark_like = query.remark.as_deref().map(|r| format!("%{r}%"));
        let sort_by = query.sort_by.as_deref().unwrap_or("");
        let max_page = query.max_page.unwrap_or(10);
        let cur_page = query.cur_page.unwrap_or(0);

        let rows: Vec<PayRow> = if query.descending.unwrap_or(true) {
            sqlx::query_as!(
                PayRow,
                r#"SELECT id, "order",
                          COALESCE(status, '') AS "status!",
                          COALESCE("type", '') AS "pay_type!",
                          order_pay,
                          COALESCE(amount, 0)::int AS "amount!",
                          COALESCE(remark, '') AS "remark!",
                          COALESCE(create_service, '') AS "create_service!",
                          create_params,
                          COALESCE(create_date, NOW())::timestamp AS "create_date!",
                          COALESCE(update_date, NOW())::timestamp AS "update_date!"
                   FROM pay
                   WHERE "id" IS NOT NULL
                     AND ($1 = '' OR status = $1)
                     AND ($2 = '' OR "type" = $2)
                     AND ($3 = '' OR remark LIKE $3)
                   ORDER BY CASE WHEN $4 = 'id' THEN id
                                 WHEN $4 = 'order' THEN "order"
                                 WHEN $4 = 'status' THEN status
                                 WHEN $4 = 'type' THEN "type"
                                 WHEN $4 = 'remark' THEN remark
                                 ELSE create_date::text END DESC
                   LIMIT $5 OFFSET $6"#,
                query.status.as_deref().unwrap_or(""),
                query.pay_type.as_deref().unwrap_or(""),
                remark_like.as_deref().unwrap_or(""),
                sort_by,
                max_page,
                cur_page,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(PayError::DatabaseError)?
        } else {
            sqlx::query_as!(
                PayRow,
                r#"SELECT id, "order",
                          COALESCE(status, '') AS "status!",
                          COALESCE("type", '') AS "pay_type!",
                          order_pay,
                          COALESCE(amount, 0)::int AS "amount!",
                          COALESCE(remark, '') AS "remark!",
                          COALESCE(create_service, '') AS "create_service!",
                          create_params,
                          COALESCE(create_date, NOW())::timestamp AS "create_date!",
                          COALESCE(update_date, NOW())::timestamp AS "update_date!"
                   FROM pay
                   WHERE "id" IS NOT NULL
                     AND ($1 = '' OR status = $1)
                     AND ($2 = '' OR "type" = $2)
                     AND ($3 = '' OR remark LIKE $3)
                   ORDER BY CASE WHEN $4 = 'id' THEN id
                                 WHEN $4 = 'order' THEN "order"
                                 WHEN $4 = 'status' THEN status
                                 WHEN $4 = 'type' THEN "type"
                                 WHEN $4 = 'remark' THEN remark
                                 ELSE create_date::text END ASC
                   LIMIT $5 OFFSET $6"#,
                query.status.as_deref().unwrap_or(""),
                query.pay_type.as_deref().unwrap_or(""),
                remark_like.as_deref().unwrap_or(""),
                sort_by,
                max_page,
                cur_page,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(PayError::DatabaseError)?
        };

        let orders = rows
            .into_iter()
            .map(|row| PayOrder {
                id: row.id,
                order: row.order,
                status: row.status,
                pay_type: row.pay_type,
                order_pay: row.order_pay,
                amount: row.amount,
                remark: row.remark,
                create_service: row.create_service,
                create_params: row.create_params,
                create_date: row.create_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                update_date: row.update_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        Ok(orders)
    }

    /// 查询最近已支付订单信息
    pub async fn latest(&self, user_id: &str) -> Result<Option<PayOrder>> {
        let client = redis::Client::open(self.redis_url.as_str()).map_err(PayError::RedisError)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(PayError::RedisError)?;

        let cache_key = format!("pay:{user_id}");

        // 尝试从Redis获取缓存
        let cached: Option<String> = conn.get(&cache_key).await.map_err(PayError::RedisError)?;

        if let Some(cached_data) = cached
            && !cached_data.is_empty() {
                let order: PayOrder = serde_json::from_str(&cached_data)
                    .map_err(|e| PayError::InternalError(e.to_string()))?;
                return Ok(Some(order));
            }

        // 从数据库查询
        let result: Option<PayRow> = sqlx::query_as!(
            PayRow,
            r#"SELECT id, "order",
                      COALESCE(status, '') AS "status!",
                      COALESCE("type", '') AS "pay_type!",
                      order_pay,
                      COALESCE(amount, 0)::int AS "amount!",
                      COALESCE(remark, '') AS "remark!",
                      COALESCE(create_service, '') AS "create_service!",
                      create_params,
                      COALESCE(create_date, NOW())::timestamp AS "create_date!",
                      COALESCE(update_date, NOW())::timestamp AS "update_date!"
               FROM pay
               WHERE "id" IS NOT NULL AND status = 'paid' AND remark LIKE $1
               ORDER BY update_date DESC
               LIMIT 1"#,
            format!("%{user_id}%"),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(PayError::DatabaseError)?;

        if let Some(row) = result {
            let order = PayOrder {
                id: row.id,
                order: row.order,
                status: row.status,
                pay_type: row.pay_type,
                order_pay: row.order_pay,
                amount: row.amount,
                remark: row.remark,
                create_service: row.create_service,
                create_params: row.create_params,
                create_date: row.create_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                update_date: row.update_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            // 缓存到Redis
            let order_json = serde_json::to_string(&order)
                .map_err(|e| PayError::InternalError(e.to_string()))?;
            let _: () = conn
                .set_ex(&cache_key, order_json, 600)
                .await
                .map_err(PayError::RedisError)?;

            Ok(Some(order))
        } else {
            Ok(None)
        }
    }

    /// 创建支付订单
    pub async fn create_order(&self, params: &PayCreateParams) -> Result<PayOrder> {
        let id = uuid::Uuid::new_v4().to_string();
        let status = params.status.as_deref().unwrap_or("pending");
        let pay_type = params.pay_type.as_deref().unwrap_or("default");
        let amount = params.amount.unwrap_or(0);
        let remark = params.remark.as_deref().unwrap_or("");
        let now = chrono::Utc::now();

        let create_date = params
            .create_date
            .clone()
            .unwrap_or_else(|| now.format("%Y-%m-%d %H:%M:%S").to_string());
        let create_date_dt = chrono::NaiveDateTime::parse_from_str(&create_date, "%Y-%m-%d %H:%M:%S")
            .map(|n| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(n, chrono::Utc))
            .unwrap_or(now);

        sqlx::query!(
            "INSERT INTO pay (id, \"order\", status, \"type\", order_pay, amount, remark, create_service, create_params, create_date, update_date) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'pay', $8, $9, $10)",
            &id,
            &params.order,
            status,
            pay_type,
            params.order_pay.as_ref(),
            BigDecimal::from(amount),
            remark,
            params.create_params.as_ref(),
            create_date_dt,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(PayError::DatabaseError)?;

        Ok(PayOrder {
            id,
            order: params.order.clone(),
            status: status.to_string(),
            pay_type: pay_type.to_string(),
            order_pay: params.order_pay.clone(),
            amount,
            remark: remark.to_string(),
            create_service: "pay".to_string(),
            create_params: params.create_params.clone(),
            create_date: create_date.clone(),
            update_date: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }
}

impl Default for PayService {
    fn default() -> Self {
        let pool = sqlx::PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
            .expect("failed to connect to database");
        Self::new(pool, std::env::var("REDIS_URL").unwrap_or_default())
    }
}
