use async_trait::async_trait;

use super::browser_client::BrowserClient;
use super::{CrawlerAdapter, RawContent};

pub struct WechatCrawler;

impl Default for WechatCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl WechatCrawler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CrawlerAdapter for WechatCrawler {
    /// 微信公众号没有公开搜索，keyword 参数直接作为 `article_url` 使用
    async fn crawl(
        &self,
        keyword: &str,
        client: &BrowserClient,
    ) -> anyhow::Result<Vec<RawContent>> {
        tracing::info!("[WechatCrawler] loading article '{}' via browser-service" , keyword);

        let session_id = client.create_session().await?;

        client.navigate(&session_id, keyword).await?;
        client.wait(&session_id, 5).await?;

        let title = client.get_title(&session_id).await?;
        let text = client.get_text(&session_id).await?;

        let _ = client.close_session(&session_id).await;

        if text.trim().is_empty() {
            Ok(vec![])
        } else {
            let combined = if title.is_empty() {
                text
            } else {
                format!("【{title}】\n{text}" )
            };

            tracing::info!("[WechatCrawler] extracted article '{}'" , title);
            Ok(vec![RawContent {
                text: combined,
                images: vec![],
                source: "wechat".to_string(),
                url: keyword.to_string(),
            }])
        }
    }
}
