//! Repository 模式定义
//!
//! 提供统一的数据库访问抽象层

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 分页查询参数
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageQuery {
    /// 页码（从1开始）
    #[serde(default = "default_page")]
    pub page: i32,
    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

const fn default_page() -> i32 {
    1
}

const fn default_page_size() -> i32 {
    10
}

impl PageQuery {
    /// 从请求参数创建分页查询
    #[must_use]
    pub fn new(page: Option<i32>, page_size: Option<i32>) -> Self {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(10).clamp(1, 100);
        Self { page, page_size }
    }

    /// 计算 OFFSET
    #[must_use]
    pub const fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }

    /// 获取 LIMIT
    #[must_use]
    pub const fn limit(&self) -> i64 {
        self.page_size as i64
    }
}

/// 分页结果
#[derive(Debug, Clone, Serialize)]
pub struct PageResult<T> {
    /// 数据列表
    pub records: Vec<T>,
    /// 总数
    pub total: i64,
    /// 当前页码
    pub page: i32,
    /// 每页数量
    pub page_size: i32,
    /// 总页数
    pub pages: i32,
}

impl<T> PageResult<T> {
    /// 创建分页结果
    #[must_use]
    pub const fn new(records: Vec<T>, total: i64, page: i32, page_size: i32) -> Self {
        let pages = if total == 0 {
            0
        } else {
            (total + page_size as i64 - 1) / page_size as i64
        };
        Self {
            records,
            total,
            page,
            page_size,
            pages: pages as i32,
        }
    }

    /// 是否为空
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 是否有下一页
    #[must_use]
    pub const fn has_next(&self) -> bool {
        self.page < self.pages
    }

    /// 是否有上一页
    #[must_use]
    pub const fn has_prev(&self) -> bool {
        self.page > 1
    }
}

/// Repository 错误类型
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("记录不存在: {0}")]
    NotFound(String),

    #[error("记录已存在")]
    AlreadyExists,

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("参数错误: {0}")]
    InvalidParam(String),

    #[error("删除失败: {0}")]
    DeleteFailed(String),

    #[error("更新失败: {0}")]
    UpdateFailed(String),
}

/// Repository 结果类型
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// 基础 Repository trait
/// 所有实体 Repository 应实现此 trait
#[async_trait]
pub trait BaseRepository: Send + Sync {
    /// 获取数据库连接池
    fn pool(&self) -> &PgPool;
}

/// 通用的 CRUD Repository trait
#[async_trait]
pub trait CrudRepository<T, ID>: BaseRepository
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// 根据 ID 查询
    async fn find_by_id(&self, id: ID) -> RepositoryResult<Option<T>>;

    /// 查询所有
    async fn find_all(&self) -> RepositoryResult<Vec<T>>;

    /// 分页查询
    async fn find_page(&self, query: &PageQuery) -> RepositoryResult<PageResult<T>>;

    /// 创建
    async fn create(&self, entity: &T) -> RepositoryResult<ID>;

    /// 更新
    async fn update(&self, id: ID, entity: &T) -> RepositoryResult<()>;

    /// 删除
    async fn delete(&self, id: ID) -> RepositoryResult<()>;
}

/// 分页查询宏
/// 用于简化分页查询的实现
#[macro_export]
macro_rules! impl_paginate {
    ($entity:ident, $table:expr) => {
        fn paginate_query(&self, query: &PageQuery) -> String {
            format!(
                "SELECT * FROM {} ORDER BY id DESC LIMIT {} OFFSET {}",
                $table,
                query.limit(),
                query.offset()
            )
        }

        fn count_query(&self) -> String {
            format!("SELECT COUNT(*) FROM {}", $table)
        }
    };
}
