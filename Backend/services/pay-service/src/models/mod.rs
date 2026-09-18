//! 支付服务数据模型
//!
//! 定义支付请求、回调、订单等核心数据结构。
//! 注意：预留部分结构用于未来功能扩展。

use serde::{Deserialize, Serialize};

/// 支付订单查询参数
#[derive(Debug, Deserialize)]
pub struct PayQuery {
    pub status: Option<String>,
    #[serde(rename = "type" )]
    pub pay_type: Option<String>,
    pub remark: Option<String>,
    pub sort_by: Option<String>,
    pub descending: Option<bool>,
    pub max_page: Option<i64>,
    pub cur_page: Option<i64>,
}

impl Default for PayQuery {
    fn default() -> Self {
        Self {
            status: None,
            pay_type: None,
            remark: None,
            sort_by: None,
            descending: Some(true),
            max_page: Some(10),
            cur_page: Some(0),
        }
    }
}

/// 支付订单信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayOrder {
    pub id: String,
    pub order: String,
    pub status: String,
    #[serde(rename = "type" )]
    pub pay_type: String,
    pub order_pay: Option<serde_json::Value>,
    pub amount: i32,
    pub remark: String,
    pub create_service: String,
    pub create_params: Option<serde_json::Value>,
    pub create_date: String,
    pub update_date: String,
}

/// 支付订单数量响应（预留）
#[derive(Debug, Serialize)]
pub struct PayCountResponse {
    pub data: i64,
    pub message: String,
    pub status: i32,
}

/// 支付订单列表响应（预留）
#[derive(Debug, Serialize)]
pub struct PayListResponse {
    pub data: Vec<PayOrder>,
    pub message: String,
    pub status: i32,
}

/// 支付订单详情响应（预留）
#[derive(Debug, Serialize)]
pub struct PayDetailResponse {
    pub data: PayOrder,
    pub message: String,
    pub status: i32,
}

/// 创建支付订单参数
#[derive(Debug, Deserialize)]
pub struct PayCreateParams {
    pub order: String,
    pub status: Option<String>,
    #[serde(rename = "type" )]
    pub pay_type: Option<String>,
    pub order_pay: Option<serde_json::Value>,
    pub amount: Option<i32>,
    pub remark: Option<String>,
    pub create_params: Option<serde_json::Value>,
    pub create_date: Option<String>,
}
