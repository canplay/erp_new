//! 功能标志（按套餐的功能开关）
//!
//! 根据租户订阅的套餐，控制功能是否可用。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 功能标志定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub default_value: bool,
}

impl FeatureFlag {
    /// 创建新的功能标志
    pub fn new(name: impl Into<String>, default_value: bool) -> Self {
        Self {
            name: name.into(),
            description: None,
            enabled: default_value,
            default_value,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// 功能标志管理器
#[derive(Debug, Clone, Default)]
pub struct FeatureFlagManager {
    flags: HashMap<String, FeatureFlag>,
}

impl FeatureFlagManager {
    /// 创建新的功能标志管理器
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    /// 注册功能标志
    pub fn register(&mut self, flag: FeatureFlag) {
        self.flags.insert(flag.name.clone(), flag);
    }

    /// 检查功能是否启用
    pub fn is_enabled(&self, name: &str) -> bool {
        self.flags
            .get(name)
            .map(|f| f.enabled)
            .unwrap_or(false)
    }

    /// 启用功能
    pub fn enable(&mut self, name: &str) {
        if let Some(flag) = self.flags.get_mut(name) {
            flag.enabled = true;
        }
    }

    /// 禁用功能
    pub fn disable(&mut self, name: &str) {
        if let Some(flag) = self.flags.get_mut(name) {
            flag.enabled = false;
        }
    }

    /// 批量设置功能状态
    pub fn set_many(&mut self, states: HashMap<String, bool>) {
        for (name, enabled) in states {
            if let Some(flag) = self.flags.get_mut(&name) {
                flag.enabled = enabled;
            }
        }
    }

    /// 获取所有功能标志
    pub fn get_all(&self) -> Vec<&FeatureFlag> {
        self.flags.values().collect()
    }

    /// 获取所有启用的功能
    pub fn get_enabled(&self) -> Vec<&FeatureFlag> {
        self.flags
            .values()
            .filter(|f| f.enabled)
            .collect()
    }

    /// 从套餐配置初始化
    pub fn from_plan_features(features: &[String]) -> Self {
        let mut manager = Self::new();
        
        // 注册所有已知功能
        let all_features = vec![
            "chat",
            "file_upload",
            "voice_message",
            "video_call",
            "moments",
            "contacts",
            "groups",
            "analytics",
            "api_access",
            "custom_branding",
            "priority_support",
            "sso",
            "audit_log",
            "data_export",
        ];
        
        for feature in all_features {
            let enabled = features.contains(&feature.to_string());
            manager.register(FeatureFlag::new(feature, enabled));
        }
        
        manager
    }
}

/// 功能标志预设（按套餐）
pub mod presets {
    use super::*;
    use std::collections::HashMap;

    /// 免费套餐功能
    pub fn free_plan() -> HashMap<String, bool> {
        let mut map = HashMap::new();
        map.insert("chat".to_string(), true);
        map.insert("file_upload".to_string(), true);
        map.insert("voice_message".to_string(), true);
        map.insert("video_call".to_string(), false);
        map.insert("moments".to_string(), true);
        map.insert("contacts".to_string(), true);
        map.insert("groups".to_string(), true);
        map.insert("analytics".to_string(), false);
        map.insert("api_access".to_string(), false);
        map.insert("custom_branding".to_string(), false);
        map.insert("priority_support".to_string(), false);
        map.insert("sso".to_string(), false);
        map.insert("audit_log".to_string(), false);
        map.insert("data_export".to_string(), false);
        map
    }

    /// 标准套餐功能
    pub fn standard_plan() -> HashMap<String, bool> {
        let mut map = HashMap::new();
        map.insert("chat".to_string(), true);
        map.insert("file_upload".to_string(), true);
        map.insert("voice_message".to_string(), true);
        map.insert("video_call".to_string(), true);
        map.insert("moments".to_string(), true);
        map.insert("contacts".to_string(), true);
        map.insert("groups".to_string(), true);
        map.insert("analytics".to_string(), true);
        map.insert("api_access".to_string(), true);
        map.insert("custom_branding".to_string(), false);
        map.insert("priority_support".to_string(), false);
        map.insert("sso".to_string(), false);
        map.insert("audit_log".to_string(), true);
        map.insert("data_export".to_string(), true);
        map
    }

    /// 企业套餐功能
    pub fn enterprise_plan() -> HashMap<String, bool> {
        let mut map = HashMap::new();
        for feature in &[
            "chat", "file_upload", "voice_message", "video_call",
            "moments", "contacts", "groups", "analytics", "api_access",
            "custom_branding", "priority_support", "sso", "audit_log",
            "data_export",
        ] {
            map.insert(feature.to_string(), true);
        }
        map
    }
}
