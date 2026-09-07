use async_trait::async_trait;

use super::browser_client::BrowserClient;
use super::{CrawlerAdapter, RawContent};

pub struct XiaohongshuCrawler;

impl Default for XiaohongshuCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl XiaohongshuCrawler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CrawlerAdapter for XiaohongshuCrawler {
    async fn crawl(
        &self,
        keyword: &str,
        client: &BrowserClient,
    ) -> anyhow::Result<Vec<RawContent>> {
        tracing::info!("[XiaohongshuCrawler] crawling '{}' via browser-service", keyword);

        let session_id = client.create_session().await?;

        let search_url = format!(
            "https://www.xiaohongshu.com/search_result?keyword={keyword}&source=web_search_result_notes"
        );
        client.navigate(&session_id, &search_url).await?;

        client.wait(&session_id, 5).await?;

        // 获取页面文本
        let text = client.get_text(&session_id).await?;
        let results = parse_xiaohongshu_text(&text, keyword);

        let _ = client.close_session(&session_id).await;

        tracing::info!("[XiaohongshuCrawler] found {} results", results.len());
        Ok(results)
    }
}

fn parse_xiaohongshu_text(text: &str, keyword: &str) -> Vec<RawContent> {
    let mut results = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 15 && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
            results.push(RawContent {
                text: trimmed.to_string(),
                images: vec![],
                source: "xiaohongshu".to_string(),
                url: format!("https://www.xiaohongshu.com/search_result?keyword={}", urlencoding::encode(keyword)),
            });
        }
        if results.len() >= 20 {
            break;
        }
    }
    results
}
