//! 分类 Repository

use crate::models::{CategoryListResult, CategoryTreeNode, CmsCategory};
use sqlx::PgPool;

/// 分类 Repository 错误
#[derive(Debug, thiserror::Error)]
pub enum CategoryRepositoryError {
    #[error("分类不存在" )]
    NotFound,
    #[error("分类代码已存在" )]
    AlreadyExists,
    #[error("存在子分类" )]
    HasChildren,
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),
}

impl From<CategoryRepositoryError> for common::AppError {
    fn from(err: CategoryRepositoryError) -> Self {
        match err {
            CategoryRepositoryError::NotFound => Self::CategoryNotFound,
            CategoryRepositoryError::AlreadyExists => Self::CategoryAlreadyExists,
            CategoryRepositoryError::HasChildren => Self::CategoryHasChildren,
            CategoryRepositoryError::Database(e) => Self::Database(e),
        }
    }
}

/// 分类 Repository
#[derive(Clone)]
pub struct CategoryRepository {
    pool: PgPool,
}

impl CategoryRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取连接池引用
    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 获取分类列表
    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
    ) -> Result<CategoryListResult, CategoryRepositoryError> {
        let offset = (page - 1) * page_size;

        let (categories, total) = if let Some(kw) = keyword {
            let keyword_pattern = format!("%{kw}%" );
            let categories = sqlx::query_as!(
                CmsCategory,
                r#"
                SELECT id, parent_id, name, slug, description, icon, sort_order,
                       seo_title, seo_keywords, seo_description,
                       status::int4 AS "status!" , (allow_attachment <> 0) AS "allow_attachment!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
                FROM cms_category
                WHERE (name ILIKE $1 OR slug ILIKE $1) AND status = 1
                ORDER BY sort_order ASC, id ASC
                LIMIT $2 OFFSET $3
                "#,
                &keyword_pattern,
                i64::from(page_size),
                i64::from(offset),
            )
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM cms_category WHERE (name ILIKE $1 OR slug ILIKE $1) AND status = 1" ,
                &keyword_pattern,
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

            (categories, total)
        } else {
            let categories = sqlx::query_as!(
                CmsCategory,
                r#"
                SELECT id, parent_id, name, slug, description, icon, sort_order,
                       seo_title, seo_keywords, seo_description,
                       status::int4 AS "status!" , (allow_attachment <> 0) AS "allow_attachment!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
                FROM cms_category
                WHERE status = 1
                ORDER BY sort_order ASC, id ASC
                LIMIT $1 OFFSET $2
                "#,
                i64::from(page_size),
                i64::from(offset),
            )
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar!("SELECT COUNT(*) FROM cms_category WHERE status = 1" )
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0);

            (categories, total)
        };

        Ok(CategoryListResult { categories, total })
    }

    /// 获取分类树（内部递归）
    async fn get_tree_recursive(
        &self,
        parent_id: Option<i64>,
    ) -> Result<Vec<CategoryTreeNode>, CategoryRepositoryError> {
        let categories = if let Some(pid) = parent_id {
            sqlx::query_as!(
                CmsCategory,
                r#"
                SELECT id, parent_id, name, slug, description, icon, sort_order,
                       seo_title, seo_keywords, seo_description,
                       status::int4 AS "status!" , (allow_attachment <> 0) AS "allow_attachment!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
                FROM cms_category
                WHERE parent_id = $1 AND status = 1
                ORDER BY sort_order ASC, id ASC
                "#,
                pid,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                CmsCategory,
                r#"
                SELECT id, parent_id, name, slug, description, icon, sort_order,
                       seo_title, seo_keywords, seo_description,
                       status::int4 AS "status!" , (allow_attachment <> 0) AS "allow_attachment!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
                FROM cms_category
                WHERE parent_id IS NULL AND status = 1
                ORDER BY sort_order ASC, id ASC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        let mut tree = Vec::new();
        for category in categories {
            let children = Box::pin(self.get_tree_recursive(Some(category.id))).await?;
            tree.push(CategoryTreeNode {
                id: category.id,
                parent_id: category.parent_id,
                name: category.name,
                slug: category.slug,
                icon: category.icon,
                sort_order: category.sort_order,
                status: category.status,
                children,
            });
        }

        Ok(tree)
    }

    /// 获取分类树
    pub fn get_tree(
        &self,
        parent_id: Option<i64>,
    ) -> impl std::future::Future<Output = Result<Vec<CategoryTreeNode>, CategoryRepositoryError>> + '_
    {
        self.get_tree_recursive(parent_id)
    }

    /// 根据 ID 获取分类
    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<CmsCategory>, CategoryRepositoryError> {
        let category = sqlx::query_as!(
            CmsCategory,
            r#"
            SELECT id, parent_id, name, slug, description, icon, sort_order,
                   seo_title, seo_keywords, seo_description,
                   status::int4 AS "status!" , (allow_attachment <> 0) AS "allow_attachment!" ,
                   COALESCE(created_at, NOW()) AS "created_at!" ,
                   COALESCE(updated_at, NOW()) AS "updated_at!"
            FROM cms_category
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(category)
    }

    /// 创建分类
    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        parent_id: Option<i64>,
        description: Option<&str>,
    ) -> Result<i64, CategoryRepositoryError> {
        // 检查 slug 是否已存在
        let existing = sqlx::query_scalar!("SELECT id FROM cms_category WHERE slug = $1" , slug)
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            return Err(CategoryRepositoryError::AlreadyExists);
        }

        let row = sqlx::query_scalar!(
            r#"
            INSERT INTO cms_category (parent_id, name, slug, description, status, allow_attachment)
            VALUES (COALESCE($1::bigint, 0), $2, $3, $4, 1, 0)
            RETURNING id
            "#,
            parent_id,
            name,
            slug,
            description,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// 更新分类
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        sort_order: Option<i32>,
        status: Option<i32>,
    ) -> Result<bool, CategoryRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE cms_category
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                sort_order = COALESCE($4, sort_order),
                status = COALESCE($5::int4, status::int4),
                updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            name,
            description,
            sort_order,
            status,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除分类
    pub async fn delete(&self, id: i64) -> Result<bool, CategoryRepositoryError> {
        // 检查是否有子分类
        let children = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM cms_category WHERE parent_id = $1" ,
            id,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        if children > 0 {
            return Err(CategoryRepositoryError::HasChildren);
        }

        // 检查是否有文章关联
        let articles = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM cms_article WHERE category_id = $1" ,
            id,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        if articles > 0 {
            return Err(CategoryRepositoryError::HasChildren);
        }

        let result = sqlx::query!("DELETE FROM cms_category WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
