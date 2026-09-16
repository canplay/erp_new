//! CMS 服务路由 — HTTP → gRPC 转发
//!
//! @date 2026-07-17

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::{json_error_response_fmt, json_success};

#[derive(Debug, Deserialize)]
pub struct ArticleListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub category_id: Option<i64>,
    pub keyword: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryQuery {
    pub parent_id: Option<i64>,
}

// ============ Category Handlers ============

async fn list_categories(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CategoryQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let resp = client.list_categories(q.parent_id.unwrap_or(0), true).await
        .map_err(|e| json_error_response_fmt(StatusCode::INTERNAL_SERVER_ERROR, "获取分类失败: {e}" , &format!("获取分类失败: {e}" )))?;

    let categories: Vec<Value> = resp.categories.into_iter().map(|c| json!({
        "id": c.id,
        "name": c.name,
        "slug": c.slug,
        "description": c.description,
        "parent_id": c.parent_id,
        "sort_order": c.sort_order,
        "icon": c.icon,
        "created_at": c.created_at,
        "updated_at": c.updated_at,
    })).collect();

    Ok(json_success(json!({"data": categories})))
}

async fn get_category_tree(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let resp = client.list_categories(0, true).await
        .map_err(|e| json_error_response_fmt(StatusCode::INTERNAL_SERVER_ERROR, "获取分类树失败: {e}" , &format!("获取分类树失败: {e}" )))?;

    let cats: Vec<Value> = resp.categories.into_iter().map(|c| json!({
        "id": c.id, "name": c.name, "slug": c.slug,
        "description": c.description, "parent_id": c.parent_id,
        "sort_order": c.sort_order, "icon": c.icon,
        "created_at": c.created_at, "updated_at": c.updated_at,
    })).collect();
    Ok(json_success(json!({"data": cats})))
}

async fn create_category(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let resp = client.create_category(
        body.get("name" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        body.get("slug" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        body.get("description" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        body.get("parent_id" ).and_then(|v| v.as_i64()).unwrap_or(0),
        body.get("sort_order" ).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        body.get("icon" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
    ).await.map_err(|e| json_error_response_fmt(StatusCode::BAD_REQUEST, "创建分类失败: {e}" , &format!("创建分类失败: {e}" )))?;

    Ok(json_success(json!({"id": resp.id, "name": resp.name})))
}

async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let resp = client.update_category(
        id,
        body.get("name" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        body.get("slug" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        body.get("description" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        body.get("parent_id" ).and_then(|v| v.as_i64()).unwrap_or(0),
        body.get("sort_order" ).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        body.get("icon" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
    ).await.map_err(|e| json_error_response_fmt(StatusCode::BAD_REQUEST, "更新分类失败: {e}" , &format!("更新分类失败: {e}" )))?;

    Ok(json_success(json!({"id": resp.id, "name": resp.name})))
}

async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    client.delete_category(id, false).await
.map_err(|e| json_error_response_fmt(StatusCode::BAD_REQUEST, "删除分类失败: {e}" , &format!("删除分类失败: {e}" )))?;

    Ok(json_success(json!({"message": "删除成功" })))
}

// ============ Article Handlers ============

async fn list_articles(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ArticleListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let resp = client.list_articles(
        q.page.unwrap_or(1), q.page_size.unwrap_or(20),
        q.category_id.unwrap_or(0), q.keyword.unwrap_or_default(), q.status.unwrap_or_default(),
    ).await.map_err(|e| json_error_response_fmt(StatusCode::INTERNAL_SERVER_ERROR, "获取文章列表失败: {e}" , &format!("获取文章列表失败: {e}" )))?;

    let arts: Vec<Value> = resp.articles.into_iter().map(|a| json!({
        "id": a.id, "title": a.title, "summary": a.summary,
        "content": a.content, "cover_image": a.cover_image,
        "category_id": a.category_id, "author": a.author,
        "status": a.status, "tags": a.tags,
        "view_count": a.view_count,
        "created_at": a.created_at, "updated_at": a.updated_at,
    })).collect();
    Ok(json_success(json!({"list": arts, "total": resp.total})))
}

async fn get_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let resp = client.get_article(id).await
        .map_err(|e| json_error_response_fmt(StatusCode::NOT_FOUND, "获取文章失败: {e}" , &format!("获取文章失败: {e}" )))?;

    let article = resp.article.unwrap_or_default();
    Ok(json_success(json!({
            "id": article.id,
            "title": article.title,
            "summary": article.summary,
            "content": article.content,
            "cover_image": article.cover_image,
            "category_id": article.category_id,
            "author": article.author,
            "status": article.status,
            "tags": article.tags,
            "view_count": article.view_count,
            "created_at": article.created_at,
            "updated_at": article.updated_at,
        })))
}

async fn create_article(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let params = crate::grpc_clients::CreateArticleParams {
        title: body.get("title" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        content: body.get("content" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        summary: body.get("summary" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        cover_image: body.get("cover_image" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        category_id: body.get("category_id" ).and_then(|v| v.as_i64()).unwrap_or(0),
        tags: body.get("tags" ).and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        author: body.get("author" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        status: body.get("status" ).and_then(|v| v.as_str()).unwrap_or("draft" ).to_string(),
    };

    let resp = client.create_article(params).await
.map_err(|e| json_error_response_fmt(StatusCode::BAD_REQUEST, "创建文章失败: {e}" , &format!("创建文章失败: {e}" )))?;

    Ok(json_success(json!({"id": resp.id})))
}

async fn update_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    let params = crate::grpc_clients::UpdateArticleParams {
        id,
        title: body.get("title" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        content: body.get("content" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        summary: body.get("summary" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        cover_image: body.get("cover_image" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        category_id: body.get("category_id" ).and_then(|v| v.as_i64()).unwrap_or(0),
        tags: body.get("tags" ).and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        status: body.get("status" ).and_then(|v| v.as_str()).unwrap_or("draft" ).to_string(),
    };

    let resp = client.update_article(params).await
.map_err(|e| json_error_response_fmt(StatusCode::BAD_REQUEST, "更新文章失败: {e}" , &format!("更新文章失败: {e}" )))?;

    Ok(json_success(json!({"id": resp.id})))
}

async fn delete_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut client = state.grpc_clients.read().await
        .cms_client().await
        .map_err(|e| json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "CMS服务不可用: {e}" , &format!("CMS服务不可用: {e}" )))?;

    client.delete_article(id).await
        .map_err(|e| json_error_response_fmt(StatusCode::BAD_REQUEST, "删除文章失败: {e}" , &format!("删除文章失败: {e}" )))?;

    Ok(json_success(json!({"message": "删除成功" })))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 分类
        .route("/api/cms/categories" , get(list_categories).post(create_category))
        .route("/api/cms/categories/tree" , get(get_category_tree))
        .route("/api/cms/categories/batch" , delete(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "批量删除暂未实现" }))
        }))
        .route("/api/cms/categories/reorder" , put(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "排序暂未实现" }))
        }))
        .route("/api/cms/categories/{id}" , put(update_category).delete(delete_category))
        // 文章
        .route("/api/cms/articles" , get(list_articles).post(create_article))
        .route("/api/cms/articles/{id}" , get(get_article).put(update_article).delete(delete_article))
        .route("/api/cms/articles/batch" , delete(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "批量删除暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/publish" , put(|_: State<Arc<AppState>>, _: Path<i64>| async {
            json_success(json!({"message": "发布功能暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/unpublish" , put(|_: State<Arc<AppState>>, _: Path<i64>| async {
            json_success(json!({"message": "下架功能暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/review" , put(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "审核功能暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/top" , put(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "置顶功能暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/feature" , put(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "推荐功能暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/view" , put(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "浏览量统计暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/like" , post(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "点赞功能暂未实现" }))
        }))
        .route("/api/cms/articles/{id}/favorite" , post(|_: State<Arc<AppState>>| async {
            json_success(json!({"message": "收藏功能暂未实现" }))
        }))
        .route("/api/cms/articles/slug/{slug}" , get(|_: State<Arc<AppState>>, _: Path<String>| async {
            json_success(json!({"message": "暂不支持slug查询" }))
        }))
        // 其他 CMS 辅助接口
        .route("/api/cms/articles/statistics" , get(|_: State<Arc<AppState>>| async {
            json_success(json!({"total": 0, "published": 0, "draft": 0}))
        }))
        .route("/api/cms/articles/hot" , get(list_articles))
        .route("/api/cms/articles/latest" , get(list_articles))
        .route("/api/cms/articles/my-drafts" , get(list_articles))
        .route("/api/cms/articles/{id}/related" , get(list_articles))
}
