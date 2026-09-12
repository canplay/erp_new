//! 车辆数据模型

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, FromRow, Default)]
pub struct CarInfo {
    pub code: String,
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
    pub gps_type: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_info_serialization_roundtrip() {
        let car = CarInfo {
            code: "CAR001".to_string(),
            status: 1,
            provide: "provider_a".to_string(),
            speed: 25.5,
            gps: Some(serde_json::json!({"lat": 23.5, "lng": 120.3})),
            r#type: 2,
            time: None,
            create_date: None,
            update_date: None,
            delete: Some(false),
            alert: None,
            remark: Some("测试车辆".to_string()),
            gps_type: 1,
        };

        let json = serde_json::to_string(&car).expect("test assertion");
        let deserialized: CarInfo = serde_json::from_str(&json).expect("deserialization should not fail");
        assert_eq!(deserialized.code, "CAR001");
        assert_eq!(deserialized.status, 1);
        assert_eq!(deserialized.provide, "provider_a");
        assert!((deserialized.speed - 25.5).abs() < f64::EPSILON);
        assert_eq!(deserialized.r#type, 2);
        assert_eq!(deserialized.gps_type, 1);
        assert_eq!(deserialized.remark, Some("测试车辆".to_string()));
        assert_eq!(deserialized.delete, Some(false));
    }

    #[test]
    fn test_car_info_minimal_deserialization() {
        let json = r#"{"code": "CAR002", "status": 0, "provide": "", "speed": 0.0, "type": 0, "gps_type": 0}"#;
        let car: CarInfo = serde_json::from_str(json).expect("test assertion");
        assert_eq!(car.code, "CAR002");
        assert!(car.gps.is_none());
        assert!(car.remark.is_none());
    }

    #[test]
    fn test_car_info_json_output_contains_type() {
        let car = CarInfo::default();
        let json = serde_json::to_string(&car).expect("test assertion");
        assert!(json.contains("\"type\""));
    }
}
