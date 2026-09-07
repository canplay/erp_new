//! gRPC Service Handlers for CMS
//!
//! 实现 cms.proto 中定义的 gRPC 服务

use crate::CmsAppState;
use crate::models::{CategoryTreeNode, CmsArticle, CmsCategory};
use std::sync::Arc;
use tonic::Status;

/// CMS gRPC 服务实现
#[derive(Clone)]
pub struct CmsGrpcService {
    state: Arc<CmsAppState>,
}

// ============== 文章相关实现 ==============

/// 获取文章列表
pub async fn list_articles(
    state: Arc<CmsAppState>,
    page: i32,
    page_size: i32,
    category_id: Option<i64>,
    status: Option<i32>,
    keyword: Option<String>,
) -> Result<Vec<CmsArticle>, Status> {
    state
        .repository
        .article
        .list(page, page_size, category_id, status, keyword.as_deref())
        .await
        .map(|result| result.articles)
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 获取文章详情
pub async fn get_article(state: Arc<CmsAppState>, id: i64) -> Result<Option<CmsArticle>, Status> {
    state
        .repository
        .article
        .find_by_id(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
    state
        .repository
        .article
        .create(category_id, &title, &content, author_id, is_draft)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
    state
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 删除文章
pub async fn delete_article(state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .article
        .delete(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 发布文章
pub async fn publish_article(state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .article
        .publish(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 下架文章
pub async fn unpublish_article(state: Arc<CmsAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .article
        .unpublish(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 审核文章
pub async fn review_article(
    state: Arc<CmsAppState>,
    id: i64,
    approved: bool,
    reason: Option<String>,
) -> Result<bool, Status> {
    state
        .repository
        .article
        .review(id, approved, reason.as_deref())
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 设置置顶
pub async fn set_article_top(
    state: Arc<CmsAppState>,
    id: i64,
    is_top: bool,
) -> Result<bool, Status> {
    state
        .repository
        .article
        .set_top(id, is_top)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 设置推荐
pub async fn set_article_featured(
    state: Arc<CmsAppState>,
    id: i64,
    is_featured: bool,
) -> Result<bool, Status> {
    state
        .repository
        .article
        .set_featured(id, is_featured)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 增加浏览次数
pub async fn increment_article_view_count(
    state: Arc<CmsAppState>,
    id: i64,
) -> Result<bool, Status> {
    state
        .repository
        .article
        .increment_view_count(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 增加点赞次数
pub async fn increment_article_like_count(
    state: Arc<CmsAppState>,
    id: i64,
) -> Result<bool, Status> {
    state
        .repository
        .article
        .increment_like_count(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 获取热门文章
pub async fn get_hot_articles(
    state: Arc<CmsAppState>,
    limit: i32,
    category_id: Option<i64>,
) -> Result<Vec<CmsArticle>, Status> {
    state
        .repository
        .article
        .get_hot_articles(limit, category_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 获取最新文章
pub async fn get_latest_articles(
    state: Arc<CmsAppState>,
    limit: i32,
    category_id: Option<i64>,
) -> Result<Vec<CmsArticle>, Status> {
    state
        .repository
        .article
        .get_latest_articles(limit, category_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

// ============== 分类相关实现 ==============

/// 获取分类列表
pub async fn list_categories(
    state: Arc<CmsAppState>,
    parent_id: Option<i64>,
    include_children: bool,
) -> Result<Vec<CmsCategory>, Status> {
    if include_children {
        // 如果需要包含子分类，使用树形结构
        state
            .repository
            .category
            .get_tree(parent_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))
            .map(|_| vec![]) // 简化处理，实际应该返回树形结构
    } else {
        // 直接获取列表
        let result = state
            .repository
            .category
            .list(1, 100, None)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;
        Ok(result.categories)
    }
}

/// 获取分类详情
pub async fn get_category(state: Arc<CmsAppState>, id: i64) -> Result<Option<CmsCategory>, Status> {
    state
        .repository
        .category
        .find_by_id(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
    state
        .repository
        .category
        .create(&name, &slug, parent_id, description.as_deref())
        .await
        .map_err(|e| match e {
            crate::repository::category_repository::CategoryRepositoryError::AlreadyExists => {
                Status::already_exists("Category slug already exists")
            }
            _ => Status::internal(format!("Database error: {e}")),
        })
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
    state
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 删除分类
pub async fn delete_category(
    state: Arc<CmsAppState>,
    id: i64,
    force: bool,
) -> Result<bool, Status> {
    if force {
        // 强制删除（需要先删除关联的文章和子分类）
        // 这里简化处理，实际应该递归删除
        state
            .repository
            .category
            .delete(id)
            .await
            .map_err(|e| match e {
                crate::repository::category_repository::CategoryRepositoryError::HasChildren => {
                    Status::failed_precondition("Category has children or articles")
                }
                _ => Status::internal(format!("Database error: {e}")),
            })
    } else {
        // 非强制删除，检查是否有依赖
        state
            .repository
            .category
            .delete(id)
            .await
            .map_err(|e| match e {
                crate::repository::category_repository::CategoryRepositoryError::HasChildren => {
                    Status::failed_precondition("Category has children or articles")
                }
                _ => Status::internal(format!("Database error: {e}")),
            })
    }
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

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
