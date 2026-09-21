//! Meilisearch HTTP 客户端
//!
//! 提供与 Meilisearch REST API 交互的客户端实现

use crate::document::{SearchParams, SearchResponse};
use crate::error::{SearchError, SearchResult};
use crate::index::IndexSettings;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;

pub use reqwest;

/// Meilisearch 客户端
#[derive(Clone)]
pub struct SearchClient {
    client: reqwest::Client,
    base_url: Arc<String>,
    api_key: Option<Arc<String>>,
}

impl SearchClient {
    /// 创建新的 Meilisearch 客户端
    ///
    /// # 参数
    /// - `url`: Meilisearch 服务地址，如 `http://meilisearch:7700`
    /// - `api_key`: 可选的 API 密钥
    pub fn new(url: &str, api_key: Option<&str>) -> SearchResult<Self> {
        let builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30));

        let client = builder.build().map_err(SearchError::Http)?;

        let base_url = if url.ends_with('/') {
            url[..url.len() - 1].to_string()
        } else {
            url.to_string()
        };

        Ok(Self {
            client,
            base_url: Arc::new(base_url),
            api_key: api_key.map(|k| Arc::new(k.to_string())),
        })
    }

    /// 构建带认证头的请求
    fn build_request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let full_url = format!("{}{}", self.base_url, url);
        let mut request = self.client.request(method, &full_url);

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key.as_str()));
        }

        request
    }

    /// 健康检查
    pub async fn health(&self) -> SearchResult<bool> {
        let response = self
            .build_request(Method::GET, "/health")
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            Ok(json.get("status").and_then(|s| s.as_str()) == Some("available"))
        } else {
            Ok(false)
        }
    }

    /// 创建索引
    pub async fn create_index(&self, uid: &str) -> SearchResult<Value> {
        let body = serde_json::json!({ "uid": uid });
        let response = self
            .build_request(Method::POST, "/indexes")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 201 || status == 202 {
            Ok(json)
        } else if status == 409 {
            tracing::info!("索引 {} 已存在", uid);
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 删除索引
    pub async fn delete_index(&self, uid: &str) -> SearchResult<()> {
        let response = self
            .build_request(Method::DELETE, &format!("/indexes/{}", uid))
            .send()
            .await?;

        let status = response.status();
        if status == 202 {
            Ok(())
        } else {
            let json: Value = response.json().await?;
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 获取所有索引列表
    pub async fn list_indexes(&self) -> SearchResult<Vec<Value>> {
        let response = self
            .build_request(Method::GET, "/indexes")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let json: Value = response.json().await?;
            let indexes = json.get("results").and_then(|r| serde_json::from_value::<Vec<Value>>(r.clone()).ok()).unwrap_or_default();
            Ok(indexes)
        } else {
            let json: Value = response.json().await?;
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 更新索引设置
    pub async fn update_index_settings(&self, uid: &str, settings: &IndexSettings) -> SearchResult<Value> {
        let response = self
            .build_request(Method::PATCH, &format!("/indexes/{}/settings", uid))
            .json(settings)
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 201 || status == 202 {
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 添加文档
    pub async fn add_documents(&self, uid: &str, documents: &[Value], primary_key: Option<&str>) -> SearchResult<Value> {
        let url = match primary_key {
            Some(pk) => format!("/indexes/{}/documents?primaryKey={}", uid, pk),
            None => format!("/indexes/{}/documents", uid),
        };

        let response = self
            .build_request(Method::POST, &url)
            .json(&documents)
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 202 {
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 批量添加文档（使用 task 队列）
    pub async fn add_documents_batched(
        &self,
        uid: &str,
        documents: Vec<Value>,
        batch_size: usize,
        primary_key: Option<&str>,
    ) -> SearchResult<Vec<Value>> {
        let mut results = Vec::new();
        let pk = primary_key.unwrap_or("id");

        for chunk in documents.chunks(batch_size) {
            let result = self.add_documents(uid, chunk, Some(pk)).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 删除单个文档
    pub async fn delete_document(&self, uid: &str, document_id: &str) -> SearchResult<Value> {
        let response = self
            .build_request(Method::DELETE, &format!("/indexes/{}/documents/{}", uid, document_id))
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 202 {
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 删除多个文档
    pub async fn delete_documents(&self, uid: &str, document_ids: &[String]) -> SearchResult<Value> {
        let response = self
            .build_request(Method::POST, &format!("/indexes/{}/documents/delete", uid))
            .json(&serde_json::json!({ "ids": document_ids }))
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 202 {
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 删除全部文档
    pub async fn delete_all_documents(&self, uid: &str) -> SearchResult<Value> {
        let response = self
            .build_request(Method::DELETE, &format!("/indexes/{}/documents", uid))
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 202 {
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 更新单个文档
    pub async fn update_document(&self, uid: &str, document: Value, primary_key: Option<&str>) -> SearchResult<Value> {
        let pk = primary_key.unwrap_or("id");
        let doc_id = document.get(pk)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let response = self
            .build_request(Method::PATCH, &format!("/indexes/{}/documents/{}", uid, doc_id))
            .json(&document)
            .send()
            .await?;

        let status = response.status();
        let json: Value = response.json().await?;

        if status == 202 {
            Ok(json)
        } else {
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 搜索文档
    pub async fn search<T: DeserializeOwned>(
        &self,
        uid: &str,
        params: &SearchParams,
    ) -> SearchResult<SearchResponse<T>> {
        let query_params = params.to_query_params();
        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let url = if query_string.is_empty() {
            format!("/indexes/{}/search", uid)
        } else {
            format!("/indexes/{}/search?{}", uid, query_string)
        };

        let response = self
            .build_request(Method::GET, &url)
            .send()
            .await?;

        let json: Value = response.json().await?;
        let response: SearchResponse<T> = serde_json::from_value(json)?;
        Ok(response)
    }

    /// 通用搜索返回 JSON 值
    pub async fn search_raw(&self, uid: &str, params: &SearchParams) -> SearchResult<SearchResponse<Value>> {
        let query_params = params.to_query_params();
        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let url = if query_string.is_empty() {
            format!("/indexes/{}/search", uid)
        } else {
            format!("/indexes/{}/search?{}", uid, query_string)
        };

        let response = self
            .build_request(Method::GET, &url)
            .send()
            .await?;

        let json: Value = response.json().await?;
        let response: SearchResponse<Value> = serde_json::from_value(json)?;
        Ok(response)
    }

    /// 获取文档详情
    pub async fn get_document<T: DeserializeOwned>(&self, uid: &str, document_id: &str) -> SearchResult<T> {
        let response = self
            .build_request(Method::GET, &format!("/indexes/{}/documents/{}", uid, document_id))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let json: T = response.json().await?;
            Ok(json)
        } else {
            let json: Value = response.json().await?;
            Err(SearchError::ApiError {
                status: status.as_u16(),
                message: json.to_string(),
            })
        }
    }

    /// 获取文档数量
    pub async fn get_document_count(&self, uid: &str) -> SearchResult<usize> {
        let response = self
            .build_request(Method::GET, &format!("/indexes/{}", uid))
            .send()
            .await?;

        let json: Value = response.json().await?;
        let stats = json.get("stats").and_then(|s| s.as_object());

        let count = match stats {
            Some(s) => s.get("numberOfDocuments")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
            None => {
                let empty_map = serde_json::Map::new();
                empty_map.get("numberOfDocuments")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize
            }
        };

        Ok(count)
    }

    /// 获取索引统计
    pub async fn get_index_stats(&self, uid: &str) -> SearchResult<Value> {
        let response = self
            .build_request(Method::GET, &format!("/indexes/{}", uid))
            .send()
            .await?;

        let json: Value = response.json().await?;
        Ok(json)
    }

    /// 等待任务完成
    pub async fn wait_for_task(&self, task_uid: usize, timeout_ms: u64) -> SearchResult<Value> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            let response = self
                .build_request(Method::GET, &format!("/tasks/{}", task_uid))
                .send()
                .await?;

            let json: Value = response.json().await?;
            let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("");

            if status != "enqueued" && status != "processing" {
                return Ok(json);
            }

            if start.elapsed() > timeout {
                return Err(SearchError::Unknown(format!("任务 {} 超时", task_uid)));
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}
