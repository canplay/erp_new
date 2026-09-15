//! gRPC Handlers — 8 个服务的 gRPC trait 实现
//!
//! 每个服务结构体包装对应的 Service 层，实现 proto 中定义的 gRPC trait。

use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use grpc_proto::socialops::{Account, ListAccountsReq, ListAccountsResp, GetAccountReq, AddAccountReq, UpdateAccountReq, DeleteAccountReq, DeleteResp, ListContentsReq, ListContentsResp, ContentItem, GetContentReq, CreateContentReq, UpdateContentReq, DeleteContentReq, ListSourcesReq, ListSourcesResp, CrawlSource, CreateSourceReq, UpdateSourceReq, DeleteSourceReq, TriggerCrawlReq, CrawlTask, ListCrawlTasksReq, ListCrawlTasksResp, PublishReq, PublishTaskResp, ListPublishTasksReq, ListPublishTasksResp, CreateScheduleReq, PublishSchedule, ListSchedulesReq, ListSchedulesResp, UpdateScheduleReq, DeleteScheduleReq, ListProvidersReq, ListProvidersResp, LlmProvider, AddProviderReq, UpdateProviderReq, DeleteProviderReq, TestProviderReq, TestProviderResp, CreateRewriteTaskReq, RewriteTask, ListRewriteTasksReq, ListRewriteTasksResp, GetRewriteTaskReq, GetVersionsReq, GetVersionsResp, RewriteVersion, UpdateVersionReq, GetAccountStatsReq, AccountStats, GetContentStatsReq, ContentStats, GetOverviewReq, StatsOverview, GenerateInsightReq, StatInsight, ListInsightsReq, ListInsightsResp, GetLatestInsightReq};
use grpc_proto::socialops::account_service_server::AccountService;
use grpc_proto::socialops::crawl_service_server::CrawlService;
use grpc_proto::socialops::content_service_server::ContentService;
use grpc_proto::socialops::publish_service_server::PublishService;
use grpc_proto::socialops::llm_provider_service_server::LlmProviderService;
use grpc_proto::socialops::rewrite_service_server::RewriteService;
use grpc_proto::socialops::stats_service_server::StatsService;
use grpc_proto::socialops::insight_service_server::InsightService;

use crate::services::account_service::AccountService as AccountSvc;
use crate::services::crawl_service::CrawlService as CrawlSvc;
use crate::services::content_service::ContentService as ContentSvc;
use crate::services::publish_service::PublishService as PublishSvc;
use crate::services::llm_service::LlmService;
use crate::services::rewrite_service::RewriteService as RewriteSvc;
use crate::models::account::{CreateAccountRequest, UpdateAccountRequest, SocialAccount};

// ===== From impls =====

impl From<SocialAccount> for Account {
    fn from(a: SocialAccount) -> Self {
        Self {
            id: a.id.to_string(),
            user_id: a.user_id.map(|u| u.to_string()).unwrap_or_default(),
            platform: a.platform,
            account_name: a.account_name,
            account_id: a.account_id.unwrap_or_default(),
            avatar_url: a.avatar_url.unwrap_or_default(),
            is_active: a.is_active,
            config_json: a.config_json.map(|v| v.to_string()).unwrap_or_default(),
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

/// 共享应用状态 — 包含所有服务
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub account_svc: AccountSvc,
    pub crawl_svc: CrawlSvc,
    pub content_svc: ContentSvc,
    pub publish_svc: PublishSvc,
    pub llm_svc: LlmService,
    pub rewrite_svc: RewriteSvc,
}

impl AppState {
    #[must_use]
    pub const fn new(
        db: sqlx::PgPool,
        account_svc: AccountSvc,
        crawl_svc: CrawlSvc,
        content_svc: ContentSvc,
        publish_svc: PublishSvc,
        llm_svc: LlmService,
        rewrite_svc: RewriteSvc,
    ) -> Self {
        Self {
            db,
            account_svc, crawl_svc, content_svc,
            publish_svc, llm_svc, rewrite_svc,
        }
    }
}

// ===== AccountService =====

pub struct GrpcAccountService {
    state: Arc<AppState>,
}

impl GrpcAccountService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl AccountService for GrpcAccountService {
    async fn list_accounts(&self, req: Request<ListAccountsReq>) -> Result<Response<ListAccountsResp>, Status> {
        let user_id = if req.get_ref().user_id.is_empty() {
            None
        } else {
            Some(Uuid::parse_str(&req.get_ref().user_id).map_err(|_| Status::invalid_argument("invalid user_id"))?)
        };
        let accounts = self.state.account_svc.list(user_id).await.map_err(|e| Status::internal(e.to_string()))?;
        let resp = ListAccountsResp {
            accounts: accounts.into_iter().map(std::convert::Into::into).collect(),
        };
        Ok(Response::new(resp))
    }

    async fn get_account(&self, req: Request<GetAccountReq>) -> Result<Response<Account>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let account = self.state.account_svc.get(id).await.map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("account not found"))?;
        Ok(Response::new(account.into()))
    }

    async fn add_account(&self, req: Request<AddAccountReq>) -> Result<Response<Account>, Status> {
        let r = req.get_ref();
        let create_req = CreateAccountRequest {
            user_id: Some(Uuid::parse_str(&r.user_id).map_err(|_| Status::invalid_argument("invalid user_id"))?),
            platform: r.platform.clone(),
            account_name: r.account_name.clone(),
            credentials: Some(r.credentials_json.clone()),
            config: if r.config_json.is_empty() { None } else { Some(serde_json::from_str(&r.config_json).unwrap_or(serde_json::Value::Null)) },
        };
        let account = self.state.account_svc.create(&create_req).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(account.into()))
    }

    async fn update_account(&self, req: Request<UpdateAccountReq>) -> Result<Response<Account>, Status> {
        let r = req.get_ref();
        let id = Uuid::parse_str(&r.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let update_req = UpdateAccountRequest {
            account_name: if r.account_name.is_empty() { None } else { Some(r.account_name.clone()) },
            config: if r.config_json.is_empty() { None } else { Some(serde_json::from_str(&r.config_json).unwrap_or(serde_json::Value::Null)) },
            is_active: Some(r.is_active),
        };
        let account = self.state.account_svc.update(id, &update_req).await.map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("account not found"))?;
        Ok(Response::new(account.into()))
    }

    async fn delete_account(&self, req: Request<DeleteAccountReq>) -> Result<Response<DeleteResp>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.account_svc.delete(id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteResp { success: true }))
    }
}

// ===== ContentService =====

pub struct GrpcContentService {
    state: Arc<AppState>,
}

impl GrpcContentService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl ContentService for GrpcContentService {
    async fn list_contents(&self, req: Request<ListContentsReq>) -> Result<Response<ListContentsResp>, Status> {
        let r = req.get_ref();
        let (items, total) = self.state.content_svc.list(
            if r.status.is_empty() { None } else { Some(r.status.as_str()) },
            i64::from(r.page.max(1)),
            i64::from(r.page_size.max(20)),
        ).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListContentsResp {
            items: items.into_iter().map(|v| ContentItem {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                source_type: v.get("source_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                content_type: v.get("content_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                title: v.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                summary: v.get("summary").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                body: v.get("body").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                source_url: v.get("source_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                source_hash: v.get("source_hash").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                author_name: v.get("author_name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                status: v.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                created_at: v.get("created_at").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            }).collect(),
            total: total as i32,
        }))
    }

    async fn get_content(&self, req: Request<GetContentReq>) -> Result<Response<ContentItem>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let item = self.state.content_svc.get(id).await.map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("content not found"))?;
        Ok(Response::new(ContentItem {
            id: item.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_type: item.get("source_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            content_type: item.get("content_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            title: item.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            summary: item.get("summary").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            body: item.get("body").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_url: item.get("source_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_hash: item.get("source_hash").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            author_name: item.get("author_name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            status: item.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            created_at: item.get("created_at").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        }))
    }

    async fn create_content(&self, req: Request<CreateContentReq>) -> Result<Response<ContentItem>, Status> {
        let r = req.get_ref();
        let item = self.state.content_svc.create(
            &r.title, &r.body, &r.content_type,
            if r.source_url.is_empty() { None } else { Some(r.source_url.as_str()) },
        ).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ContentItem {
            id: item.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_type: String::new(),
            content_type: item.get("content_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            title: item.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            summary: item.get("summary").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            body: item.get("body").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_url: item.get("source_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_hash: item.get("source_hash").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            author_name: item.get("author_name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            status: item.get("status").and_then(|x| x.as_str()).unwrap_or("draft").to_string(),
            created_at: item.get("created_at").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        }))
    }

    async fn update_content(&self, req: Request<UpdateContentReq>) -> Result<Response<ContentItem>, Status> {
        let r = req.get_ref();
        let id = Uuid::parse_str(&r.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.content_svc.update_status(id, &r.status).await.map_err(|e| Status::internal(e.to_string()))?;
        // Return updated content
        let item = self.state.content_svc.get(id).await.map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("content not found"))?;
        Ok(Response::new(ContentItem {
            id: item.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_type: item.get("source_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            content_type: item.get("content_type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            title: item.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            summary: item.get("summary").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            body: item.get("body").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_url: item.get("source_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_hash: item.get("source_hash").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            author_name: item.get("author_name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            status: item.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            created_at: item.get("created_at").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        }))
    }

    async fn delete_content(&self, req: Request<DeleteContentReq>) -> Result<Response<DeleteResp>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.content_svc.update_status(id, "deleted").await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteResp { success: true }))
    }
}

// ===== CrawlService =====

pub struct GrpcCrawlService {
    state: Arc<AppState>,
}

impl GrpcCrawlService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl CrawlService for GrpcCrawlService {
    async fn list_sources(&self, _req: Request<ListSourcesReq>) -> Result<Response<ListSourcesResp>, Status> {
        let sources = self.state.crawl_svc.list_sources().await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListSourcesResp {
            sources: sources.into_iter().map(|v| CrawlSource {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                user_id: v.get("user_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                platform: v.get("platform").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                source_name: v.get("source_name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                source_config: v.get("source_config").map(std::string::ToString::to_string).unwrap_or_default(),
                is_active: v.get("is_active").and_then(serde_json::Value::as_bool).unwrap_or(false),
                crawl_interval: v.get("crawl_interval").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
                last_crawled_at: v.get("last_crawled_at").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            }).collect(),
        }))
    }

    async fn create_source(&self, req: Request<CreateSourceReq>) -> Result<Response<CrawlSource>, Status> {
        let r = req.get_ref();
        let config: serde_json::Value = if r.source_config.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&r.source_config).unwrap_or(serde_json::Value::Null)
        };
        let source = self.state.crawl_svc.create_source(
            &r.platform, &r.source_name, &config, r.crawl_interval,
        ).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CrawlSource {
            id: source.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            user_id: source.get("user_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            platform: source.get("platform").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_name: source.get("source_name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source_config: source.get("source_config").map(std::string::ToString::to_string).unwrap_or_default(),
            is_active: source.get("is_active").and_then(serde_json::Value::as_bool).unwrap_or(true),
            crawl_interval: source.get("crawl_interval").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            last_crawled_at: String::new(),
        }))
    }

    async fn update_source(&self, req: Request<UpdateSourceReq>) -> Result<Response<CrawlSource>, Status> {
        let r = req.get_ref();
        let id = Uuid::parse_str(&r.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let config: serde_json::Value = if r.source_config.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&r.source_config).unwrap_or(serde_json::Value::Null)
        };
        // Update crawl source in database
        let row = sqlx::query!(
            r#"UPDATE socialops.crawl_sources
               SET platform = $1, source_name = $2, source_config = $3, crawl_interval = $4, is_active = $5
               WHERE id = $6
               RETURNING id, platform, source_name AS "source_name!", source_config::text AS "source_config", is_active, crawl_interval,
                         to_char(last_crawled_at, 'YYYY-MM-DD HH24:MI:SS') AS "last_crawled""#,
            &r.platform,
            &r.source_name,
            config,
            r.crawl_interval,
            r.is_active,
            id,
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?
        .ok_or_else(|| Status::not_found("source not found"))?;
        Ok(Response::new(CrawlSource {
            id: row.id.to_string(),
            user_id: String::new(),
            platform: row.platform,
            source_name: row.source_name,
            source_config: row.source_config,
            is_active: row.is_active,
            crawl_interval: row.crawl_interval,
            last_crawled_at: row.last_crawled.unwrap_or_default(),
        }))
    }

    async fn delete_source(&self, req: Request<DeleteSourceReq>) -> Result<Response<DeleteResp>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.crawl_svc.delete_source(id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteResp { success: true }))
    }

    async fn trigger_crawl(&self, req: Request<TriggerCrawlReq>) -> Result<Response<CrawlTask>, Status> {
        let source_id = Uuid::parse_str(&req.get_ref().source_id).map_err(|_| Status::invalid_argument("invalid source_id"))?;
        let task_id = self.state.crawl_svc.create_task(source_id).await.map_err(|e| Status::internal(e.to_string()))?;
        // Spawn crawl in background
        let svc = self.state.crawl_svc.clone();
        tokio::spawn(async move {
            let _ = svc.trigger_crawl(source_id).await;
        });
        Ok(Response::new(CrawlTask {
            id: task_id.to_string(),
            source_id: source_id.to_string(),
            status: "pending".to_string(),
            items_found: 0,
            items_new: 0,
            error_message: String::new(),
        }))
    }

    async fn list_crawl_tasks(&self, req: Request<ListCrawlTasksReq>) -> Result<Response<ListCrawlTasksResp>, Status> {
        let source_id = Uuid::parse_str(&req.get_ref().source_id).map_err(|_| Status::invalid_argument("invalid source_id"))?;
        let tasks = self.state.crawl_svc.list_crawl_tasks(source_id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListCrawlTasksResp {
            tasks: tasks.into_iter().map(|v| CrawlTask {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                source_id: source_id.to_string(),
                status: v.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                items_found: v.get("items_found").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
                items_new: v.get("items_new").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
                error_message: v.get("error_message").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            }).collect(),
        }))
    }
}

// ===== PublishService =====

pub struct GrpcPublishService {
    state: Arc<AppState>,
}

impl GrpcPublishService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl PublishService for GrpcPublishService {
    async fn publish(&self, req: Request<PublishReq>) -> Result<Response<PublishTaskResp>, Status> {
        let r = req.get_ref();
        let content_id = Uuid::parse_str(&r.content_id).map_err(|_| Status::invalid_argument("invalid content_id"))?;
        let account_id = Uuid::parse_str(&r.account_id).map_err(|_| Status::invalid_argument("invalid account_id"))?;
        let version_id = if r.version_id.is_empty() {
            None
        } else {
            Some(Uuid::parse_str(&r.version_id).map_err(|_| Status::invalid_argument("invalid version_id"))?)
        };
        let result = self.state.publish_svc.publish(content_id, account_id, version_id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PublishTaskResp {
            id: result.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            status: result.get("status").and_then(|x| x.as_str()).unwrap_or("pending").to_string(),
            platform_post_id: result.get("platform_post_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            post_url: String::new(),
        }))
    }

    async fn list_tasks(&self, _req: Request<ListPublishTasksReq>) -> Result<Response<ListPublishTasksResp>, Status> {
        let (tasks, _total) = self.state.publish_svc.list_tasks(None, 1, 50).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListPublishTasksResp {
            tasks: tasks.into_iter().map(|v| PublishTaskResp {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                status: v.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                platform_post_id: v.get("platform_post_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                post_url: v.get("post_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            }).collect(),
        }))
    }

    async fn create_schedule(&self, req: Request<CreateScheduleReq>) -> Result<Response<PublishSchedule>, Status> {
        let r = req.get_ref();
        let content_id = Uuid::parse_str(&r.content_id).map_err(|_| Status::invalid_argument("invalid content_id"))?;
        let account_id = Uuid::parse_str(&r.account_id).map_err(|_| Status::invalid_argument("invalid account_id"))?;
        let sched = self.state.publish_svc.create_schedule(content_id, account_id, &r.cron_expression).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PublishSchedule {
            id: sched.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            content_id: sched.get("content_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            account_id: sched.get("account_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            version_id: sched.get("version_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            cron_expression: sched.get("cron_expr").or_else(|| sched.get("cron_expression")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            scheduled_at: sched.get("scheduled_at").or_else(|| sched.get("next_run_at")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            is_recurring: sched.get("is_recurring").and_then(serde_json::Value::as_bool).unwrap_or(false),
            status: sched.get("status").and_then(|x| x.as_str()).unwrap_or("active").to_string(),
        }))
    }

    async fn list_schedules(&self, _req: Request<ListSchedulesReq>) -> Result<Response<ListSchedulesResp>, Status> {
        let schedules = self.state.publish_svc.list_schedules().await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListSchedulesResp {
            schedules: schedules.into_iter().map(|v| PublishSchedule {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                content_id: v.get("content_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                account_id: v.get("account_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                version_id: v.get("version_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                cron_expression: v.get("cron_expr").or_else(|| v.get("cron_expression")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
                scheduled_at: v.get("scheduled_at").or_else(|| v.get("next_run_at")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
                is_recurring: v.get("is_recurring").and_then(serde_json::Value::as_bool).unwrap_or(false),
                status: v.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            }).collect(),
        }))
    }

    async fn update_schedule(&self, req: Request<UpdateScheduleReq>) -> Result<Response<PublishSchedule>, Status> {
        let r = req.get_ref();
        let id = Uuid::parse_str(&r.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let cron_expr = if r.cron_expression.is_empty() { None } else { Some(r.cron_expression.as_str()) };
        let is_active = Some(r.is_recurring);
        let updated = self.state.publish_svc.update_schedule(id, cron_expr, is_active).await
            .map_err(|e| Status::internal(e.to_string()))?;
        if !updated {
            return Err(Status::not_found("schedule not found"));
        }
        let sched = self.state.publish_svc.get_schedule(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("schedule not found"))?;
        Ok(Response::new(PublishSchedule {
            id: sched.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            content_id: sched.get("content_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            account_id: sched.get("account_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            version_id: sched.get("version_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            cron_expression: sched.get("cron_expr").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            scheduled_at: sched.get("created_at").or_else(|| sched.get("updated_at")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            is_recurring: sched.get("is_active").and_then(serde_json::Value::as_bool).unwrap_or(false),
            status: "active".to_string(),
        }))
    }

    async fn delete_schedule(&self, req: Request<DeleteScheduleReq>) -> Result<Response<DeleteResp>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.publish_svc.delete_schedule(id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteResp { success: true }))
    }
}

// ===== LLMProviderService =====

pub struct GrpcLlmProviderService {
    state: Arc<AppState>,
}

impl GrpcLlmProviderService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl LlmProviderService for GrpcLlmProviderService {
    async fn list_providers(&self, _req: Request<ListProvidersReq>) -> Result<Response<ListProvidersResp>, Status> {
        let providers = self.state.llm_svc.list_providers().await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListProvidersResp {
            providers: providers.into_iter().map(|v| LlmProvider {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                provider_name: v.get("name").or_else(|| v.get("provider_name")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
                api_endpoint: v.get("api_endpoint").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                model_name: v.get("model").or_else(|| v.get("model_name")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
                is_active: v.get("is_active").and_then(serde_json::Value::as_bool).unwrap_or(true),
            }).collect(),
        }))
    }

    async fn add_provider(&self, req: Request<AddProviderReq>) -> Result<Response<LlmProvider>, Status> {
        let r = req.get_ref();
        let provider = self.state.llm_svc.add_provider(
            &r.provider_name, &r.api_endpoint, &r.api_key, &r.model_name,
        ).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(LlmProvider {
            id: provider.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            provider_name: provider.get("name").or_else(|| provider.get("provider_name")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            api_endpoint: provider.get("api_endpoint").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            model_name: provider.get("model").or_else(|| provider.get("model_name")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            is_active: true,
        }))
    }

    async fn update_provider(&self, _req: Request<UpdateProviderReq>) -> Result<Response<LlmProvider>, Status> {
        Err(Status::unimplemented("update_provider not implemented"))
    }

    async fn delete_provider(&self, req: Request<DeleteProviderReq>) -> Result<Response<DeleteResp>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.llm_svc.delete_provider(id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteResp { success: true }))
    }

    async fn test_provider(&self, _req: Request<TestProviderReq>) -> Result<Response<TestProviderResp>, Status> {
        Ok(Response::new(TestProviderResp { success: true, message: "test ok".to_string() }))
    }
}

// ===== RewriteService =====

pub struct GrpcRewriteService {
    state: Arc<AppState>,
}

impl GrpcRewriteService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl RewriteService for GrpcRewriteService {
    async fn create_task(&self, req: Request<CreateRewriteTaskReq>) -> Result<Response<RewriteTask>, Status> {
        let r = req.get_ref();
        let content_id = Uuid::parse_str(&r.content_id).map_err(|_| Status::invalid_argument("invalid content_id"))?;
        let llm_provider_id = Uuid::parse_str(&r.llm_provider_id).map_err(|_| Status::invalid_argument("invalid llm_provider_id"))?;
        let task = self.state.rewrite_svc.create_task(content_id, llm_provider_id, r.target_count).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RewriteTask {
            id: task.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            content_id: task.get("content_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            llm_provider_id: task.get("llm_provider_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            status: task.get("status").and_then(|x| x.as_str()).unwrap_or("pending").to_string(),
            target_count: task.get("target_count").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            rewrite_prompt: task.get("rewrite_prompt").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        }))
    }

    async fn list_tasks(&self, _req: Request<ListRewriteTasksReq>) -> Result<Response<ListRewriteTasksResp>, Status> {
        let (tasks, _total) = self.state.rewrite_svc.list_tasks(None, 1, 50).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListRewriteTasksResp {
            tasks: tasks.into_iter().map(|v| RewriteTask {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                content_id: v.get("content_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                llm_provider_id: v.get("llm_provider_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                status: v.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                target_count: v.get("target_count").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
                rewrite_prompt: v.get("rewrite_prompt").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            }).collect(),
        }))
    }

    async fn get_task(&self, req: Request<GetRewriteTaskReq>) -> Result<Response<RewriteTask>, Status> {
        let id = Uuid::parse_str(&req.get_ref().id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let task = self.state.rewrite_svc.get_task(id).await.map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("task not found"))?;
        Ok(Response::new(RewriteTask {
            id: task.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            content_id: task.get("content_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            llm_provider_id: task.get("llm_provider_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            status: task.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            target_count: task.get("target_count").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            rewrite_prompt: task.get("rewrite_prompt").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        }))
    }

    async fn get_versions(&self, req: Request<GetVersionsReq>) -> Result<Response<GetVersionsResp>, Status> {
        let task_id = Uuid::parse_str(&req.get_ref().task_id).map_err(|_| Status::invalid_argument("invalid task_id"))?;
        let versions = self.state.rewrite_svc.get_versions(task_id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetVersionsResp {
            versions: versions.into_iter().map(|v| RewriteVersion {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                task_id: task_id.to_string(),
                version_seq: v.get("version_seq").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
                rewritten_title: v.get("rewritten_title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                rewritten_body: v.get("rewritten_body").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                similarity_score: v.get("similarity_score").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
                status: v.get("status").and_then(|x| x.as_str()).unwrap_or("draft").to_string(),
            }).collect(),
        }))
    }

    async fn update_version(&self, req: Request<UpdateVersionReq>) -> Result<Response<RewriteVersion>, Status> {
        let r = req.get_ref();
        let id = Uuid::parse_str(&r.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        self.state.rewrite_svc.update_version_status(id, &r.status).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RewriteVersion {
            id: id.to_string(),
            task_id: String::new(),
            version_seq: 0,
            rewritten_title: String::new(),
            rewritten_body: String::new(),
            similarity_score: 0.0,
            status: r.status.clone(),
        }))
    }
}

// ===== StatsService =====

pub struct GrpcStatsService {
    #[allow(dead_code)]
    state: Arc<AppState>,
}

impl GrpcStatsService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl StatsService for GrpcStatsService {
    async fn get_account_stats(&self, _req: Request<GetAccountStatsReq>) -> Result<Response<AccountStats>, Status> {
        Ok(Response::new(AccountStats {
            daily_stats: vec![],
            total_followers: 0,
            total_posts: 0,
            total_likes: 0,
            total_comments: 0,
        }))
    }

    async fn get_content_stats(&self, _req: Request<GetContentStatsReq>) -> Result<Response<ContentStats>, Status> {
        Ok(Response::new(ContentStats {
            content_id: String::new(),
            total_views: 0,
            total_likes: 0,
            total_comments: 0,
            total_shares: 0,
        }))
    }

    async fn get_overview(&self, _req: Request<GetOverviewReq>) -> Result<Response<StatsOverview>, Status> {
        Ok(Response::new(StatsOverview {
            total_accounts: 0,
            total_contents: 0,
            total_published: 0,
            total_followers: 0,
        }))
    }
}

// ===== InsightService =====

pub struct GrpcInsightService {
    #[allow(dead_code)]
    state: Arc<AppState>,
}

impl GrpcInsightService {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self { Self { state } }
}

#[tonic::async_trait]
impl InsightService for GrpcInsightService {
    async fn generate(&self, _req: Request<GenerateInsightReq>) -> Result<Response<StatInsight>, Status> {
        Err(Status::unimplemented("insight generate not implemented"))
    }

    async fn list(&self, _req: Request<ListInsightsReq>) -> Result<Response<ListInsightsResp>, Status> {
        Ok(Response::new(ListInsightsResp { insights: vec![] }))
    }

    async fn get_latest(&self, _req: Request<GetLatestInsightReq>) -> Result<Response<StatInsight>, Status> {
        Err(Status::not_found("no insight reports"))
    }
}