//! 订单数据模型

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, FromRow, Default)]
pub struct OrderInfo {
    pub code: Option<String>,
    pub status: i64,
    pub provide: String,
    pub speed: f64,
    pub gps: Option<Value>,
    /// 车辆类型（预留字段，当前所有写入路径均硬编码为 0，暂不使用）。
    /// 用途：未来可能区分共享单车/电单车/摩托车等车型。
    /// 状态：已文档化，等待业务需求。
    #[sqlx(rename = "type")]
    pub r#type: i64,
    pub time: Option<Value>,
    pub create_date: Option<NaiveDateTime>,
    pub update_date: Option<NaiveDateTime>,
    pub delete: Option<bool>,
    pub alert: Option<String>,
    pub remark: Option<String>,
    /// 订单哈希/签名（预留字段，当前所有写入路径均硬编码为空串，暂不使用）。
    /// 用途：用于支付回调签名验证或防篡改。
    /// 状态：已文档化，等待业务需求。
    pub hash: String,
    pub payable: i64,
    pub pay: i64,
    pub refund: f64,
    pub coupon: f64,
    pub order: Option<String>,
    pub pay_type: i64,
    pub pay_time: Option<NaiveDateTime>,
    pub pay_status: i64,
    #[sqlx(skip)]
    #[serde(skip_serializing)]
    pub paytype: i64,
    #[sqlx(skip)]
    #[serde(skip_serializing)]
    pub paytime: Option<NaiveDateTime>,
    pub gps_type: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_info_serialization() {
        let order = OrderInfo {
            code: Some("ORD001".to_string()),
            status: 1,
            provide: "provider_a".to_string(),
            speed: 15.0,
            gps: Some(serde_json::json!({"lat": 23.5})),
            r#type: 1,
            time: None,
            create_date: None,
            update_date: None,
            delete: Some(false),
            alert: None,
            remark: None,
            hash: "abc123".to_string(),
            payable: 100,
            pay: 100,
            refund: 0.0,
            coupon: 0.0,
            order: Some("PAY001".to_string()),
            pay_type: 1,
            pay_time: None,
            pay_status: 1,
            paytype: 1,
            paytime: None,
            gps_type: Some(1),
        };

        let json = serde_json::to_string(&order).expect("test assertion");
        assert!(json.contains("\"code\":\"ORD001\""));
        assert!(json.contains("\"payable\":100"));
        assert!(json.contains("\"hash\":\"abc123\""));
    }

    #[test]
    fn test_order_info_deserialization() {
        let json = r#"{
            "code": "ORD002",
            "status": 0,
            "provide": "",
            "speed": 0.0,
            "type": 0,
            "hash": "",
            "payable": 0,
            "pay": 0,
            "refund": 0.0,
            "coupon": 0.0,
            "pay_type": 0,
            "pay_status": 0,
            "paytype": 0
        }"#;
        let order: OrderInfo = serde_json::from_str(json).expect("test assertion");
        assert_eq!(order.code, Some("ORD002".to_string()));
        assert_eq!(order.status, 0);
    }

    #[test]
    fn test_order_info_default() {
        let order = OrderInfo::default();
        assert!(order.code.is_none());
        assert_eq!(order.status, 0);
        assert_eq!(order.hash, "");
        assert_eq!(order.payable, 0);
    }
}
