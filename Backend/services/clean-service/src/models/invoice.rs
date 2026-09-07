// 发票数据模型
// Invoice data model

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 发票记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {pub id: String, pub no: String, pub imposing_no: Option<i64>, pub imposing_name: Option<String>, pub fingerprint: String, pub zone: Option<String>, pub payer: Option<String>, pub sum: Option<String>, pub sum_capital: Option<String>, pub remark: Option<String>, pub review: Option<String>, pub operator: Option<String>, pub collection_name: Option<String>, pub print: Option<i32>, pub project: Option<serde_json::Value>, pub invalid: bool, pub create_user: String, pub create_date: String, pub update_user: Option<String>, pub update_date: Option<String>, pub delete: bool}

/// 发票查询参数
#[derive(Debug, Deserialize, Default)]
pub struct InvoiceQuery {pub id: Option<String>, pub no: Option<String>, pub imposing_no: Option<i64>, pub imposing_name: Option<String>, pub collection_name: Option<String>, pub fingerprint: Option<String>, pub create_date: Option<String>, pub zone: Option<String>, pub sum: Option<String>, pub sum_capital: Option<String>, pub remark: Option<String>, pub review: Option<String>, pub operator: Option<String>, pub create_user: Option<String>, pub sort_by: Option<String>, pub descending: Option<bool>, pub max_page: Option<i64>, pub cur_page: Option<i64>}

/// 发票创建/更新参数
#[derive(Debug, Deserialize)]
pub struct InvoiceCreateParam {pub id: Option<String>, pub no: String, pub imposing_no: Option<i64>, pub imposing_name: Option<String>, pub zone: Option<String>, pub payer: Option<String>, pub sum: Option<String>, pub sum_capital: Option<String>, pub remark: Option<String>, pub review: Option<String>, pub operator: Option<String>, pub collection_name: Option<String>, pub print: Option<i32>, pub project: Option<serde_json::Value>, pub invalid: Option<bool>, pub delete: Option<bool>, pub create_user: Option<String>}
