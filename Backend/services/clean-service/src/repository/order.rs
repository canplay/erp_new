// 订单仓储
// Order repository

use sqlx::PgPool;

use common::AppError;
use common::AppResult;
use crate::models::{FormalBill, FormalBillQuery, Order, OrderQuery, PaymentWeb, PaymentWebQuery};

/// 订单仓储
pub struct OrderRepository {pool: PgPool}

impl OrderRepository {
  /// 创建订单仓储实例
  /// 审计修复 (H8-架构评审): 注入共享 PgPool, 不再每仓储自建连接池(原 4 池 x 10 连接)
  #[must_use]
  pub fn new(pool: PgPool) -> Self {
 Self { pool}
 }

 /// 统计订单数量
 pub async fn count(&self, query: &OrderQuery) -> AppResult<i64> {
     // 安全加固: 全部改为参数绑定 + 编译期校验 (修复: 原实现直接拼接用户输入, SQL 注入)
     let id = query.id.as_deref().unwrap_or("" );
     let serial_number = query.serial_number.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().unwrap_or("" );
     let cashier = query.cashier.as_deref().unwrap_or("" );
     let payment_time = query.payment_time.as_deref().unwrap_or("" );

     let row = sqlx::query_scalar!(
         r#"
         SELECT COUNT(id) FROM sf_payment_info
         WHERE ($1 = '' OR id = $1)
           AND ($2 = '' OR serial_number = $2)
           AND ($3 = '' OR numbering = $3)
           AND ($4 = '' OR cashier = $4)
           AND ($5 = '' OR payment_time = $5)
         "#,
         id, serial_number, numbering, cashier, payment_time,
     )
     .fetch_one(&self.pool)
     .await
     .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     .unwrap_or(0);

     Ok(row)
     }

     /// 查询订单列表
 pub async fn list(&self, query: &OrderQuery) -> AppResult<Vec<Order>> {
     // 安全加固: 全部改为参数绑定; ORDER BY 列名白名单(经 CASE 表达式参数化)
     let id = query.id.as_deref().unwrap_or(" ");
     let serial_number = query.serial_number.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().unwrap_or("" );
     let cashier = query.cashier.as_deref().unwrap_or("" );
     let payment_time = query.payment_time.as_deref().unwrap_or("" );
     let sort_by = query.sort_by.as_deref().unwrap_or("" );
     let descending = query.descending.unwrap_or(false);
     let limit = query.max_page.unwrap_or(-1);
     let offset = query.cur_page.unwrap_or(0);

     let orders = if descending {
         sqlx::query_as!(
             Order,
             r#"
             SELECT id, no, imposing_no, imposing_name, fingerprint, zone, payer,
                    project_code, project_name, unit, num, criteria, sum, sum_capital,
                    remark, review, operator, collection_name, print, project, invalid,
                    create_user, create_date, update_user, update_date, delete,
                    serial_number, numbering, cashier, payment_time, source, type, status
             FROM sf_payment_info
             WHERE ($1 = '' OR id = $1)
               AND ($2 = '' OR serial_number = $2)
               AND ($3 = '' OR numbering = $3)
               AND ($4 = '' OR cashier = $4)
               AND ($5 = '' OR payment_time = $5)
             ORDER BY CASE WHEN $6 = 'id' THEN id
                           WHEN $6 = 'serial_number' THEN serial_number
                           WHEN $6 = 'numbering' THEN numbering
                           WHEN $6 = 'cashier' THEN cashier
                           WHEN $6 = 'payment_time' THEN payment_time
                           ELSE create_date END DESC
             LIMIT CASE WHEN $7 = -1 THEN NULL ELSE $7 END OFFSET $8
             "#,
             id, serial_number, numbering, cashier, payment_time, sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     } else {
         sqlx::query_as!(
             Order,
             r#"
             SELECT id, no, imposing_no, imposing_name, fingerprint, zone, payer,
                    project_code, project_name, unit, num, criteria, sum, sum_capital,
                    remark, review, operator, collection_name, print, project, invalid,
                    create_user, create_date, update_user, update_date, delete,
                    serial_number, numbering, cashier, payment_time, source, type, status
             FROM sf_payment_info
             WHERE ($1 = '' OR id = $1)
               AND ($2 = '' OR serial_number = $2)
               AND ($3 = '' OR numbering = $3)
               AND ($4 = '' OR cashier = $4)
               AND ($5 = '' OR payment_time = $5)
             ORDER BY CASE WHEN $6 = 'id' THEN id
                           WHEN $6 = 'serial_number' THEN serial_number
                           WHEN $6 = 'numbering' THEN numbering
                           WHEN $6 = 'cashier' THEN cashier
                           WHEN $6 = 'payment_time' THEN payment_time
                           ELSE create_date END ASC
             LIMIT CASE WHEN $7 = -1 THEN NULL ELSE $7 END OFFSET $8
             "#,
             id, serial_number, numbering, cashier, payment_time, sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     };

     Ok(orders)
 }

 /// 统计正式账单数量
 pub async fn count_formal_bill(&self, query: &FormalBillQuery) -> AppResult<i64> {
     // 安全加固: 全部改为参数绑定(修复: 原实现直接拼接用户输入, SQL 注入)
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
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

     /// 查询正式账单列表
 pub async fn list_formal_bill(&self, query: &FormalBillQuery) -> AppResult<Vec<FormalBill>> {
     // 安全加固: 全部改为参数绑定; ORDER BY 列名白名单(经 CASE 表达式参数化)
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().map(|n| format!("%{n}%" )).unwrap_or_default();
     let fzr = query.fzr.as_deref().map(|f| format!("%{f}%" )).unwrap_or_default();
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };
     let sort_by = query.sort_by.as_deref().unwrap_or("" );
     let descending = query.descending.unwrap_or(false);
     let limit = query.max_page.unwrap_or(-1);
     let offset = query.cur_page.unwrap_or(0);

     let bills = if descending {
         sqlx::query_as!(
             FormalBill,
             r#"
             SELECT id, create_user, create_date, update_user, update_date, numbering,
                    name, address, principal, telephone, bill_year, bill_month,
                    amount, price, money, status, fzr_id, sys_org_code, sys_company_code,
                    charge_object_id
             FROM sf_formal_bill
             WHERE id != ''
               AND ($1 = '' OR create_date > $1)
               AND ($2 = '' OR create_date < $2)
               AND ($3 = '' OR numbering LIKE $3)
               AND ($4 = '' OR fzr_id LIKE $4)
               AND ($5 = '' OR status = $5)
             ORDER BY CASE WHEN $6 = 'id' THEN id
                           WHEN $6 = 'numbering' THEN numbering
                           WHEN $6 = 'create_date' THEN create_date
                           WHEN $6 = 'fzr_id' THEN fzr_id
                           ELSE create_date END DESC
             LIMIT CASE WHEN $7 = -1 THEN NULL ELSE $7 END OFFSET $8
             "#,
             create_date_start, create_date_end, numbering, fzr, status, sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     } else {
         sqlx::query_as!(
             FormalBill,
             r#"
             SELECT id, create_user, create_date, update_user, update_date, numbering,
                    name, address, principal, telephone, bill_year, bill_month,
                    amount, price, money, status, fzr_id, sys_org_code, sys_company_code,
                    charge_object_id
             FROM sf_formal_bill
             WHERE id != ''
               AND ($1 = '' OR create_date > $1)
               AND ($2 = '' OR create_date < $2)
               AND ($3 = '' OR numbering LIKE $3)
               AND ($4 = '' OR fzr_id LIKE $4)
               AND ($5 = '' OR status = $5)
             ORDER BY CASE WHEN $6 = 'id' THEN id
                           WHEN $6 = 'numbering' THEN numbering
                           WHEN $6 = 'create_date' THEN create_date
                           WHEN $6 = 'fzr_id' THEN fzr_id
                           ELSE create_date END ASC
             LIMIT CASE WHEN $7 = -1 THEN NULL ELSE $7 END OFFSET $8
             "#,
             create_date_start, create_date_end, numbering, fzr, status, sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     };

     Ok(bills)
 }

 /// 统计网络支付数量
 pub async fn count_payment_web(&self, query: &PaymentWebQuery) -> AppResult<i64> {
     // 安全加固: 全部改为参数绑定(修复: 原实现直接拼接用户输入, SQL 注入)
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let payment_date_start = query.payment_date_start.as_deref().unwrap_or("" );
     let payment_date_end = query.payment_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().map(|n| format!("%{n}%" )).unwrap_or_default();
     let type_ = query.type_.as_deref().map(|t| format!("%{t}%" )).unwrap_or_default();
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

     /// 查询网络支付列表
 pub async fn list_payment_web(&self, query: &PaymentWebQuery) -> AppResult<Vec<PaymentWeb>> {
     // 安全加固: 全部改为参数绑定; ORDER BY 列名白名单(经 CASE 表达式参数化)
     let create_date_start = query.create_date_start.as_deref().unwrap_or(" ");
     let create_date_end = query.create_date_end.as_deref().unwrap_or("" );
     let payment_date_start = query.payment_date_start.as_deref().unwrap_or("" );
     let payment_date_end = query.payment_date_end.as_deref().unwrap_or("" );
     let numbering = query.numbering.as_deref().map(|n| format!("%{n}%" )).unwrap_or_default();
     let type_ = query.type_.as_deref().map(|t| format!("%{t}%" )).unwrap_or_default();
     let status = match query.status.as_deref() {
         Some("已支付" ) => "Y" ,
         Some("未支付" ) => "N" ,
         _ => "" ,
     };
     let sort_by = query.sort_by.as_deref().unwrap_or("" );
     let descending = query.descending.unwrap_or(false);
     let limit = query.max_page.unwrap_or(-1);
     let offset = query.cur_page.unwrap_or(0);

     let payments = if descending {
         sqlx::query_as!(
             PaymentWeb,
             r#"
             SELECT id, create_user, create_date, update_user, update_date, orderform_code,
                    payment_time, payment_amount, payment_type, numbering, name, principal,
                    telephone, address, receipt_status, status, openid, type AS "type_" ,
                    receipt_number, serial_number
             FROM sf_payment_web
             WHERE id != ''
               AND ($1 = '' OR create_date > $1)
               AND ($2 = '' OR create_date < $2)
               AND ($3 = '' OR payment_time > $3)
               AND ($4 = '' OR payment_time < $4)
               AND ($5 = '' OR numbering LIKE $5)
               AND ($6 = '' OR type LIKE $6)
               AND ($7 = '' OR status = $7)
             ORDER BY CASE WHEN $8 = 'id' THEN id
                           WHEN $8 = 'numbering' THEN numbering
                           WHEN $8 = 'create_date' THEN create_date
                           WHEN $8 = 'payment_time' THEN payment_time
                           WHEN $8 = 'type' THEN type
                           ELSE create_date END DESC
             LIMIT CASE WHEN $9 = -1 THEN NULL ELSE $9 END OFFSET $10
             "#,
             create_date_start, create_date_end, payment_date_start, payment_date_end,
             numbering, type_, status, sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     } else {
         sqlx::query_as!(
             PaymentWeb,
             r#"
             SELECT id, create_user, create_date, update_user, update_date, orderform_code,
                    payment_time, payment_amount, payment_type, numbering, name, principal,
                    telephone, address, receipt_status, status, openid, type AS "type_" ,
                    receipt_number, serial_number
             FROM sf_payment_web
             WHERE id != ''
               AND ($1 = '' OR create_date > $1)
               AND ($2 = '' OR create_date < $2)
               AND ($3 = '' OR payment_time > $3)
               AND ($4 = '' OR payment_time < $4)
               AND ($5 = '' OR numbering LIKE $5)
               AND ($6 = '' OR type LIKE $6)
               AND ($7 = '' OR status = $7)
             ORDER BY CASE WHEN $8 = 'id' THEN id
                           WHEN $8 = 'numbering' THEN numbering
                           WHEN $8 = 'create_date' THEN create_date
                           WHEN $8 = 'payment_time' THEN payment_time
                           WHEN $8 = 'type' THEN type
                           ELSE create_date END ASC
             LIMIT CASE WHEN $9 = -1 THEN NULL ELSE $9 END OFFSET $10
             "#,
             create_date_start, create_date_end, payment_date_start, payment_date_end,
             numbering, type_, status, sort_by, limit as i32, offset as i32,
         )
         .fetch_all(&self.pool)
         .await
         .map_err(|e: sqlx::Error| AppError::DatabaseError(e.to_string()))?
     };

     Ok(payments)
     }
     }
