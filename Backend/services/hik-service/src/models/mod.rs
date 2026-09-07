//! 海康服务数据模型
//!
//! 定义海康 API 请求参数、优惠券请求、停车场信息和 Signo 停车场配置。

use serde::{Deserialize, Serialize};

/// 海康API请求参数
#[derive(Debug, Deserialize)]
pub struct HikRequest {
    pub method: String,
    // 司机相关
    pub phone: Option<i64>,
    pub driver_id: Option<String>,
    pub plate_no: Option<String>,
    pub plate_color: Option<String>,
    // 停车场相关
    pub page_no: Option<i32>,
    pub page_size: Option<i32>,
    pub park_name: Option<String>,
    pub park_code: Option<String>,
    // 订单相关
    pub request_type: Option<String>,
    pub unique_id: Option<String>,
    pub is_use_coupon: Option<String>,
    pub appeal_type: Option<String>,
    pub appeal_remark: Option<String>,
    pub appeal_in_time: Option<String>,
    pub appeal_out_time: Option<String>,
    pub appeal_source: Option<String>,
    pub arrears_ids: Option<String>,
}

/// 优惠券请求参数
#[derive(Debug, Deserialize)]
pub struct CouponRequest {
    pub phone: String,
    pub amount: i32,
    pub start: i64,
    pub end: String,
    pub r#type: String,
}

/// 停车场信息
/// 注意: 预留用于停车场管理功能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkInfo {
    pub park_code: Option<String>,
    pub park_name: Option<String>,
    pub address: Option<String>,
}

/// 订单信息
/// 注意: 预留用于订单管理功能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkOrder {
    pub unique_id: Option<String>,
    pub park_code: Option<String>,
    pub plate_no: Option<String>,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub amount: Option<i32>,
    pub status: Option<String>,
}

/// 停车场标签配置
#[derive(Debug, Clone)]
pub struct SignoTag {
    pub phone: String,
    pub key: String,
}

/// Signo停车场配置
#[derive(Debug, Clone, Default)]
pub struct SignoConfig {
    pub tags: std::collections::HashMap<String, SignoTag>,
}

impl SignoConfig {
    #[must_use]
    pub fn new() -> Self {
        let mut tags = std::collections::HashMap::new();
        tags.insert(
            "双桥花园地下停车场".to_string(),
            SignoTag {
                phone: "13800000001".to_string(),
                key: "F7A1C5BA".to_string(),
            },
        );
        tags.insert(
            "琵琶岛停车场".to_string(),
            SignoTag {
                phone: "13800000002".to_string(),
                key: "6B30AB0B".to_string(),
            },
        );
        tags.insert(
            "秀峰路中行停车场".to_string(),
            SignoTag {
                phone: "13800000003".to_string(),
                key: "7F3BE64A".to_string(),
            },
        );
        tags.insert(
            "民族村停车场".to_string(),
            SignoTag {
                phone: "13800000004".to_string(),
                key: "6E5F5EAE".to_string(),
            },
        );
        tags.insert(
            "文化广场停车场".to_string(),
            SignoTag {
                phone: "13800000005".to_string(),
                key: "E6D0F91B".to_string(),
            },
        );
        tags.insert(
            "四小停车场".to_string(),
            SignoTag {
                phone: "13800000006".to_string(),
                key: "9ED54E41".to_string(),
            },
        );
        tags.insert(
            "示例停车场A".to_string(),
            SignoTag {
                phone: "13800000007".to_string(),
                key: "EF6DFDFA".to_string(),
            },
        );
        Self { tags }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signo_config_new_has_seven_parking_lots() {
        let config = SignoConfig::new();
        assert_eq!(config.tags.len(), 7);
    }

    #[test]
    fn test_signo_config_contains_all_lots() {
        let config = SignoConfig::new();
        assert!(config.tags.contains_key("双桥花园地下停车场"));
        assert!(config.tags.contains_key("琵琶岛停车场"));
        assert!(config.tags.contains_key("秀峰路中行停车场"));
        assert!(config.tags.contains_key("民族村停车场"));
        assert!(config.tags.contains_key("文化广场停车场"));
        assert!(config.tags.contains_key("四小停车场"));
        assert!(config.tags.contains_key("示例停车场A"));
    }

    #[test]
    fn test_signo_config_tags_have_phone_and_key() {
        let config = SignoConfig::new();
        let tag = config.tags.get("双桥花园地下停车场").unwrap();
        assert_eq!(tag.phone, "13800000001");
        assert_eq!(tag.key, "F7A1C5BA");
    }

    #[test]
    fn test_hik_request_deserialize() {
        let json = r#"{"method": "test", "phone": 13800000001}"#;
        let req: HikRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "test");
        assert_eq!(req.phone, Some(13800000001));
    }

    #[test]
    fn test_coupon_request_deserialize() {
        let json = r#"{"phone": "13800000001", "amount": 500, "start": 1000, "end": "2026-01-01", "type": "discount"}"#;
        let req: CouponRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.phone, "13800000001");
        assert_eq!(req.amount, 500);
        assert_eq!(req.r#type, "discount");
    }

    #[test]
    fn test_hik_request_defaults() {
        let json = r#"{"method": "test"}"#;
        let req: HikRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "test");
        assert!(req.phone.is_none());
        assert!(req.plate_no.is_none());
    }

    #[test]
    fn test_signo_tag_default() {
        let config = SignoConfig::default();
        assert!(config.tags.is_empty());
    }
}
