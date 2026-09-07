use async_trait::async_trait;

use super::browser_client::BrowserClient;
use super::{CrawlerAdapter, RawContent};

pub struct WeiboCrawler;

impl Default for WeiboCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl WeiboCrawler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CrawlerAdapter for WeiboCrawler {
    async fn crawl(
        &self,
        keyword: &str,
        client: &BrowserClient,
    ) -> anyhow::Result<Vec<RawContent>> {
        tracing::info!("[WeiboCrawler] crawling '{}' via browser-service", keyword);

        // 1. 创建浏览器标签页
        let session_id = client.create_session().await?;
        tracing::info!("[WeiboCrawler] session_id={}", session_id);

        // 2. 导航到微博搜索页
        let search_url = format!(
            "https://s.weibo.com/weibo?q={}&typeall=1&suball=1&page=1",
            urlencoding::encode(keyword)
        );
        client.navigate(&session_id, &search_url).await?;

        // 3. 等待页面加载
        client.wait(&session_id, 5).await?;

        // 4. 获取页面 HTML 并解析
        let html = client.get_html(&session_id).await?;
        let results = parse_weibo_html(&html, keyword);

        // 5. 关闭标签页
        let _ = client.close_session(&session_id).await;

        tracing::info!("[WeiboCrawler] found {} results", results.len());
        Ok(results)
    }
}

fn parse_weibo_html(html: &str, keyword: &str) -> Vec<RawContent> {
    let mut results = Vec::new();

    // 简单提取 card-wrap 中的文本
    for card in html.split("card-wrap") {
        if card.len() < 50 {
            continue;
        }

        // 提取文本
        let text = extract_text_between(card, "\\\"text\\\":\\\"", "\\\"");
        if text.len() < 10 {
            continue;
        }
        let text = text.replace("\\n", "\n").replace("\\\"", "\"").replace("&nbsp;", " ");

        // 提取图片 URL
        let mut images = Vec::new();
        for (start, _) in card.match_indices("src=\"https://") {
            let remain = &card[start + 5..];
            if let Some(end) = remain.find('\"') {
                let img_url = &remain[..end];
                if img_url.contains("sinaimg") {
                    images.push(img_url.to_string());
                }
            }
        }

        results.push(RawContent {
            text,
            images,
            source: "weibo".to_string(),
            url: format!("https://s.weibo.com/weibo?q={}", urlencoding::encode(keyword)),
        });
    }

    results
}

/// 提取两个字符串之间的文本
fn extract_text_between(s: &str, start: &str, end: &str) -> String {
    if let Some(pos) = s.find(start) {
        let from = pos + start.len();
        let rest = &s[from..];
        if let Some(end_pos) = rest.find(end) {
            return rest[..end_pos].to_string();
        }
    }
    String::new()
}
