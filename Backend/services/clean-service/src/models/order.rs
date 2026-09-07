// 订单数据模型
// Order data model

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 订单记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {pub id: String, pub no: Option<String>, pub imposing_no: Option<String>, pub imposing_name: Option<String>, pub fingerprint: Option<String>, pub zone: Option<String>, pub payer: Option<String>, pub project_code: Option<String>, pub project_name: Option<String>, pub unit: Option<String>, pub num: Option<String>, pub criteria: Option<String>, pub sum: Option<String>, pub sum_capital: Option<String>, pub remark: Option<String>, pub review: Option<String>, pub operator: Option<String>, pub collection_name: Option<String>, pub print: Option<i16>, pub project: Option<serde_json::Value>, pub invalid: Option<bool>, pub create_user: String, pub create_date: String, pub update_user: Option<String>, pub update_date: Option<String>, pub delete: Option<bool>, pub serial_number: Option<String>, pub numbering: Option<String>, pub cashier: Option<String>, pub payment_time: Option<String>, pub source: Option<String>, pub r#type: Option<String>, pub status: Option<String>}

/// 订单查询参数
#[derive(Debug, Deserialize, Default)]
pub struct OrderQuery {pub id: Option<String>, pub serial_number: Option<String>, pub numbering: Option<String>, pub cashier: Option<String>, pub payment_time: Option<String>, pub sort_by: Option<String>, pub descending: Option<bool>, pub max_page: Option<i64>, pub cur_page: Option<i64>}

/// 正式账单记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FormalBill {pub id: String, pub create_user: Option<String>, pub create_date: Option<String>, pub update_user: Option<String>, pub update_date: Option<String>, pub numbering: Option<String>, pub name: Option<String>, pub address: Option<String>, pub principal: Option<String>, pub telephone: Option<String>, pub bill_year: Option<String>, pub bill_month: Option<String>, pub amount: Option<String>, pub price: Option<String>, pub money: Option<i64>, pub status: Option<String>, pub fzr_id: Option<String>, pub sys_org_code: Option<String>, pub sys_company_code: Option<String>, pub charge_object_id: Option<String>}

/// 正式账单查询参数
#[derive(Debug, Deserialize, Default)]
pub struct FormalBillQuery {pub id: Option<String>, pub create_date_start: Option<String>, pub create_date_end: Option<String>, pub numbering: Option<String>, pub fzr: Option<String>, pub status: Option<String>, pub sort_by: Option<String>, pub descending: Option<bool>, pub max_page: Option<i64>, pub cur_page: Option<i64>}

/// 网络支付记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentWeb {pub id: String, pub create_user: String, pub create_date: String, pub update_user: Option<String>, pub update_date: Option<String>, pub orderform_code: Option<String>, pub payment_time: Option<String>, pub payment_amount: Option<i64>, pub payment_type: Option<String>, pub numbering: Option<String>, pub name: Option<String>, pub principal: Option<String>, pub telephone: Option<String>, pub address: Option<String>, pub receipt_status: Option<String>, pub status: String, pub openid: Option<String>, pub type_: Option<String>, pub receipt_number: Option<String>, pub serial_number: Option<String>}

/// 网络支付查询参数
#[derive(Debug, Deserialize, Default)]
pub struct PaymentWebQuery {pub create_date_start: Option<String>, pub create_date_end: Option<String>, pub payment_date_start: Option<String>, pub payment_date_end: Option<String>, pub numbering: Option<String>, pub type_: Option<String>, pub status: Option<String>, pub sort_by: Option<String>, pub descending: Option<bool>, pub max_page: Option<i64>, pub cur_page: Option<i64>}

/// 支付信息查询参数
#[derive(Debug, Deserialize, Default)]
pub struct PaymentInfoQuery {pub create_date_start: Option<String>, pub create_date_end: Option<String>, pub payment_date_start: Option<String>, pub payment_date_end: Option<String>, pub numbering: Option<String>, pub fzr: Option<String>, pub source: Option<String>, pub type_: Option<String>, pub status: Option<String>, pub sort_by: Option<String>, pub descending: Option<bool>, pub max_page: Option<i64>, pub cur_page: Option<i64>}
