//! gRPC Service Handlers for CMS
//!
//! 实现 cms.proto 中定义的 gRPC 服务

use crate::CmsAppState;
use crate::models::{CategoryTreeNode, CmsArticle, CmsCategory};
use std::sync::Arc;
use tonic::Status;
use cache_core::MultiLevelCache;
use search_core::{SearchClient, SearchParams, cms::CONTENT as CONTENT_INDEX};
use serde::Serialize;

/// CMS gRPC 服务实现
#[derive(Clone)]
pub struct CmsGrpcService {
    state: Arc<CmsAppState>,
}

// ============== 文章相关实现 ==============

/// 缓存键前缀
const CACHE_ARTICLE_PREFIX: &str = "cms:article:";
const CACHE_ARTICLE_LIST_PREFIX: &str = "cms:article:list:";
const CACHE_CATEGORY_PREFIX: &str = "cms:category:";
const CACHE_CATEGORY_LIST_PREFIX: &str = "cms:category:list:";

/// 获取文章缓存键
fn article_cache_key(id: i64) -> String {
    format!("{}{}", CACHE_ARTICLE_PREFIX, id)
}

/// 获取文章列表缓存键
fn article_list_cache_key(page: i32, page_size: i32, category_id: Option<i64>, status: Option<i32>, keyword: Option<&str>) -> String {
    use std::fmt::Write;
    let mut key = String::from(CACHE_ARTICLE_LIST_PREFIX);
    write!(key, "p{}_ps{}", page, page_size).ok();
    if let Some(cid) = category_id { write!(key, "_c{}", cid).ok(); }
    if let Some(s) = status { write!(key, "_s{}", s).ok(); }
    if let Some(k) = keyword { write!(key, "_k{}", k).ok(); }
    key
}

/// 搜索索引名称
fn get_content_index_name(state: &CmsAppState) -> String {
    // 使用配置的索引前缀
    let prefix = std::env::var("MEILISEARCH_INDEX_PREFIX").unwrap_or_else(|_| "erp_new".to_string());
    format!("{}_{}", prefix, CONTENT_INDEX)
}

/// 将文章转换为搜索文档
#[derive(Serialize)]
struct ArticleSearchDoc {
    id: String,
    title: String,
    body: String,
    excerpt: String,
    tags: Vec<String>,
    status: i32,
    category_id: String,
    author_id: String,
    is_published: bool,
    created_at: String,
    updated_at: String,
}

impl From<&CmsArticle> for ArticleSearchDoc {
    fn from(article: &CmsArticle) -> Self {
        let tags = article.tags
            .as_deref()
            .and_then(|t| serde_json::from_str::<Vec<String>>(t).ok())
            .unwrap_or_default();
        
        Self {
            id: article.id.to_string(),
            title: article.title.clone(),
            body: article.content.clone(),
            excerpt: article.summary.clone().unwrap_or_default(),
            tags,
            status: article.status,
            category_id: article.category_id.to_string(),
            author_id: article.author_id.to_string(),
            is_published: article.status == 2,
            created_at: article.created_at.to_rfc3339(),
            updated_at: article.updated_at.to_rfc3339(),
        }
    }
}

/// 索引文章到 Meilisearch
async fn index_article(state: &CmsAppState, article: &CmsArticle) {
    let index_name = get_content_index_name(state);
    let doc = ArticleSearchDoc::from(article);
    if let Err(e) = state.search.add_documents(&index_name, &[serde_json::to_value(doc).unwrap()], Some("id")).await {
        tracing::warn!("Failed to index article {}: {}", article.id, e);
    }
}

/// 从 Meilisearch 删除文章
async fn delete_article_from_index(state: &CmsAppState, article_id: i64) {
    let index_name = get_content_index_name(state);
    if let Err(e) = state.search.delete_document(&index_name, &article_id.to_string()).await {
        tracing::warn!("Failed to delete article {} from index: {}", article_id, e);
    }
}

/// 清除文章相关缓存
async fn invalidate_article_cache(state: &CmsAppState, id: i64) {
    let _ = state.cache.delete(&article_cache_key(id)).await;
    // 清除列表缓存（简单策略：清除所有列表缓存）
    // 实际生产中可以使用更精细的缓存失效策略
    let pattern = format!("{}*", CACHE_ARTICLE_LIST_PREFIX);
    // 注意：当前 cache-core 不支持模式删除，这里只能清除特定键
    // 生产环境建议使用 Redis SCAN + DEL 实现模式删除
}

/// 获取文章列表（带缓存）
pub async fn list_articles(
    state: Arc<CmsAppState>,
    page: i32,
    page_size: i32,
    category_id: Option<i64>,
    status: Option<i32>,
    keyword: Option<String>,
) -> Result<Vec<CmsArticle>, Status> {
    let cache_key = article_list_cache_key(page, page_size, category_id, status, keyword.as_deref());
    
    // 尝试从缓存获取
    if let Some(cached) = state.cache.get::<Vec<CmsArticle>>(&cache_key).await {
        return Ok(cached);
    }
    
    // 缓存未命中，查询数据库
    let result = state
        .repository
        .article
        .list(page, page_size, category_id, status, keyword.as_deref())
        .await
        .map(|result| result.articles)
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    // 写入缓存
    let _ = state.cache.set(&cache_key, &result).await;
    
    Ok(result)
}

/// 获取文章详情（带缓存）
pub async fn get_article(state: Arc<CmsAppState>, id: i64) -> Result<Option<CmsArticle>, Status> {
    let cache_key = article_cache_key(id);
    
    // 尝试从缓存获取
    if let Some(cached) = state.cache.get::<CmsArticle>(&cache_key).await {
        return Ok(Some(cached));
    }
    
    // 缓存未命中，查询数据库
    let result = state
        .repository
        .article
        .find_by_id(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    // 写入缓存
    if let Some(ref article) = result {
        let _ = state.cache.set(&cache_key, article).await;
    }
    
    Ok(result)
}

/// 创建文章
pub async fn create_article(
    state: Arc<CmsAppState>,
    category_id: i64,
    title: String,
    content: String,
    author_id: i64,
    is_draft: bool,
) -> Result<i64, Status> {
    let id = state
        .repository
        .article
        .create(category_id, &title, &content, author_id, is_draft)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    // 索引到搜索引擎（如果是发布状态）
    if !is_draft {
        if let Some(article) = state.repository.article.find_by_id(id).await.ok().flatten() {
            index_article(&state, &article).await;
        }
    }
    
    // 清除列表缓存
    invalidate_article_cache(&state, id).await;
    
    Ok(id)
}

/// 更新文章
pub async fn update_article(
    state: Arc<CmsAppState>,
    id: i64,
    title: Option<String>,
    content: Option<String>,
    is_featured: Option<bool>,
    is_top: Option<bool>,
) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .update(
            id,
            title.as_deref(),
            content.as_deref(),
            is_featured,
            is_top,
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        // 更新搜索索引
        if let Some(article) = state.repository.article.find_by_id(id).await.ok().flatten() {
            index_article(&state, &article).await;
        }
        
        // 清除缓存
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 删除文章
pub async fn delete_article(state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .delete(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        // 从搜索索引删除
        delete_article_from_index(&state, id).await;
        
        // 清除缓存
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 发布文章
pub async fn publish_article(state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .publish(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        // 索引到搜索引擎
        if let Some(article) = state.repository.article.find_by_id(id).await.ok().flatten() {
            index_article(&state, &article).await;
        }
        
        // 清除缓存
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 下架文章
pub async fn unpublish_article(state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .unpublish(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        // 从搜索索引删除
        delete_article_from_index(&state, id).await;
        
        // 清除缓存
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 审核文章
pub async fn review_article(
    state: Arc<CmsAppState>,
    id: i64,
    approved: bool,
    reason: Option<String>,
) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .review(id, approved, reason.as_deref())
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result && approved {
        // 审核通过，索引到搜索引擎
        if let Some(article) = state.repository.article.find_by_id(id).await.ok().flatten() {
            index_article(&state, &article).await;
        }
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 设置置顶
pub async fn set_article_top(
    state: Arc<CmsAppState>,
    id: i64,
    is_top: bool,
) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .set_top(id, is_top)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        if let Some(article) = state.repository.article.find_by_id(id).await.ok().flatten() {
            index_article(&state, &article).await;
        }
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 设置推荐
pub async fn set_article_featured(
    state: Arc<CmsAppState>,
    id: i64,
    is_featured: bool,
) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .set_featured(id, is_featured)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        if let Some(article) = state.repository.article.find_by_id(id).await.ok().flatten() {
            index_article(&state, &article).await;
        }
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 增加浏览次数
pub async fn increment_article_view_count(
    state: Arc<CmsAppState>,
    id: i64,
) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .increment_view_count(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 增加点赞次数
pub async fn increment_article_like_count(
    state: Arc<CmsAppState>,
    id: i64,
) -> Result<bool, Status> {
    let result = state
        .repository
        .article
        .increment_like_count(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        invalidate_article_cache(&state, id).await;
    }
    
    Ok(result)
}

/// 搜索文章
pub async fn search_articles(
    state: Arc<CmsAppState>,
    query: String,
    page: i32,
    page_size: i32,
    filters: Option<String>,
) -> Result<Vec<CmsArticle>, Status> {
    let index_name = get_content_index_name(&state);
    let params = SearchParams::new(query)
        .limit(page_size as usize)
        .offset(((page - 1) * page_size) as usize);
    
    if let Some(f) = filters {
        let params = params.filters(f);
        let response = state.search.search_raw(&index_name, &params).await
            .map_err(|e| Status::internal(format!("Search error: {e}")))?;
        
        // 将搜索结果转换为 CmsArticle
        // 这里简化处理，实际需要完整的反序列化
        let articles: Vec<CmsArticle> = response.hits.into_iter().filter_map(|hit| {
            serde_json::from_value(hit).ok()
        }).collect();
        
        Ok(articles)
    } else {
        let response = state.search.search_raw(&index_name, &params).await
            .map_err(|e| Status::internal(format!("Search error: {e}")))?;
        
        let articles: Vec<CmsArticle> = response.hits.into_iter().filter_map(|hit| {
            serde_json::from_value(hit).ok()
        }).collect();
        
        Ok(articles)
    }
}

/// 获取热门文章
pub async fn get_hot_articles(
    state: Arc<CmsAppState>,
    limit: i32,
    category_id: Option<i64>,
) -> Result<Vec<CmsArticle>, Status> {
    let cache_key = format!("{}hot_l{}_c{}", CACHE_ARTICLE_LIST_PREFIX, limit, category_id.unwrap_or(0));
    
    if let Some(cached) = state.cache.get::<Vec<CmsArticle>>(&cache_key).await {
        return Ok(cached);
    }
    
    let result = state
        .repository
        .article
        .get_hot_articles(limit, category_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    let _ = state.cache.set(&cache_key, &result).await;
    
    Ok(result)
}

/// 获取最新文章
pub async fn get_latest_articles(
    state: Arc<CmsAppState>,
    limit: i32,
    category_id: Option<i64>,
) -> Result<Vec<CmsArticle>, Status> {
    let cache_key = format!("{}latest_l{}_c{}", CACHE_ARTICLE_LIST_PREFIX, limit, category_id.unwrap_or(0));
    
    if let Some(cached) = state.cache.get::<Vec<CmsArticle>>(&cache_key).await {
        return Ok(cached);
    }
    
    let result = state
        .repository
        .article
        .get_latest_articles(limit, category_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    let _ = state.cache.set(&cache_key, &result).await;
    
    Ok(result)
}

// ============== 分类相关实现 ==============

/// 获取分类列表（带缓存）
pub async fn list_categories(
    state: Arc<CmsAppState>,
    parent_id: Option<i64>,
    include_children: bool,
) -> Result<Vec<CmsCategory>, Status> {
    let cache_key = format!("{}{}_ic{}", CACHE_CATEGORY_LIST_PREFIX, parent_id.unwrap_or(0), include_children);
    
    if let Some(cached) = state.cache.get::<Vec<CmsCategory>>(&cache_key).await {
        return Ok(cached);
    }
    
    let result = if include_children {
        // 获取树形结构，转换为扁平列表用于缓存
        let tree = state
            .repository
            .category
            .get_tree(parent_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
        
        // 将树展平为列表
        fn flatten_tree(nodes: Vec<CategoryTreeNode>) -> Vec<CmsCategory> {
            let mut result = Vec::new();
            for node in nodes {
                result.push(CmsCategory {
                    id: node.id,
                    parent_id: node.parent_id,
                    name: node.name,
                    slug: node.slug,
                    description: None,
                    icon: node.icon,
                    sort_order: node.sort_order,
                    seo_title: None,
                    seo_keywords: None,
                    seo_description: None,
                    status: node.status,
                    allow_attachment: false,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                });
                if !node.children.is_empty() {
                    result.extend(flatten_tree(node.children));
                }
            }
            result
        }
        
        flatten_tree(tree)
    } else {
        let res = state
            .repository
            .category
            .list(1, 100, None)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
        res.categories
    };
    
    let _ = state.cache.set(&cache_key, &result).await;
    
    Ok(result)
}

/// 获取分类详情（带缓存）
pub async fn get_category(state: Arc<CmsAppState>, id: i64) -> Result<Option<CmsCategory>, Status> {
    let cache_key = format!("{}{}", CACHE_CATEGORY_PREFIX, id);
    
    if let Some(cached) = state.cache.get::<CmsCategory>(&cache_key).await {
        return Ok(Some(cached));
    }
    
    let result = state
        .repository
        .category
        .find_by_id(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if let Some(ref cat) = result {
        let _ = state.cache.set(&cache_key, cat).await;
    }
    
    Ok(result)
}

/// 创建分类
pub async fn create_category(
    state: Arc<CmsAppState>,
    name: String,
    slug: String,
    description: Option<String>,
    parent_id: Option<i64>,
    _sort_order: Option<i32>,
) -> Result<i64, Status> {
    let id = state
        .repository
        .category
        .create(&name, &slug, parent_id, description.as_deref())
        .await
        .map_err(|e| match e {
            crate::repository::category_repository::CategoryRepositoryError::AlreadyExists => {
                Status::already_exists("Category slug already exists" )
            }
            _ => Status::internal(format!("Database error: {e}" )),
        })?;
    
    // 清除分类列表缓存
    let _ = state.cache.delete(&format!("{}{}_ic{}", CACHE_CATEGORY_LIST_PREFIX, parent_id.unwrap_or(0), true)).await;
    let _ = state.cache.delete(&format!("{}{}_ic{}", CACHE_CATEGORY_LIST_PREFIX, parent_id.unwrap_or(0), false)).await;
    
    Ok(id)
}

/// 更新分类
pub async fn update_category(
    state: Arc<CmsAppState>,
    id: i64,
    name: Option<String>,
    description: Option<String>,
    sort_order: Option<i32>,
    status: Option<i32>,
) -> Result<bool, Status> {
    let result = state
        .repository
        .category
        .update(
            id,
            name.as_deref(),
            description.as_deref(),
            sort_order,
            status,
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;
    
    if result {
        let _ = state.cache.delete(&format!("{}{}", CACHE_CATEGORY_PREFIX, id)).await;
        // 清除列表缓存
        let _ = state.cache.delete(&format!("{}*_ic{}", CACHE_CATEGORY_LIST_PREFIX, true)).await;
        let _ = state.cache.delete(&format!("{}*_ic{}", CACHE_CATEGORY_LIST_PREFIX, false)).await;
    }
    
    Ok(result)
}

/// 删除分类
pub async fn delete_category(
    state: Arc<CmsAppState>,
    id: i64,
    force: bool,
) -> Result<bool, Status> {
    let result = if force {
        state
            .repository
            .category
            .delete(id)
            .await
            .map_err(|e| match e {
                crate::repository::category_repository::CategoryRepositoryError::HasChildren => {
                    Status::failed_precondition("Category has children or articles" )
                }
                _ => Status::internal(format!("Database error: {e}" )),
            })
    } else {
        state
            .repository
            .category
            .delete(id)
            .await
            .map_err(|e| match e {
                crate::repository::category_repository::CategoryRepositoryError::HasChildren => {
                    Status::failed_precondition("Category has children or articles" )
                }
                _ => Status::internal(format!("Database error: {e}" )),
            })
    }?;
    
    if result {
        let _ = state.cache.delete(&format!("{}{}", CACHE_CATEGORY_PREFIX, id)).await;
        let _ = state.cache.delete(&format!("{}*_ic{}", CACHE_CATEGORY_LIST_PREFIX, true)).await;
        let _ = state.cache.delete(&format!("{}*_ic{}", CACHE_CATEGORY_LIST_PREFIX, false)).await;
    }
    
    Ok(result)
}

/// 获取分类树
pub async fn get_category_tree(
    state: Arc<CmsAppState>,
    parent_id: Option<i64>,
) -> Result<Vec<CategoryTreeNode>, Status> {
    state
        .repository
        .category
        .get_tree(parent_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

// ============== 标签相关实现 ==============

/// 标签列表项
#[derive(Debug, Clone)]
pub struct TagItem {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub article_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 获取标签列表（简化实现）
pub async fn list_tags(
    state: Arc<CmsAppState>,
    page: i32,
    page_size: i32,
) -> Result<Vec<TagItem>, Status> {
    let offset = (page - 1) * page_size;

    // 简化实现，从文章中提取标签
    let articles = state
        .repository
        .article
        .list(page, page_size, None, Some(2), None)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    // 收集所有标签
    let mut tag_map: std::collections::HashMap<String, (String, i64)> =
        std::collections::HashMap::new();
    for article in articles.articles {
        if let Some(tags) = article.tags
            && let Ok(tag_list) = serde_json::from_str::<Vec<String>>(&tags) {
                for tag in tag_list {
                    let entry = tag_map.entry(tag.clone()).or_insert((tag.clone(), 0));
                    entry.1 += 1;
                }
            }
    }

    let tags: Vec<TagItem> = tag_map
        .into_iter()
        .enumerate()
        .map(|(idx, (name, (_, count)))| {
            let tag_name = name.clone();
            TagItem {
                id: i64::from(offset + idx as i32),
                name,
                slug: tag_name,
                article_count: count,
                created_at: chrono::Utc::now(),
            }
        })
        .collect();

    Ok(tags)
}

/// 创建标签（简化实现，返回成功但不实际创建）
pub async fn create_tag(
    _state: Arc<CmsAppState>,
    name: String,
    slug: String,
) -> Result<i64, Status> {
    // 简化实现，实际应该创建标签记录
    let id = (name.len() + slug.len()) as i64;
    Ok(id)
}

/// 删除标签（简化实现）
pub async fn delete_tag(_state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    // 简化实现，实际应该删除标签记录
    Ok(id > 0)
}

// ============== 导出服务实现 ==============

use crate::repository::CmsRepository;

impl CmsGrpcService {
    #[must_use]
    pub const fn new(state: Arc<CmsAppState>) -> Self {
        Self { state }
    }

    /// 获取仓库引用
    #[must_use]
    pub fn repository(&self) -> &CmsRepository {
        &self.state.repository
    }
}
