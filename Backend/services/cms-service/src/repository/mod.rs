//! CMS Repository 模块

pub mod article_repository;
pub mod category_repository;

pub use article_repository::ArticleRepository;
pub use category_repository::CategoryRepository;

/// CMS Repository（聚合仓库）
#[derive(Clone)]
pub struct CmsRepository {
    pub category: CategoryRepository,
    pub article: ArticleRepository,
}

impl CmsRepository {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            category: CategoryRepository::new(pool.clone()),
            article: ArticleRepository::new(pool),
        }
    }
}
