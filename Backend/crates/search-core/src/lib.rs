//! search-core - Meilisearch 集成核心库
//!
//! 提供基于 HTTP 的 Meilisearch 客户端封装，支持：
//! - 索引管理
//! - 文档增删改查
//! - 全文搜索
//! - 健康检查
//!
//! 使用方式：
//! ```ignore
//! use search_core::SearchClient;
//!
//! let client = SearchClient::new("http://meilisearch:7700", None)?;
//! client.create_index("products").await?;
//! let results = client.search("products", "phone").await?;
//! ```

pub mod client;
pub mod document;
pub mod error;
pub mod index;

pub use client::SearchClient;
pub use document::{SearchDocument, SearchParams, SearchResponse};
pub use error::SearchError;
pub use index::IndexSettings;

/// CMS 内容搜索索引常量
pub mod cms {
    pub const CONTENT: &str = "content";
    pub const MEDIA: &str = "media";
    pub const CATEGORY: &str = "category";
    pub const TEMPLATE: &str = "template";
    pub const MENU: &str = "menu";
}

/// 用户相关搜索索引常量
pub mod user {
    pub const PROFILE: &str = "profile";
    pub const MESSAGE: &str = "message";
}

/// 租户相关搜索索引常量
pub mod tenant {
    pub const TENANT: &str = "tenant";
}

/// 索引管理模块
pub mod setup {
    use super::{IndexSettings, SearchClient};
    use anyhow::Result;
    use std::collections::HashMap;

    /// 设置 CMS 内容索引
    pub async fn setup_cms_indexes(client: &SearchClient, prefix: &str) -> Result<()> {
        let indexes: HashMap<String, IndexSettings> = [
            ("content", IndexSettings {
                searchable: vec!["title".to_string(), "body".to_string(), "excerpt".to_string(), "tags".to_string()],
                filterable: vec!["tenant_id".to_string(), "status".to_string(), "category_id".to_string(), "author_id".to_string(), "is_published".to_string()],
                sortable: vec!["created_at".to_string(), "updated_at".to_string(), "publish_at".to_string()],
                ..Default::default()
            }),
            ("media", IndexSettings {
                searchable: vec!["name".to_string(), "description".to_string(), "tags".to_string()],
                filterable: vec!["tenant_id".to_string(), "file_type".to_string(), "status".to_string()],
                sortable: vec!["created_at".to_string(), "updated_at".to_string(), "size".to_string()],
                ..Default::default()
            }),
            ("category", IndexSettings {
                searchable: vec!["name".to_string(), "description".to_string()],
                filterable: vec!["tenant_id".to_string(), "parent_id".to_string()],
                sortable: vec!["sort_order".to_string(), "created_at".to_string()],
                ..Default::default()
            }),
        ]
        .into_iter()
        .map(|(k, v)| (format!("{}_{}", prefix, k), v))
        .collect();

        for (name, settings) in indexes {
            client.create_index(&name).await.ok();
            client.update_index_settings(&name, &settings).await?;
        }
        Ok(())
    }
}
