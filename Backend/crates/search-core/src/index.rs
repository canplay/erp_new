//! 索引管理

use serde::{Deserialize, Serialize};

/// 索引设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexSettings {
    /// 可搜索属性
    #[serde(rename = "searchableAttributes", skip_serializing_if = "Vec::is_empty")]
    pub searchable: Vec<String>,
    /// 可过滤属性
    #[serde(rename = "filterableAttributes", skip_serializing_if = "Vec::is_empty")]
    pub filterable: Vec<String>,
    /// 可排序属性
    #[serde(rename = "sortableAttributes", skip_serializing_if = "Vec::is_empty")]
    pub sortable: Vec<String>,
    /// 停用词
    #[serde(rename = "stop-words", skip_serializing_if = "Option::is_none")]
    pub stop_words: Option<Vec<String>>,
    /// 同义词
    #[serde(rename = "synonyms", skip_serializing_if = "Option::is_none")]
    pub synonyms: Option<std::collections::HashMap<String, Vec<String>>>,
    /// 校验和
    #[serde(rename = "checksum", skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub uid: String,
    pub name: Option<String>,
    #[serde(rename = "primaryKey")]
    pub primary_key: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// 索引统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub number_of_documents: usize,
    #[serde(rename = "isIndexingTasks")]
    pub is_indexing_tasks: Option<bool>,
    #[serde(rename = "isFieldDistributionDefinedByType")]
    pub is_field_distribution_defined_by_type: Option<bool>,
}
