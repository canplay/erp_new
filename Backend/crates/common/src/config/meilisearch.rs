//! Meilisearch 配置

use config::ConfigError;
use serde::Deserialize;

/// Meilisearch 配置
#[derive(Debug, Clone, Deserialize)]
pub struct MeilisearchConfig {
    /// Meilisearch 服务 URL
    pub url: String,
    /// API 密钥（可选）
    pub api_key: Option<String>,
    /// 索引前缀
    pub index_prefix: String,
}

impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://meilisearch:7700".to_string()),
            api_key: std::env::var("MEILISEARCH_API_KEY").ok(),
            index_prefix: std::env::var("MEILISEARCH_INDEX_PREFIX")
                .unwrap_or_else(|_| "erp_new".to_string()),
        }
    }
}

impl MeilisearchConfig {
    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::Message("MEILISEARCH_URL 不能为空".to_string()));
        }
        Ok(())
    }

    /// 构造带前缀的索引名称
    #[must_use]
    pub fn index_name(&self, index: &str) -> String {
        format!("{}_{}", self.index_prefix, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meilisearch_config_default() {
        let config = MeilisearchConfig::default();
        assert!(!config.url.is_empty());
        assert_eq!(config.index_prefix, "erp_new");
    }

    #[test]
    fn test_index_name() {
        let config = MeilisearchConfig::default();
        assert_eq!(config.index_name("content"), "erp_new_content");
    }
}
