//! HTTP 客户端配置 — 单例/连接池模式
//!
//! 所有 HTTP 请求共享同一个 `reqwest::Client` 实例，
//! 启用 TCP 连接池（`pool_max_idle_per_host`），避免每次请求都新建连接。

use reqwest::Client;
use std::sync::Arc;

/// HTTP 客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 请求超时（秒）
    pub timeout_secs: u64,
    /// 每个 host 的最大空闲连接数
    pub max_idle_connections_per_host: u32,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_idle_connections_per_host: 10,
        }
    }
}

/// HTTP 客户端管理器（单例）
///
/// 内部持有 `Arc<Client>`，所有调用者共享同一个底层连接池。
#[derive(Debug, Clone)]
pub struct HttpClientManager {
    /// 共享的 HTTP 客户端
    pub client: Arc<Client>,
    /// 配置
    pub config: HttpClientConfig,
}

impl HttpClientManager {
    /// 创建一个新的 HTTP 客户端管理器
    ///
    /// # Panics
    /// 如果 `reqwest::Client::builder().build()` 失败则 panic。
    #[must_use]
    pub fn new(config: HttpClientConfig) -> Self {
        let builder = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(config.max_idle_connections_per_host as usize)
            .http2_keep_alive_interval(std::time::Duration::from_secs(30))
            .http2_keep_alive_timeout(std::time::Duration::from_secs(60))
            .http2_keep_alive_while_idle(true);

        let client = builder
            .build()
            .expect("Failed to build HTTP client (invalid TLS/HTTP2 config)");

        Self {
            client: Arc::new(client),
            config,
        }
    }

    /// 获取 HTTP 客户端（`Arc<Client>`，clone 是 O(1)）
    #[must_use]
    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
    }
}
