use async_trait::async_trait;

use super::browser_client::BrowserClient;
use super::{CrawlerAdapter, RawContent};

pub struct DouyinCrawler;

impl Default for DouyinCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl DouyinCrawler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CrawlerAdapter for DouyinCrawler {
    async fn crawl(
        &self,
        keyword: &str,
        client: &BrowserClient,
    ) -> anyhow::Result<Vec<RawContent>> {
        tracing::info!("[DouyinCrawler] crawling '{}' via browser-service", keyword);

        let session_id = client.create_session().await?;

        let search_url = format!("https://www.douyin.com/search/{keyword}?type=general");
        client.navigate(&session_id, &search_url).await?;

        // 等待页面加载
        client.wait(&session_id, 8).await?;

        // 获取页面文本
        let text = client.get_text(&session_id).await?;
        let results = parse_douyin_text(&text, keyword);

        let _ = client.close_session(&session_id).await;

        tracing::info!("[DouyinCrawler] found {} results", results.len());
        Ok(results)
    }
}

fn parse_douyin_text(text: &str, keyword: &str) -> Vec<RawContent> {
    let mut results = Vec::new();
    // 按行分割，过滤出较长的有意义的文本
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 20 {
            results.push(RawContent {
                text: trimmed.to_string(),
                images: vec![],
                source: "douyin".to_string(),
                url: format!("https://www.douyin.com/search/{}?type=general", urlencoding::encode(keyword)),
            });
        }
        if results.len() >= 20 {
            break;
        }
    }
    results
}
