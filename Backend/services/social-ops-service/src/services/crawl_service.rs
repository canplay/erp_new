use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use sha2::{Sha256, Digest};
use std::sync::Arc;

use crate::adapters::browser_client::BrowserClient;
use crate::adapters::CrawlerAdapter;
use crate::adapters::douyin::DouyinCrawler;
use crate::adapters::wechat::WechatCrawler;
use crate::adapters::weibo::WeiboCrawler;
use crate::adapters::xiaohongshu::XiaohongshuCrawler;
use crate::adapters::bilibili::BiliCrawler;

#[derive(Clone)]
pub struct CrawlService {
    db: PgPool,
    browser_client: Arc<BrowserClient>,
}

impl CrawlService {
    #[must_use]
    pub const fn new(db: PgPool, browser_client: Arc<BrowserClient>) -> Self {
        Self { db, browser_client }
    }

    pub async fn list_sources(&self) -> Result<Vec<Value>, sqlx::Error> {
        let rows = sqlx::query(r#"SELECT id, platform, source_name, source_config::text AS "source_config", is_active, crawl_interval,
               to_char(last_crawled_at, 'YYYY-MM-DD HH24:MI:SS') AS "last_crawled"
             FROM socialops.crawl_sources ORDER BY created_at DESC"#).fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.id, "platform": row.platform, "source_name": row.source_name,
                "source_config": row.source_config, "is_active": row.is_active,
                "crawl_interval": row.crawl_interval, "last_crawled_at": row.last_crawled
            })
        }).collect())
    }

    pub async fn create_source(&self, platform: &str, name: &str, config: &Value, interval: i32) -> Result<Value, sqlx::Error> {
        let row = sqlx::query(r#"INSERT INTO socialops.crawl_sources (platform, source_name, source_config, crawl_interval)
             VALUES ($1, $2, $3, $4)
             RETURNING id, platform, source_name AS "source_name!", crawl_interval"#).bind(platform).bind(name).bind(config).bind(interval).bind()
        .fetch_one(&self.db).await?;

        Ok(serde_json::json!({
            "id": row.id, "platform": row.platform, "source_name": row.source_name, "crawl_interval": row.crawl_interval
        }))
    }

    pub async fn delete_source(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let r = sqlx::query("DELETE FROM socialops.crawl_sources WHERE id = $1").bind(id)
            .execute(&self.db).await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn create_task(&self, source_id: Uuid) -> Result<Uuid, sqlx::Error> {
        let row = sqlx::query_scalar(r#"INSERT INTO socialops.crawl_tasks (source_id, status) VALUES ($1, 'pending') RETURNING id"#).bind(source_id)
        .fetch_one(&self.db).await?;
        Ok(row)
    }

    /// B站公开 API 抓取 — 纯 reqwest，不需要浏览器
    pub async fn crawl_bilibili(&self, task_id: Uuid, keyword: &str) -> Result<i32, anyhow::Error> {
        sqlx::query("UPDATE socialops.crawl_tasks SET status = 'running', started_at = NOW() WHERE id = $1").bind(task_id)
        .execute(&self.db).await?;

        let crawler = BiliCrawler;
        let results = crawler.crawl(keyword, &self.browser_client).await?;

        let mut new_count = 0i32;
        for item in &results {
            let hash = Sha256::digest(item.text.as_bytes());
            let source_hash = format!("bilibili:{}", &hex::encode(hash)[..32]);
            let exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.content_items WHERE source_hash = $1").bind(&source_hash).fetch_one(&self.db).await.unwrap_or(Some(0)).unwrap_or(0);
            if exists == 0 {
                let title = item.text.chars().take(100).collect::<String>();
                sqlx::query(r#"INSERT INTO socialops.content_items (source_type, content_type, title, body, source_url, source_hash, author_name, status)
                     VALUES ('crawled', 'video', $1, $2, $3, $4, $5, 'draft')"#).bind(&title).bind(&item.text).bind(&item.url).bind(&source_hash).bind(&item.source).bind()
                .execute(&self.db).await?;
                new_count += 1;
            }
        }

        sqlx::query("UPDATE socialops.crawl_tasks SET status = 'completed', items_found = $1, items_new = $2, completed_at = NOW() WHERE id = $3").bind(results.len() as i32).bind(new_count).bind(task_id)
        .execute(&self.db).await?;
        Ok(new_count)
    }

    /// 通用浏览器爬虫流程：运行 adapter -> 去重存入 `content_items` -> 更新 task 状态
    async fn crawl_with_adapter(
        &self,
        task_id: Uuid,
        adapter: &dyn CrawlerAdapter,
        keyword: &str,
        platform: &str,
    ) -> Result<i32, anyhow::Error> {
        sqlx::query("UPDATE socialops.crawl_tasks SET status = 'running', started_at = NOW() WHERE id = $1").bind(task_id)
        .execute(&self.db).await?;

        let results = adapter.crawl(keyword, &self.browser_client).await?;

        let mut new_count = 0i32;
        for item in &results {
            let hash = Sha256::digest(item.text.as_bytes());
            let source_hash = format!("{}:{}", platform, hex::encode(hash));

            let exists = sqlx::query_scalar("SELECT COUNT(*) FROM socialops.content_items WHERE source_hash = $1").bind(&source_hash)
            .fetch_one(&self.db).await
            .unwrap_or(Some(0))
            .unwrap_or(0);

            if exists == 0 {
                let title = item.text.chars().take(100).collect::<String>();
                sqlx::query(r#"INSERT INTO socialops.content_items
                       (source_type, content_type, title, body, source_url, source_hash, author_name, status)
                       VALUES ('crawled', 'post', $1, $2, $3, $4, $5, 'draft')"#).bind(&title).bind(&item.text).bind(&item.url).bind(&source_hash).bind(&item.source).bind()
                .execute(&self.db).await?;
                new_count += 1;
            }
        }

        let total = results.len() as i32;
        sqlx::query(r#"UPDATE socialops.crawl_tasks SET status = 'completed', items_found = $1, items_new = $2,
               completed_at = NOW() WHERE id = $3"#).bind(total).bind(new_count).bind(task_id).bind()
        .execute(&self.db).await?;

        Ok(new_count)
    }

    pub async fn list_crawl_tasks(&self, source_id: Uuid) -> Result<Vec<Value>, sqlx::Error> {
        let rows = sqlx::query(r#"SELECT id, status, COALESCE(items_found, 0) AS "items_found!", COALESCE(items_new, 0) AS "items_new!",
               to_char(started_at, 'YYYY-MM-DD HH24:MI:SS') AS "started_at",
               to_char(completed_at, 'YYYY-MM-DD HH24:MI:SS') AS "completed_at"
             FROM socialops.crawl_tasks
             WHERE source_id = $1
             ORDER BY created_at DESC"#).bind(source_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "status": row.status,
                    "items_found": row.items_found,
                    "items_new": row.items_new,
                    "started_at": row.started_at,
                    "completed_at": row.completed_at,
                })
            })
            .collect())
    }

    pub async fn trigger_crawl(&self, source_id: Uuid) -> Result<String, anyhow::Error> {
        let row = sqlx::query(r#"SELECT platform, source_config FROM socialops.crawl_sources WHERE id = $1"#).bind(source_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("source not found: {source_id}"))?;

        let platform = row.platform;
        let source_config = row.source_config;

        let keyword = source_config["keyword"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("keyword not found in source_config"))?
            .to_string();

        let task_id = self.create_task(source_id).await?;

        match platform.as_str() {
            "bilibili" => {
                let new_count = self.crawl_bilibili(task_id, &keyword).await?;
                Ok(format!("bilibili crawl done, {new_count} new items"))
            }
            "weibo" => {
                let crawler = WeiboCrawler::new();
                let new_count = self.crawl_with_adapter(task_id, &crawler, &keyword, "weibo").await?;
                Ok(format!("weibo crawl done, {new_count} new items"))
            }
            "xiaohongshu" => {
                let crawler = XiaohongshuCrawler::new();
                let new_count = self.crawl_with_adapter(task_id, &crawler, &keyword, "xiaohongshu").await?;
                Ok(format!("xiaohongshu crawl done, {new_count} new items"))
            }
            "douyin" => {
                let crawler = DouyinCrawler::new();
                let new_count = self.crawl_with_adapter(task_id, &crawler, &keyword, "douyin").await?;
                Ok(format!("douyin crawl done, {new_count} new items"))
            }
            "wechat" => {
                let article_url = source_config["url"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("url not found in source_config for wechat platform"))?
                    .to_string();
                let crawler = WechatCrawler::new();
                let new_count = self.crawl_with_adapter(task_id, &crawler, &article_url, "wechat").await?;
                Ok(format!("wechat crawl done, {new_count} new items"))
            }
            _ => Err(anyhow::anyhow!("unsupported platform: {platform}")),
        }
    }
}
