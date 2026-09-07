//! 浏览器池管理
//!
//! 基于 drission 的 `ChromiumPage` 管理多个无头浏览器实例，
//! 通过轮询调度分配浏览器标签页给请求方。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use drission::cdp::{ChromiumBrowser, ChromiumOptions, ChromiumPage};
use tokio::sync::Mutex;

/// 浏览器池 — 管理多个 `ChromiumPage` 实例
pub struct BrowserPool {
    browsers: Vec<Arc<Mutex<ChromiumPage>>>,
    pool_size: usize,
    next_index: AtomicUsize,
}

impl BrowserPool {
    /// 创建指定大小的浏览器池
    pub async fn new(size: usize) -> anyhow::Result<Self> {
        let edge_path = ChromiumBrowser::find_chrome().ok();
        let mut opts = ChromiumOptions::new().headless(true);
        if let Some(ref path) = edge_path {
            opts = opts.binary_path(path.clone());
        }

        let mut browsers = Vec::with_capacity(size);
        for i in 0..size {
            tracing::info!("[BrowserPool] launching browser #{}/{}", i + 1, size);
            match ChromiumPage::with(opts.clone()).await {
                Ok(page) => {
                    tracing::info!("[BrowserPool] browser #{} ready", i + 1);
                    browsers.push(Arc::new(Mutex::new(page)));
                }
                Err(e) => {
                    tracing::error!("[BrowserPool] failed to launch browser #{}: {}", i + 1, e);
                    if browsers.is_empty() {
                        anyhow::bail!("无法启动任何浏览器: {e}");
                    }
                    tracing::warn!("[BrowserPool] continuing with {} browsers", browsers.len());
                }
            }
        }

        let pool_size = browsers.len();
        Ok(Self {
            browsers,
            pool_size,
            next_index: AtomicUsize::new(0),
        })
    }

    /// 轮询获取一个浏览器实例（用于创建新标签页）
    pub fn get_page(&self) -> Option<Arc<Mutex<ChromiumPage>>> {
        if self.browsers.is_empty() {
            return None;
        }
        let idx = self.next_index.fetch_add(1, Ordering::Relaxed) % self.pool_size;
        Some(self.browsers[idx].clone())
    }

    /// 池大小
    pub const fn size(&self) -> usize {
        self.pool_size
    }
}

/// 浏览器会话句柄 — 对应一个打开的标签页
pub struct BrowserSession {
    pub id: String,
    pub tab: tokio::sync::Mutex<Option<drission::cdp::ChromiumTab>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl BrowserSession {
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id,
            tab: tokio::sync::Mutex::new(None),
            created_at: chrono::Utc::now(),
        }
    }
}

/// Crawler trait — 所有平台爬虫实现此接口
#[async_trait::async_trait]
pub trait CrawlerAdapter: Send + Sync {
    /// 使用浏览器池抓取指定关键词的内容
    async fn crawl(
        &self,
        keyword: &str,
        pool: Arc<BrowserPool>,
    ) -> anyhow::Result<Vec<RawContent>>;
}

/// 抓取结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RawContent {
    pub text: String,
    pub images: Vec<String>,
    pub source: String,
    pub url: String,
}
