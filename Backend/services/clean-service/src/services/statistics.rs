//! 统计服务
//!
//! 提供各种统计查询功能，使用参数化查询防止 SQL 注入
//!
//! B11 迁移: 全部动态 sqlx::query_as 已转换为编译期校验的 sqlx::query_scalar! 宏,
//! 可选过滤条件统一采用 `($N = '' OR col = $N)` 静态 SQL 模式(与 repository/order.rs 一致)。

use sqlx::PgPool;

use common::AppError;
use common::AppResult;
use crate::models::{FormalBillQuery, PaymentInfoQuery, PaymentStatisticsQuery, PaymentWebQuery, PaymentWebStatisticsQuery, StatisticsResult};

/// 统计服务
pub struct StatisticsService {pool: PgPool}

impl StatisticsService {/// 创建统计服务实例
 ///
  /// 创建统计服务实例
  /// 审计修复 (H8-架构评审): 注入共享 PgPool, 不再每仓储自建连接池(原 4 池 x 10 连接)
  #[must_use]
 pub fn new(pool: PgPool) -> Self {
 Self { pool}
 }

 /// 统计正式账单数量
 ///
 /// 使用参数化查询防止 SQL 注入
 pub async fn formal_bill_count(&self, query: &FormalBillQuery) -> AppResult<i64> {
     let create_date_start = query.create_date_start.as_deref().unwrap_or("" );
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().map(|n| format!("%{n}%" )).unwrap_or_default();
     let fzr = query.fzr.as_deref().map(|f| format!("%{f}%" )).unwrap_or_default();
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };

     let row = sqlx::query_scalar!(
         r#"
         SELECT COUNT(id) FROM sf_formal_bill
         WHERE id != ''
           AND ($1 = '' OR create_date > $1)
           AND ($2 = '' OR create_date < $2)
           AND ($3 = '' OR numbering LIKE $3)
           AND ($4 = '' OR fzr_id LIKE $4)
           AND ($5 = '' OR status = $5)
         "#,
         create_date_start, create_date_end, numbering, fzr, status,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
 }

 /// 统计正式账单总额
 ///
 /// 使用参数化查询防止 SQL 注入
 pub async fn formal_bill_total(&self, query: &FormalBillQuery) -> AppResult<StatisticsResult> {
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );

     // 使用 COALESCE 处理 NULL 值; ::bigint 将 numeric 聚合结果转为 i64
     let total = sqlx::query_scalar!(
         r#"
         SELECT COALESCE(SUM(money), 0)::bigint FROM sf_formal_bill
         WHERE id != ''
           AND ($1 = '' OR create_date >= $1)
           AND ($2 = '' OR create_date <= $2)
         "#,
         create_date_start, create_date_end,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     let paid = sqlx::query_scalar!(
         r#"
         SELECT COALESCE(SUM(money), 0)::bigint FROM sf_formal_bill
         WHERE id != '' AND status = 'Y'
           AND ($1 = '' OR create_date >= $1)
           AND ($2 = '' OR create_date <= $2)
         "#,
         create_date_start, create_date_end,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(StatisticsResult { total, paid })
 }

 /// 统计支付信息数量
 ///
 /// 使用参数化查询防止 SQL 注入
 pub async fn payment_info_count(&self, query: &PaymentInfoQuery) -> AppResult<i64> {
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let payment_date_start = query.payment_date_start.as_deref().unwrap_or("" );
     let payment_date_end = query.payment_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().unwrap_or("" );
     let cashier = query.fzr.as_deref().unwrap_or("" );
     let source = query.source.as_deref().unwrap_or("" );
     let type_ = query.type_.as_deref().unwrap_or("" );
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };

     let row = sqlx::query_scalar!(
         r#"
         SELECT COUNT(id) FROM sf_payment_info
         WHERE id != ''
           AND ($1 = '' OR create_date > $1)
           AND ($2 = '' OR create_date < $2)
           AND ($3 = '' OR payment_time > $3)
           AND ($4 = '' OR payment_time < $4)
           AND ($5 = '' OR numbering LIKE $5)
           AND ($6 = '' OR cashier LIKE $6)
           AND ($7 = '' OR source = $7)
           AND ($8 = '' OR type = $8)
           AND ($9 = '' OR status = $9)
         "#,
         create_date_start, create_date_end, payment_date_start, payment_date_end,
         numbering, cashier, source, type_, status,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
 }

 /// 统计支付信息总额
 ///
 /// 使用参数化查询防止 SQL 注入
 pub async fn payment_info_total(&self, query: &PaymentStatisticsQuery) -> AppResult<i64> {
     let reality = query.reality.unwrap_or(false);
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let payment_date_start = query.payment_date_start.as_deref().unwrap_or("" );
     let payment_date_end = query.payment_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().unwrap_or("" );
     let cashier = query.fzr.as_deref().unwrap_or("" );
     let source = query.source.as_deref().unwrap_or("" );
     let type_ = query.type_.as_deref().unwrap_or("" );
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };

     // reality=true 时统计实缴金额, 否则统计应缴金额(与旧实现一致)
     let row = if reality {
         sqlx::query_scalar!(
             r#"
             SELECT COALESCE(SUM(reality_amount), 0)::bigint FROM sf_payment_info
             WHERE id != ''
               AND ($1 = '' OR create_date > $1)
               AND ($2 = '' OR create_date < $2)
               AND ($3 = '' OR payment_time > $3)
               AND ($4 = '' OR payment_time < $4)
               AND ($5 = '' OR numbering LIKE $5)
               AND ($6 = '' OR cashier LIKE $6)
               AND ($7 = '' OR source = $7)
               AND ($8 = '' OR type = $8)
               AND ($9 = '' OR status = $9)
             "#,
             create_date_start, create_date_end, payment_date_start, payment_date_end,
             numbering, cashier, source, type_, status,
         )
     } else {
         sqlx::query_scalar!(
             r#"
             SELECT COALESCE(SUM(payment_amount), 0)::bigint FROM sf_payment_info
             WHERE id != ''
               AND ($1 = '' OR create_date > $1)
               AND ($2 = '' OR create_date < $2)
               AND ($3 = '' OR payment_time > $3)
               AND ($4 = '' OR payment_time < $4)
               AND ($5 = '' OR numbering LIKE $5)
               AND ($6 = '' OR cashier LIKE $6)
               AND ($7 = '' OR source = $7)
               AND ($8 = '' OR type = $8)
               AND ($9 = '' OR status = $9)
             "#,
             create_date_start, create_date_end, payment_date_start, payment_date_end,
             numbering, cashier, source, type_, status,
         )
     }
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
 }

 /// 统计网络支付数量
 ///
 /// 使用参数化查询防止 SQL 注入
 pub async fn payment_web_count(&self, query: &PaymentWebQuery) -> AppResult<i64> {
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let payment_date_start = query.payment_date_start.as_deref().unwrap_or("" );
     let payment_date_end = query.payment_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().unwrap_or("" );
     let type_ = query.type_.as_deref().unwrap_or("" );
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };

     let row = sqlx::query_scalar!(
         r#"
         SELECT COUNT(id) FROM sf_payment_web
         WHERE id != ''
           AND ($1 = '' OR create_date > $1)
           AND ($2 = '' OR create_date < $2)
           AND ($3 = '' OR payment_time > $3)
           AND ($4 = '' OR payment_time < $4)
           AND ($5 = '' OR numbering LIKE $5)
           AND ($6 = '' OR type LIKE $6)
           AND ($7 = '' OR status = $7)
         "#,
         create_date_start, create_date_end, payment_date_start, payment_date_end,
         numbering, type_, status,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
 }

 /// 统计网络支付总额
 ///
 /// 使用参数化查询防止 SQL 注入
 pub async fn payment_web_total(&self, query: &PaymentWebStatisticsQuery) -> AppResult<i64> {
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let payment_date_start = query.payment_date_start.as_deref().unwrap_or("" );
     let payment_date_end = query.payment_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().unwrap_or("" );
     let type_ = query.type_.as_deref().unwrap_or("" );
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };

     // 使用 COALESCE 处理 NULL 值; ::bigint 将 numeric 聚合结果转为 i64
     let row = sqlx::query_scalar!(
         r#"
         SELECT COALESCE(SUM(payment_amount), 0)::bigint FROM sf_payment_web
         WHERE id != ''
           AND ($1 = '' OR create_date > $1)
           AND ($2 = '' OR create_date < $2)
           AND ($3 = '' OR payment_time > $3)
           AND ($4 = '' OR payment_time < $4)
           AND ($5 = '' OR numbering LIKE $5)
           AND ($6 = '' OR type LIKE $6)
           AND ($7 = '' OR status = $7)
         "#,
         create_date_start, create_date_end, payment_date_start, payment_date_end,
         numbering, type_, status,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
 }
}

// sanitize_sql_string / sanitize_sql_like_string 已删除 (2026-08-04 审计修复):
// 手工转义函数无任何生产调用(仅测试引用), 且转义不可靠——SQL 防注入统一走 sqlx 参数绑定。
