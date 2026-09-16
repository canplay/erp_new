//! 文章 Repository

use crate::models::{ArticleListResult, ArticleStatistics, CmsArticle};
use sqlx::PgPool;

/// 文章 Repository 错误
#[derive(Debug, thiserror::Error)]
pub enum ArticleRepositoryError {
    #[error("文章不存在" )]
    NotFound,
    #[error("文章代码已存在" )]
    AlreadyExists,
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),
}

/// 文章 Repository
#[derive(Clone)]
pub struct ArticleRepository {
    pool: PgPool,
}

impl ArticleRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取文章列表
    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        category_id: Option<i64>,
        status: Option<i32>,
        keyword: Option<&str>,
    ) -> Result<ArticleListResult, ArticleRepositoryError> {
        let offset = (page - 1) * page_size;

        // 直接执行查询，避免动态构建导致的生命周期问题
        let articles: Vec<CmsArticle>;

        if let Some(cid) = category_id {
            if let Some(s) = status {
                if let Some(kw) = keyword {
                    let pattern = format!("%{kw}%" );
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                        SELECT id, category_id, title, slug, summary, content, content_type,
                               cover_image, author_id, tags, view_count, like_count, comment_count,
                               share_count, is_featured, is_top, status, reject_reason, published_at,
                               source, source_url, seo_title, seo_keywords, seo_description,
                               created_at, updated_at
                        FROM cms_article
                        WHERE category_id = $1 AND status = $2 AND (title ILIKE $3 OR content ILIKE $3)
                        ORDER BY is_top DESC, id DESC
                        LIMIT $4 OFFSET $5
                        " ,
                        cid,
                        s,
                        &pattern,
                        page_size as i32,
                        offset as i32,
                    )
                    .fetch_all(&self.pool)
                    .await?;
                } else {
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            WHERE category_id = $1 AND status = $2
                            ORDER BY is_top DESC, id DESC
                            LIMIT $3 OFFSET $4
                            " ,
                        cid,
                        s,
                        page_size as i32,
                        offset as i32,
                    )
                    .fetch_all(&self.pool)
                    .await?;
                }
            } else {
                if let Some(kw) = keyword {
                    let pattern = format!("%{kw}%" );
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            WHERE category_id = $1 AND (title ILIKE $2 OR content ILIKE $2)
                            ORDER BY is_top DESC, id DESC
                            LIMIT $3 OFFSET $4
                            " ,
                            cid,
                            &pattern,
                            page_size as i32,
                            offset as i32,
                        )
                        .fetch_all(&self.pool)
                        .await?;
                } else {
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            WHERE category_id = $1
                            ORDER BY is_top DESC, id DESC
                            LIMIT $2 OFFSET $3
                            " ,
                            cid,
                            page_size as i32,
                            offset as i32,
                        )
                        .fetch_all(&self.pool)
                        .await?;
                }
            }
        } else {
            if let Some(s) = status {
                if let Some(kw) = keyword {
                    let pattern = format!("%{kw}%" );
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            WHERE status = $1 AND (title ILIKE $2 OR content ILIKE $2)
                            ORDER BY is_top DESC, id DESC
                            LIMIT $3 OFFSET $4
                            " ,
                            s,
                            &pattern,
                            page_size as i32,
                            offset as i32,
                        )
                        .fetch_all(&self.pool)
                        .await?;
                } else {
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            WHERE status = $1
                            ORDER BY is_top DESC, id DESC
                            LIMIT $2 OFFSET $3
                            " ,
                            s,
                            page_size as i32,
                            offset as i32,
                        )
                        .fetch_all(&self.pool)
                        .await?;
                }
            } else {
                if let Some(kw) = keyword {
                    let pattern = format!("%{kw}%" );
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            WHERE title ILIKE $1 OR content ILIKE $1
                            ORDER BY is_top DESC, id DESC
                            LIMIT $2 OFFSET $3
                            " ,
                            &pattern,
                            page_size as i32,
                            offset as i32,
                        )
                        .fetch_all(&self.pool)
                        .await?;
                } else {
                    articles = sqlx::query_as!(
                        CmsArticle,
                        r"
                            SELECT id, category_id, title, slug, summary, content, content_type,
                                   cover_image, author_id, tags, view_count, like_count, comment_count,
                                   share_count, is_featured, is_top, status, reject_reason, published_at,
                                   source, source_url, seo_title, seo_keywords, seo_description,
                                   created_at, updated_at
                            FROM cms_article
                            ORDER BY is_top DESC, id DESC
                            LIMIT $1 OFFSET $2
                            " ,
                            page_size as i32,
                            offset as i32,
                        )
                        .fetch_all(&self.pool)
                        .await?;
                }
            }
        }

        // 获取总数
        // 获取总数
        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM cms_article" )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok(ArticleListResult {
            articles,
            total,
        })
    }

    /// 根据 ID 获取文章
    pub async fn find_by_id(&self, id: i64) -> Result<Option<CmsArticle>, ArticleRepositoryError> {
        let article = sqlx::query_as!(
            CmsArticle,
            r"
            SELECT id, category_id, title, slug, summary, content, content_type,
                   cover_image, author_id, tags, view_count, like_count, comment_count,
                   share_count, is_featured, is_top, status, reject_reason, published_at,
                   source, source_url, seo_title, seo_keywords, seo_description,
                   created_at, updated_at
            FROM cms_article
            WHERE id = $1
            " ,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(article)
    }

    /// 创建文章
    pub async fn create(
        &self,
        category_id: i64,
        title: &str,
        content: &str,
        author_id: i64,
        is_draft: bool,
    ) -> Result<i64, ArticleRepositoryError> {
        let id: i64 = sqlx::query_scalar!(
            r"
            INSERT INTO cms_article (category_id, title, content, author_id, status, content_type)
            VALUES ($1, $2, $3, $4, $5, 0)
            RETURNING id
            " ,
            category_id,
            title,
            content,
            author_id,
            i32::from(!is_draft) as i32,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// 更新文章
    pub async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
        is_featured: Option<bool>,
        is_top: Option<bool>,
    ) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            r"
            UPDATE cms_article
            SET title = COALESCE($2, title),
                content = COALESCE($3, content),
                is_featured = COALESCE($4, is_featured),
                is_top = COALESCE($5, is_top),
                updated_at = NOW()
            WHERE id = $1
            " ,
            id,
            title,
            content,
            is_featured,
            is_top,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 发布文章
    pub async fn publish(&self, id: i64) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            r"
            UPDATE cms_article
            SET status = 2, published_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status IN (0, 1)
            " ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除文章
    pub async fn delete(&self, id: i64) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM cms_article WHERE id = $1" ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取文章统计
    pub async fn get_statistics(&self) -> Result<ArticleStatistics, ArticleRepositoryError> {
        let stats = sqlx::query_as!(
            ArticleStatistics,
            r#"
            SELECT
                COUNT(*) AS "total_articles!" ,
                COALESCE(SUM(view_count), 0)::bigint AS "total_views!" ,
                COALESCE(SUM(like_count), 0)::bigint AS "total_likes!" ,
                COUNT(*) FILTER (WHERE published_at::date = CURRENT_DATE) AS "published_today!" ,
                COUNT(*) FILTER (WHERE status = 1) AS "pending_review!"
            FROM cms_article
"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    /// 下架文章
    pub async fn unpublish(&self, id: i64) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            r"
            UPDATE cms_article
            SET status = 0, updated_at = NOW()
            WHERE id = $1 AND status = 2
            " ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 审核文章
    pub async fn review(
        &self,
        id: i64,
        approved: bool,
        reason: Option<&str>,
    ) -> Result<bool, ArticleRepositoryError> {
        let new_status = if approved { 2 } else { 1 }; // 2=已发布, 1=待审核(驳回后仍为待审核)
        let result = sqlx::query!(
            r"
            UPDATE cms_article
            SET status = $2, reject_reason = $3, published_at = CASE WHEN $2 = 2 THEN NOW() ELSE published_at END, updated_at = NOW()
            WHERE id = $1
            " ,
            id,
            new_status,
            reason,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 设置置顶
    pub async fn set_top(&self, id: i64, is_top: bool) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            "UPDATE cms_article SET is_top = $2, updated_at = NOW() WHERE id = $1" ,
            id,
            is_top,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 设置推荐
    pub async fn set_featured(
        &self,
        id: i64,
        is_featured: bool,
    ) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            "UPDATE cms_article SET is_featured = $2, updated_at = NOW() WHERE id = $1" ,
            id,
            is_featured,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 增加浏览次数
    pub async fn increment_view_count(&self, id: i64) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            "UPDATE cms_article SET view_count = view_count + 1 WHERE id = $1" ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 增加点赞次数
    pub async fn increment_like_count(&self, id: i64) -> Result<bool, ArticleRepositoryError> {
        let result = sqlx::query!(
            "UPDATE cms_article SET like_count = like_count + 1 WHERE id = $1" ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取热门文章
    pub async fn get_hot_articles(
        &self,
        limit: i32,
        category_id: Option<i64>,
    ) -> Result<Vec<CmsArticle>, ArticleRepositoryError> {
        let articles: Vec<CmsArticle> = if let Some(cid) = category_id {
            sqlx::query_as!(
                CmsArticle,
                r"
                SELECT id, category_id, title, slug, summary, content, content_type,
                       cover_image, author_id, tags, view_count, like_count, comment_count,
                       share_count, is_featured, is_top, status, reject_reason, published_at,
                       source, source_url, seo_title, seo_keywords, seo_description,
                       created_at, updated_at
                FROM cms_article
                WHERE status = 2 AND category_id = $1
                ORDER BY view_count DESC, like_count DESC
                LIMIT $2
                " ,
                cid,
                limit as i32,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                CmsArticle,
                r"
                SELECT id, category_id, title, slug, summary, content, content_type,
                       cover_image, author_id, tags, view_count, like_count, comment_count,
                       share_count, is_featured, is_top, status, reject_reason, published_at,
                       source, source_url, seo_title, seo_keywords, seo_description,
                       created_at, updated_at
                FROM cms_article
                WHERE status = 2
                ORDER BY view_count DESC, like_count DESC
                LIMIT $1
                " ,
                limit as i32,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(articles)
    }

    /// 获取最新文章
    pub async fn get_latest_articles(
        &self,
        limit: i32,
        category_id: Option<i64>,
    ) -> Result<Vec<CmsArticle>, ArticleRepositoryError> {
        let articles: Vec<CmsArticle> = if let Some(cid) = category_id {
            sqlx::query_as!(
                CmsArticle,
                r"
                SELECT id, category_id, title, slug, summary, content, content_type,
                       cover_image, author_id, tags, view_count, like_count, comment_count,
                       share_count, is_featured, is_top, status, reject_reason, published_at,
                       source, source_url, seo_title, seo_keywords, seo_description,
                       created_at, updated_at
                FROM cms_article
                WHERE status = 2 AND category_id = $1
                ORDER BY published_at DESC
                LIMIT $2
                " ,
                cid,
                limit as i32,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                CmsArticle,
                r"
                SELECT id, category_id, title, slug, summary, content, content_type,
                       cover_image, author_id, tags, view_count, like_count, comment_count,
                       share_count, is_featured, is_top, status, reject_reason, published_at,
                       source, source_url, seo_title, seo_keywords, seo_description,
                       created_at, updated_at
                FROM cms_article
                WHERE status = 2
                ORDER BY published_at DESC
                LIMIT $1
                " ,
                limit as i32,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(articles)
    }
}
