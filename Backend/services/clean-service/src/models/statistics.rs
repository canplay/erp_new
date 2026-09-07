// 统计数据模型
// Statistics data model

use serde::{Deserialize, Serialize};

/// 统计查询参数
#[derive(Debug, Deserialize, Default)]
pub struct StatisticsQuery {pub create_date_start: Option<String>, pub create_date_end: Option<String>, pub numbering: Option<String>, pub fzr: Option<String>, pub status: Option<String>}

/// 支付统计查询参数
#[derive(Debug, Deserialize, Default)]
pub struct PaymentStatisticsQuery {pub create_date_start: Option<String>, pub create_date_end: Option<String>, pub payment_date_start: Option<String>, pub payment_date_end: Option<String>, pub numbering: Option<String>, pub fzr: Option<String>, pub source: Option<String>, pub type_: Option<String>, pub status: Option<String>, pub reality: Option<bool>}

/// 网络支付统计查询参数
#[derive(Debug, Deserialize, Default)]
pub struct PaymentWebStatisticsQuery {pub create_date_start: Option<String>, pub create_date_end: Option<String>, pub payment_date_start: Option<String>, pub payment_date_end: Option<String>, pub numbering: Option<String>, pub type_: Option<String>, pub status: Option<String>}

/// 统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsResult {pub total: i64, pub paid: i64}
