//! 仓储数据模型

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, FromRow, Default)]
pub struct StorageInfo {
    pub code: String,
    pub status: i64,
    pub provide: String,
    pub gps: Option<Value>,
    /// 存储类型（预留字段，当前所有写入路径均硬编码为 0，暂不使用）
    #[sqlx(rename = "type")]
    pub r#type: i64,
    pub sum: i64,
    pub cur: i64,
    pub create_date: Option<NaiveDateTime>,
    pub update_date: Option<NaiveDateTime>,
    pub delete: Option<bool>,
    pub alert: Option<String>,
    pub remark: Option<String>,
    pub points: Option<String>,
    pub gps_type: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_info_serialization_roundtrip() {
        let storage = StorageInfo {
            code: "ST001".to_string(),
            status: 1,
            provide: "provider_a".to_string(),
            gps: Some(serde_json::json!({"lat": 23.5, "lng": 120.3})),
            r#type: 1,
            sum: 100,
            cur: 45,
            create_date: None,
            update_date: None,
            delete: Some(false),
            alert: None,
            remark: Some("测试".to_string()),
            points: Some("POLYGON(...)".to_string()),
            gps_type: 1,
        };

        let json = serde_json::to_string(&storage).expect("test assertion");
        let deserialized: StorageInfo = serde_json::from_str(&json).expect("test assertion");
        assert_eq!(deserialized.code, "ST001");
        assert_eq!(deserialized.sum, 100);
        assert_eq!(deserialized.cur, 45);
        assert_eq!(deserialized.points, Some("POLYGON(...)".to_string()));
        assert_eq!(deserialized.gps_type, 1);
    }

    #[test]
    fn test_storage_info_minimal_deserialization() {
        let json = r#"{"code": "ST002", "status": 0, "provide": "", "speed": 0.0, "type": 0, "sum": 0, "cur": 0, "gps_type": 0}"#;
        let storage: StorageInfo = serde_json::from_str(json).expect("test assertion");
        assert_eq!(storage.code, "ST002");
        assert_eq!(storage.sum, 0);
        assert_eq!(storage.cur, 0);
    }
}
