//! 系统配置数据模型

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct OptionsInfo {
    pub name: String,
    pub options: Value,
    pub level: i64,
    pub create_date: Option<NaiveDateTime>,
    pub update_date: Option<NaiveDateTime>,
    pub delete: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_info_serialization() {
        let options = OptionsInfo {
            name: "site_config".to_string(),
            options: serde_json::json!({"key": "value" , "count": 10}),
            level: 1,
            create_date: None,
            update_date: None,
            delete: Some(false),
        };

        let json = serde_json::to_string(&options).expect("test assertion" );
        assert!(json.contains("\"name\":\"site_config\"" ));
        assert!(json.contains("\"level\":1" ));
        assert!(json.contains("\"key\":\"value\"" ));
        assert!(json.contains("\"count\":10" ));
    }

    #[test]
    fn test_options_info_deserialization() {
        let json = r#"{
            "name": "system" ,
            "options": {"theme": "dark" },
            "level": 2
        }"#;
        let options: OptionsInfo = serde_json::from_str(json).expect("test assertion" );
        assert_eq!(options.name, "system" );
        assert_eq!(options.options["theme" ], "dark" );
        assert_eq!(options.level, 2);
        assert!(options.delete.is_none());
    }
}
