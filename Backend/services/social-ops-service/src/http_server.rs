//! Social Ops Service HTTP API
//!
//! # 功能
//!
//! 社交运营服务，提供账号管理、内容管理和爬虫源管理功能。
//!
//! # API 端点
//!
//! ## 账号管理
//! - `GET    /api/v1/social-ops/accounts` — 获取账号列表
//! - `POST   /api/v1/social-ops/accounts` — 创建账号
//! - `GET    /api/v1/social-ops/accounts/:id` — 获取单个账号
//! - `PUT    /api/v1/social-ops/accounts/:id` — 更新账号
//! - `DELETE /api/v1/social-ops/accounts/:id` — 删除账号
//!
//! ## 内容管理
//! - `GET  /api/v1/social-ops/contents` — 获取内容列表
//! - `GET  /api/v1/social-ops/contents/:id` — 获取单个内容
//! - `PUT  /api/v1/social-ops/contents/:id` — 更新内容状态
//! - `POST /api/v1/social-ops/contents/manual` — 手动创建内容
//!
//! ## 爬虫源管理
//! - `POST   /api/v1/social-ops/crawl/trigger/:source_id` — 触发爬虫
//! - `GET    /api/v1/social-ops/crawl/sources` — 获取爬虫源列表
//! - `POST   /api/v1/social-ops/crawl/sources` — 创建爬虫源
//! - `DELETE /api/v1/social-ops/crawl/sources/:id` — 删除爬虫源

use std::sync::Arc;

use axum::{
    Router, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::account::{CreateAccountRequest, UpdateAccountRequest};
use crate::services::account_service::AccountService;
use crate::services::content_service::ContentService;
use crate::services::crawl_service::CrawlService;
use crate::helpers::{
    json_success, json_ok, json_error,
    json_social_accounts, json_social_created, json_social_status, json_social_ok,
    json_social_sources, json_social_source, json_social_items,
};

#[derive(Clone)]
pub struct HttpAppState {
    pub account_service: AccountService,
    pub content_service: ContentService,
    pub crawl_service: CrawlService,
}

impl HttpAppState {
    #[must_use]
    pub const fn new(
        account_service: AccountService,
        content_service: ContentService,
        crawl_service: CrawlService,
    ) -> Self {
        Self {
            account_service,
            content_service,
            crawl_service,
        }
    }
}

// ===== Account Handler DTOs =====

#[derive(Deserialize)]
pub struct AccountQuery {
    pub user_id: Option<Uuid>,
}

// ===== Content Handler DTOs =====

#[derive(Deserialize)]
pub struct ContentQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateContentBody {
    pub status: String,
}

#[derive(Deserialize)]
pub struct CreateContentBody {
    pub title: String,
    pub body: String,
    pub content_type: String,
    pub source_url: Option<String>,
}

// ===== Account Handlers =====

async fn list_accounts(
    State(state): State<Arc<HttpAppState>>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let accounts = state
        .account_service
        .list(query.user_id)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(json_social_accounts(&accounts))
}

async fn add_account(
    State(state): State<Arc<HttpAppState>>,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let account = state
        .account_service
        .create(&body)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(json_social_created(account.id))
}

async fn get_account(
    State(state): State<Arc<HttpAppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let account = state
        .account_service
        .get(id)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    match account {
        Some(a) => Ok(json_success(serde_json::json!(a))),
        None => Err(error_response(StatusCode::NOT_FOUND, "Account not found")),
    }
}

async fn update_account(
    State(state): State<Arc<HttpAppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAccountRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let account = state
        .account_service
        .update(id, &body)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    match account {
        Some(_) => Ok(json_social_status("updated")),
        None => Err(error_response(StatusCode::NOT_FOUND, "Account not found")),
    }
}

async fn delete_account(
    State(state): State<Arc<HttpAppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let deleted = state
        .account_service
        .delete(id)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    if deleted {
        Ok(json_ok())
    } else {
        Err(error_response(StatusCode::NOT_FOUND, "Account not found"))
    }
}

// ===== Content Handlers =====

async fn list_contents(
    State(state): State<Arc<HttpAppState>>,
    Query(query): Query<ContentQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let (items, total) = state
        .content_service
        .list(query.status.as_deref(), page, page_size)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(json_social_items(&items, total))
}

async fn get_content(
    State(state): State<Arc<HttpAppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let content = state
        .content_service
        .get(id)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    match content {
        Some(c) => Ok(Json(c)),
        None => Err(error_response(StatusCode::NOT_FOUND, "Content not found")),
    }
}

async fn update_content(
    State(state): State<Arc<HttpAppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateContentBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let updated = state
        .content_service
        .update_status(id, &body.status)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    if updated {
        Ok(json_social_status("updated"))
    } else {
        Err(error_response(StatusCode::NOT_FOUND, "Content not found"))
    }
}

async fn create_content_manual(
    State(state): State<Arc<HttpAppState>>,
    Json(body): Json<CreateContentBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = state
        .content_service
        .create(&body.title, &body.body, &body.content_type, body.source_url.as_deref())
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let id = result["id"].as_str().unwrap_or_default();
    Ok(json_social_created(id))
}

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<Value>) {
    (status, json_error(&format!("{}", message)))
}

// ===== Crawl Handlers =====

#[derive(Deserialize)]
pub struct CreateSourceBody {
    pub platform: String,
    pub name: String,
    pub keyword: Option<String>,
    pub url: Option<String>,
    pub crawl_interval: Option<i32>,
}

async fn trigger_crawl(
    State(state): State<Arc<HttpAppState>>,
    Path(source_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.crawl_service.trigger_crawl(source_id).await {
        Ok(msg) => Ok(json_social_ok(&msg)),
        Err(e) => Err(error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

async fn list_sources(
    State(state): State<Arc<HttpAppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.crawl_service.list_sources().await {
        Ok(sources) => Ok(json_social_sources(&sources)),
        Err(e) => Err(error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

async fn create_source(
    State(state): State<Arc<HttpAppState>>,
    Json(body): Json<CreateSourceBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut config = serde_json::json!({});
    if let Some(ref kw) = body.keyword {
        config["keyword"] = serde_json::json!(kw);
    }
    if let Some(ref url) = body.url {
        config["url"] = serde_json::json!(url);
    }
    let interval = body.crawl_interval.unwrap_or(3600);

    match state.crawl_service.create_source(&body.platform, &body.name, &config, interval).await {
        Ok(source) => Ok(json_social_source(&source)),
        Err(e) => Err(error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

async fn delete_source(
    State(state): State<Arc<HttpAppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.crawl_service.delete_source(id).await {
        Ok(true) => Ok(json_social_status("deleted")),
        Ok(false) => Err(error_response(StatusCode::NOT_FOUND, "Source not found")),
        Err(e) => Err(error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

pub fn routes() -> Router<Arc<HttpAppState>> {
    Router::new()
        // Accounts
        .route("/api/v1/social-ops/accounts", get(list_accounts))
        .route("/api/v1/social-ops/accounts", post(add_account))
        .route("/api/v1/social-ops/accounts/:id", get(get_account))
        .route("/api/v1/social-ops/accounts/:id", put(update_account))
        .route("/api/v1/social-ops/accounts/:id", delete(delete_account))
        // Contents
        .route("/api/v1/social-ops/contents", get(list_contents))
        .route("/api/v1/social-ops/contents/:id", get(get_content))
        .route("/api/v1/social-ops/contents/:id", put(update_content))
        .route("/api/v1/social-ops/contents/manual", post(create_content_manual))
        // Crawl
        .route("/api/v1/social-ops/crawl/trigger/:source_id", post(trigger_crawl))
        .route("/api/v1/social-ops/crawl/sources", get(list_sources))
        .route("/api/v1/social-ops/crawl/sources", post(create_source))
        .route("/api/v1/social-ops/crawl/sources/:id", delete(delete_source))
}
