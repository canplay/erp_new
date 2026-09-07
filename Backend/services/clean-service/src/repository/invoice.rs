// 发票仓储
// Invoice repository

use sqlx::PgPool;

use crate::error::{CleanServiceError};
use crate::error::CleanResult;
use crate::models::{Invoice, InvoiceCreateParam, InvoiceQuery};

/// 发票仓储
pub struct InvoiceRepository {pool: PgPool}

impl InvoiceRepository {/// 创建发票仓储实例
 /// 审计修复 (H8-架构评审): 注入共享 PgPool, 不再每仓储自建连接池(原 4 池 x 10 连接)
 #[must_use]
 pub fn new(pool: PgPool) -> Self {
 Self { pool}
 }

 /// 统计发票数量
 pub async fn count(&self, query: &InvoiceQuery) -> CleanResult<i64> {
     // 安全加固: 全部改为参数绑定 + 编译期校验(修复: 原实现直接拼接用户输入, SQL 注入)
     let id = query.id.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let no = query.no.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let imposing_no = query.imposing_no.filter(|&v| v != 0);
     let imposing_name = query.imposing_name.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let collection_name = query.collection_name.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let fingerprint = query.fingerprint.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let create_date = query.create_date.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let zone = query.zone.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();

     let row = sqlx::query_scalar!(
         r#"
         SELECT COUNT(id) FROM clean_invoice
         WHERE delete = FALSE
           AND ($1 = '' OR id LIKE $1)
           AND ($2 = '' OR no LIKE $2)
           AND ($3::bigint IS NULL OR imposing_no = $3)
           AND ($4 = '' OR imposing_name LIKE $4)
           AND ($5 = '' OR collection_name LIKE $5)
           AND ($6 = '' OR fingerprint LIKE $6)
           AND ($7 = '' OR create_date LIKE $7)
           AND ($8 = '' OR zone LIKE $8)
         "#,
         id, no, imposing_no, imposing_name, collection_name, fingerprint, create_date, zone,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| CleanServiceError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
 }

 /// 查询发票列表
 pub async fn list(&self, query: &InvoiceQuery) -> CleanResult<Vec<Invoice>> {
     // 安全加固: 全部改为参数绑定; ORDER BY 列名白名单(经 CASE 表达式参数化)
     let id = query.id.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let no = query.no.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let imposing_no = query.imposing_no.filter(|&v| v != 0);
     let imposing_name = query.imposing_name.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let collection_name = query.collection_name.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let fingerprint = query.fingerprint.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let create_date = query.create_date.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let zone = query.zone.as_deref().map(|s| format!("%{s}%")).unwrap_or_default();
     let sort_by = query.sort_by.as_deref().unwrap_or("");
     let descending = query.descending.unwrap_or(false);
     let limit = query.max_page.unwrap_or(-1);
     let offset = query.cur_page.unwrap_or(0);

     let invoices = if descending {
         sqlx::query_as!(
             Invoice,
             r#"
             SELECT id, COALESCE(no, '') AS "no!",
                    imposing_no, imposing_name,
                    COALESCE(fingerprint, '') AS "fingerprint!",
                    zone, payer, sum, sum_capital, remark, review, operator, collection_name,
                    print, project, COALESCE(invalid, false) AS "invalid!",
                    COALESCE(create_user, '') AS "create_user!",
                     COALESCE(create_date, '') AS "create_date!",
                     update_user, update_date,
                    COALESCE(delete, false) AS "delete!"
             FROM clean_invoice
             WHERE delete = FALSE
               AND ($1 = '' OR id LIKE $1)
               AND ($2 = '' OR no LIKE $2)
               AND ($3::bigint IS NULL OR imposing_no = $3)
               AND ($4 = '' OR imposing_name LIKE $4)
               AND ($5 = '' OR collection_name LIKE $5)
               AND ($6 = '' OR fingerprint LIKE $6)
               AND ($7 = '' OR create_date LIKE $7)
               AND ($8 = '' OR zone LIKE $8)
             ORDER BY CASE WHEN $9 = 'id' THEN id
                           WHEN $9 = 'no' THEN no
                           WHEN $9 = 'imposing_no' THEN imposing_no::text
                           WHEN $9 = 'imposing_name' THEN imposing_name
                           WHEN $9 = 'collection_name' THEN collection_name
                           WHEN $9 = 'fingerprint' THEN fingerprint
                           WHEN $9 = 'create_date' THEN create_date
                           WHEN $9 = 'zone' THEN zone
                           ELSE create_date END DESC
             LIMIT CASE WHEN $10 = -1 THEN NULL ELSE $10 END OFFSET $11
             "#,
             id, no, imposing_no, imposing_name, collection_name, fingerprint, create_date, zone,
             sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| CleanServiceError::DatabaseError(e.to_string()))?
     } else {
         sqlx::query_as!(
             Invoice,
             r#"
             SELECT id, COALESCE(no, '') AS "no!",
                    imposing_no, imposing_name,
                    COALESCE(fingerprint, '') AS "fingerprint!",
                    zone, payer, sum, sum_capital, remark, review, operator, collection_name,
                    print, project, COALESCE(invalid, false) AS "invalid!",
                    COALESCE(create_user, '') AS "create_user!",
                     COALESCE(create_date, '') AS "create_date!",
                     update_user, update_date,
                    COALESCE(delete, false) AS "delete!"
             FROM clean_invoice
             WHERE delete = FALSE
               AND ($1 = '' OR id LIKE $1)
               AND ($2 = '' OR no LIKE $2)
               AND ($3::bigint IS NULL OR imposing_no = $3)
               AND ($4 = '' OR imposing_name LIKE $4)
               AND ($5 = '' OR collection_name LIKE $5)
               AND ($6 = '' OR fingerprint LIKE $6)
               AND ($7 = '' OR create_date LIKE $7)
               AND ($8 = '' OR zone LIKE $8)
             ORDER BY CASE WHEN $9 = 'id' THEN id
                           WHEN $9 = 'no' THEN no
                           WHEN $9 = 'imposing_no' THEN imposing_no::text
                           WHEN $9 = 'imposing_name' THEN imposing_name
                           WHEN $9 = 'collection_name' THEN collection_name
                           WHEN $9 = 'fingerprint' THEN fingerprint
                           WHEN $9 = 'create_date' THEN create_date
                           WHEN $9 = 'zone' THEN zone
                           ELSE create_date END ASC
             LIMIT CASE WHEN $10 = -1 THEN NULL ELSE $10 END OFFSET $11
             "#,
             id, no, imposing_no, imposing_name, collection_name, fingerprint, create_date, zone,
             sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| CleanServiceError::DatabaseError(e.to_string()))?
     };

     Ok(invoices)
 }

 /// 创建或更新发票
 pub async fn upsert(&self, param: &InvoiceCreateParam) -> CleanResult<()> {
     let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

     if let Some(ref id) = param.id {
         // 更新
         // 审计修复 (B1-架构评审): 修正绑定错位——原 $18(create_date) 绑的是 create_user,
         // 导致更新时创建时间被覆盖为用户名字符串; 原 fingerprint 每次更新重新生成, 破坏指纹稳定性。
         // 修复: UPDATE 不再写 create_date(创建时间创建后不变) 与 fingerprint(仅创建时生成一次),
         // 两者保持数据库原值。
         sqlx::query!(
             r#"UPDATE clean_invoice SET
                no = $1, imposing_no = $2, imposing_name = $3, zone = $4, payer = $5,
                sum = $6, sum_capital = $7, remark = $8, review = $9, operator = $10,
                collection_name = $11, print = $12, project = $13, invalid = $14,
                update_user = $15, update_date = $16, delete = $17 WHERE id = $18"#,
             &param.no,
             param.imposing_no,
             param.imposing_name.as_deref(),
             param.zone.as_deref(),
             param.payer.as_deref(),
             param.sum.as_deref(),
             param.sum_capital.as_deref(),
             param.remark.as_deref(),
             param.review.as_deref(),
             param.operator.as_deref(),
             param.collection_name.as_deref(),
             param.print,
             param.project.as_ref(),
             param.invalid.unwrap_or(false),
             param.create_user.as_deref(),
             &now,
             param.delete.unwrap_or(false),
             id,
         )
         .execute(&self.pool)
         .await
         .map_err(|e: sqlx::Error| CleanServiceError::DatabaseError(e.to_string()))?;
     } else {
         // 新增: fingerprint 仅在创建时生成一次
         let id = uuid::Uuid::new_v4().to_string();
         let fingerprint = uuid::Uuid::new_v4().to_string();
         // 审计修复 (B1): 表结构位于遗留库(schema.sql 未收录), 按模型字段顺序补显式列名,
         // 列顺序与遗留表一致(见 DB 迁移脚本), 保持位置映射。
         sqlx::query!(
             r#"INSERT INTO clean_invoice (
                id, no, imposing_no, imposing_name, fingerprint, zone, payer,
                sum, sum_capital, remark, review, operator, create_user, create_date,
                update_user, update_date, collection_name, print, project, invalid, delete
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                       $15, $16, $17, $18, $19, false, false)"#,
             &id,
             &param.no,
             param.imposing_no,
             param.imposing_name.as_deref(),
             &fingerprint,
             param.zone.as_deref(),
             param.payer.as_deref(),
             param.sum.as_deref(),
             param.sum_capital.as_deref(),
             param.remark.as_deref(),
             param.review.as_deref(),
             param.operator.as_deref(),
             param.create_user.as_deref(),
             &now,
             param.create_user.as_deref(),
             &now,
             param.collection_name.as_deref(),
             param.print,
             param.project.as_ref(),
         )
         .execute(&self.pool)
         .await
         .map_err(|e: sqlx::Error| CleanServiceError::DatabaseError(e.to_string()))?;
     }

     Ok(())
 }
}
