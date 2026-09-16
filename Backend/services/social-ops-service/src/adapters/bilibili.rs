use async_trait::async_trait;
use serde_json::Value;

use super::browser_client::BrowserClient;
use super::{CrawlerAdapter, RawContent};

pub struct BiliCrawler;

impl BiliCrawler {
    fn build_client() -> reqwest::Result<reqwest::Client> {
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36" )
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(reqwest::header::REFERER, reqwest::header::HeaderValue::from_static("https://www.bilibili.com/" ));
                h
            })
            .build()
    }

    async fn try_search(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<RawContent>> {
        let resp = client.get(url).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("B站 API 返回 {status}" );
        }

        let json: Value = serde_json::from_str(&text)?;
        if json["code" ].as_i64() != Some(0) {
            anyhow::bail!("B站 API code: {}" , json["code" ]);
        }

        let mut contents = Vec::new();

        // 尝试解析 search/type 格式 (data.result)
        if let Some(items) = json["data" ]["result" ].as_array() {
            for item in items {
                contents.push(RawContent {
                    text: format!("【{}】\n{}" ,
                        item["title" ].as_str().unwrap_or("" ),
                        item["desc" ].as_str().unwrap_or("" )),
                    images: vec![item["pic" ].as_str().unwrap_or("" ).to_string()],
                    source: "bilibili".to_string(),
                    url: format!("https://www.bilibili.com/video/{}" , item["bvid" ].as_str().unwrap_or("" )),
                });
            }
        }

        // 尝试解析 search/default 格式 (一层的 data)
        if contents.is_empty()
            && let Some(name) = json["data" ]["show_name" ].as_str() {
                let url = json["data" ]["url" ].as_str().unwrap_or("" );
                contents.push(RawContent {
                    text: name.to_string(),
                    images: vec![],
                    source: "bilibili".to_string(),
                    url: url.to_string(),
                });
            }

        Ok(contents)
    }
}

#[async_trait]
impl CrawlerAdapter for BiliCrawler {
    /// B站使用公开 API `抓取，不需要浏览器，browser_client` 参数忽略
    async fn crawl(&self, keyword: &str, _browser_client: &BrowserClient) -> anyhow::Result<Vec<RawContent>> {
        let client = Self::build_client()?;
        let encoded = urlencoding::encode(keyword);

        // 依次尝试多个端点
        let urls = vec![
            format!("https://api.bilibili.com/x/web-interface/search/type?search_type=video&keyword={}&page=1" , encoded),
            format!("https://api.bilibili.com/x/web-interface/search/default?keyword={}&page=1" , encoded),
        ];

        for url in urls {
            match Self::try_search(&client, &url).await {
                Ok(items) if !items.is_empty() => return Ok(items),
                _ => continue,
            }
        }

        Ok(Vec::new())
    }
}
