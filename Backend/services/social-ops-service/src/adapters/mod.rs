pub mod bilibili;
pub mod browser_client;
pub mod douyin;
pub mod wechat;
pub mod weibo;
pub mod xiaohongshu;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 抓取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContent {
    pub text: String,
    pub images: Vec<String>,
    pub source: String,
    pub url: String,
}

/// Crawler trait — 所有平台爬虫实现此接口
#[async_trait]
pub trait CrawlerAdapter: Send + Sync {
    /// 使用 browser-service 抓取指定关键词的内容
    async fn crawl(
        &self,
        keyword: &str,
        browser_client: &browser_client::BrowserClient,
    ) -> anyhow::Result<Vec<RawContent>>;
}
