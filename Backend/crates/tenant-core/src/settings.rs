//! 租户设置与配置管理
//!
//! 提供租户级别的配置存储和读取功能。

use crate::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 租户设置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<i64>,
}

/// 租户设置集合
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantSettings {
    pub tenant_id: TenantId,
    pub settings: HashMap<String, TenantSetting>,
    pub updated_at: DateTime<Utc>,
}

impl TenantSettings {
    /// 创建新的租户设置集合
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            settings: HashMap::new(),
            updated_at: Utc::now(),
        }
    }

    /// 获取设置值
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.settings.get(key).map(|s| &s.value)
    }

    /// 获取设置值（带类型转换）
    pub fn get_typed<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.settings.get(key).and_then(|s| serde_json::from_value(s.value.clone()).ok())
    }

    /// 设置值
    pub fn set(&mut self, key: impl Into<String>, value: impl Serialize) -> Result<(), serde_json::Error> {
        let key = key.into();
        let value = serde_json::to_value(value)?;
        let setting = TenantSetting {
            key: key.clone(),
            value,
            category: "general".to_string(),
            description: None,
            is_public: false,
            updated_at: Utc::now(),
            updated_by: None,
        };
        self.settings.insert(key, setting);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 设置带分类的值
    pub fn set_with_category(
        &mut self,
        key: impl Into<String>,
        value: impl Serialize,
        category: impl Into<String>,
    ) -> Result<(), serde_json::Error> {
        let key = key.into();
        let value = serde_json::to_value(value)?;
        let setting = TenantSetting {
            key: key.clone(),
            value,
            category: category.into(),
            description: None,
            is_public: false,
            updated_at: Utc::now(),
            updated_by: None,
        };
        self.settings.insert(key, setting);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 批量设置
    pub fn set_many(&mut self, values: HashMap<String, serde_json::Value>) {
        for (key, value) in values {
            let setting = TenantSetting {
                key: key.clone(),
                value,
                category: "general".to_string(),
                description: None,
                is_public: false,
                updated_at: Utc::now(),
                updated_by: None,
            };
            self.settings.insert(key, setting);
        }
        self.updated_at = Utc::now();
    }

    /// 删除设置
    pub fn remove(&mut self, key: &str) -> Option<TenantSetting> {
        self.settings.remove(key)
    }

    /// 获取分类下的所有设置
    pub fn get_by_category(&self, category: &str) -> Vec<&TenantSetting> {
        self.settings
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    /// 获取所有公开设置
    pub fn get_public(&self) -> Vec<&TenantSetting> {
        self.settings
            .values()
            .filter(|s| s.is_public)
            .collect()
    }

    /// 清空所有设置
    pub fn clear(&mut self) {
        self.settings.clear();
        self.updated_at = Utc::now();
    }

    /// 合并另一个设置集合
    pub fn merge(&mut self, other: &Self) {
        for (key, setting) in &other.settings {
            self.settings.insert(key.clone(), setting.clone());
        }
        self.updated_at = Utc::now();
    }
}

/// 租户主题配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantTheme {
    /// 主色调
    pub primary_color: Option<String>,
    /// 次要色
    pub secondary_color: Option<String>,
    /// 背景色
    pub background_color: Option<String>,
    /// Logo URL
    pub logo_url: Option<String>,
    /// Favicon URL
    pub favicon_url: Option<String>,
    /// 自定义 CSS
    pub custom_css: Option<String>,
    /// 深色模式
    pub dark_mode: bool,
    /// 紧凑模式
    pub compact_mode: bool,
    /// 语言
    pub language: String,
    /// 时区
    pub timezone: String,
}

impl TenantTheme {
    /// 创建默认主题
    pub fn default() -> Self {
        Self {
            primary_color: Some("#1976d2".to_string()),
            secondary_color: Some("#26a69a".to_string()),
            background_color: None,
            logo_url: None,
            favicon_url: None,
            custom_css: None,
            dark_mode: false,
            compact_mode: false,
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
        }
    }

    /// 应用主题到设置
    pub fn apply_to_settings(&self, settings: &mut TenantSettings) -> Result<(), serde_json::Error> {
        settings.set("theme" , self)
    }
}

/// 租户功能配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantFeatures {
    /// 是否启用聊天
    pub chat_enabled: bool,
    /// 是否启用文件上传
    pub file_upload_enabled: bool,
    /// 是否启用语音消息
    pub voice_message_enabled: bool,
    /// 是否启用视频通话
    pub video_call_enabled: bool,
    /// 是否启用朋友圈
    pub moments_enabled: bool,
    /// 是否启用朋友圈评论
    pub moments_comment_enabled: bool,
    /// 是否启用朋友圈点赞
    pub moments_like_enabled: bool,
    /// 是否启用通讯录
    pub contacts_enabled: bool,
    /// 是否启用群组
    pub group_enabled: bool,
    /// 最大群成员数
    pub max_group_members: i32,
    /// 单文件大小限制 (MB)
    pub max_file_size_mb: i32,
    /// 总存储空间限制 (GB)
    pub max_storage_gb: i32,
}

impl TenantFeatures {
    /// 创建默认功能配置
    pub fn default() -> Self {
        Self {
            chat_enabled: true,
            file_upload_enabled: true,
            voice_message_enabled: true,
            video_call_enabled: false,
            moments_enabled: true,
            moments_comment_enabled: true,
            moments_like_enabled: true,
            contacts_enabled: true,
            group_enabled: true,
            max_group_members: 500,
            max_file_size_mb: 50,
            max_storage_gb: 10,
        }
    }

    /// 创建免费套餐功能配置
    pub fn free_plan() -> Self {
        Self {
            video_call_enabled: false,
            max_group_members: 100,
            max_file_size_mb: 10,
            max_storage_gb: 1,
            ..Self::default()
        }
    }

    /// 创建标准套餐功能配置
    pub fn standard_plan() -> Self {
        Self {
            video_call_enabled: true,
            max_group_members: 500,
            max_file_size_mb: 50,
            max_storage_gb: 10,
            ..Self::default()
        }
    }

    /// 创建企业套餐功能配置
    pub fn enterprise_plan() -> Self {
        Self {
            video_call_enabled: true,
            max_group_members: 5000,
            max_file_size_mb: 200,
            max_storage_gb: 100,
            ..Self::default()
        }
    }
}
