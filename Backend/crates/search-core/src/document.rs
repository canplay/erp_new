//! 搜索文档类型
//!
//! 提供文档trait抽象，用于 Meilisearch 文档模型

use serde::{Deserialize, Serialize};

/// 搜索文档 trait
///
/// 实现此 trait 的类型可以被索引到 Meilisearch
pub trait SearchDocument {
    /// 获取文档 ID
    fn document_id(&self) -> &str;

    /// 序列化为 JSON 值
    fn to_json(&self) -> serde_json::Value;

    /// 从 JSON 值反序列化
    fn from_json(value: serde_json::Value) -> Self;
}

/// 通用搜索文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericDocument {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(flatten)]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl SearchDocument for GenericDocument {
    fn document_id(&self) -> &str {
        &self.id
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    fn from_json(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap_or_else(|_| Self {
            id: String::new(),
            fields: serde_json::Map::new(),
        })
    }
}

/// 搜索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse<T> {
    pub hits: Vec<T>,
    pub estimated_total_hits: Option<usize>,
    pub total_hits: Option<usize>,
    pub processing_time_ms: Option<u64>,
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub facet_distribution: Option<serde_json::Value>,
    pub facet_stats: Option<serde_json::Value>,
}

/// 分页搜索参数
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: String,
    pub offset: usize,
    pub limit: usize,
    pub filters: Option<String>,
    pub sort: Option<Vec<String>>,
    pub attributes_to_fetch: Option<Vec<String>>,
    pub attributes_to_crop: Option<Vec<String>>,
    pub crop_length: Option<usize>,
    pub attributes_to_highlight: Option<Vec<String>>,
}

impl SearchParams {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: 20,
            offset: 0,
            ..Default::default()
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn filters(mut self, filters: impl Into<String>) -> Self {
        self.filters = Some(filters.into());
        self
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("q".to_string(), self.query.clone()),
            ("offset".to_string(), self.offset.to_string()),
            ("limit".to_string(), self.limit.to_string()),
        ];

        if let Some(ref filters) = self.filters {
            params.push(("filter".to_string(), filters.clone()));
        }

        if let Some(ref sort) = self.sort {
            params.push(("sort".to_string(), sort.join(",")));
        }

        if let Some(ref attrs) = self.attributes_to_fetch {
            params.push(("attributesToFetch".to_string(), attrs.join(",")));
        }

        params
    }
}
