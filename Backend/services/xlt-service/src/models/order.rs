//! 订单/计费相关数据模型
//!
//! 停车费计费规则、计费请求与响应。

use serde::{Deserialize, Serialize};

/// 计费规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRule {
    pub park_code: String,
    pub free_minutes: i32,
    pub first_hour_fee: i64,
    pub hourly_fee: i64,
    pub daily_cap: i64,
    pub night_fee: i64,
    pub night_start: String,
    pub night_end: String,
}

#[derive(Debug, Deserialize)]
pub struct BillingRequest {
    pub plate_no: String,
    pub park_code: String,
    pub entry_time: String,
    pub exit_time: String,
}

#[derive(Debug, Serialize)]
pub struct BillingResult {
    pub plate_no: String,
    pub park_code: String,
    pub entry_time: String,
    pub exit_time: String,
    pub duration_minutes: i64,
    pub total_amount: i64,
    pub rule: String,
}
