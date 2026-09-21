//! 搜索错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON 序列化/反序列化失败: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Meilisearch API 错误: {status} - {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    #[error("未知错误: {0}")]
    Unknown(String),
}

pub type SearchResult<T> = std::result::Result<T, SearchError>;
