//! 用户数据模型

use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub gender: i32,
    pub address: Option<String>,
    pub role: String,
    pub status: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_info_serialization() {
        let user = UserInfo {
            id: 1,
            username: "test_user".to_string(),
            nickname: Some("测试".to_string()),
            password_hash: "secret_hash".to_string(),
            avatar: None,
            phone: Some("13800000001".to_string()),
            email: None,
            gender: 1,
            address: None,
            role: "user".to_string(),
            status: 1,
            created_at: None,
            updated_at: None,
        };

        let json = serde_json::to_string(&user).expect("test assertion");
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"username\":\"test_user\""));
        assert!(json.contains("\"nickname\":\"测试\""));
        // password_hash should be skipped in serialization
        assert!(!json.contains("password_hash"));
        assert!(json.contains("\"phone\":\"13800000001\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"status\":1"));
    }

    #[test]
    fn test_user_info_deserialization() {
        let json = r#"{
            "id": 2,
            "username": "admin",
            "password_hash": "hash123",
            "gender": 1,
            "role": "admin",
            "status": 1
        }"#;
        let user: UserInfo = serde_json::from_str(json).expect("deserialization should not fail");
        assert_eq!(user.id, 2);
        assert_eq!(user.username, "admin");
        assert_eq!(user.password_hash, "hash123");
        assert_eq!(user.role, "admin");
        assert!(user.nickname.is_none());
        assert!(user.phone.is_none());
    }

    #[test]
    fn test_user_info_default_values() {
        let json = r#"{
            "id": 0,
            "username": "",
            "password_hash": "",
            "gender": 0,
            "role": "",
            "status": 0
        }"#;
        let user: UserInfo = serde_json::from_str(json).expect("test assertion");
        assert_eq!(user.id, 0);
        assert!(user.created_at.is_none());
        assert!(user.nickname.is_none());
    }
}
