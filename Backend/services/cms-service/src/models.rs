//! CMS 数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============ 分类模型 ============

/// 文章分类
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct CmsCategory {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub seo_title: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_description: Option<String>,
    pub status: i32,
    pub allow_attachment: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 分类树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeNode {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i32,
    pub children: Vec<Self>,
}

/// 分类查询结果
#[derive(Debug, Clone)]
pub struct CategoryListResult {
    pub categories: Vec<CmsCategory>,
    pub total: i64,
}

// ============ 文章模型 ============

/// 文章状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ArticleStatus {
    Draft = 0,
    Pending = 1,
    Published = 2,
    Rejected = 3,
    Archived = 4,
}

impl ArticleStatus {
    #[must_use]
    pub const fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Draft,
            1 => Self::Pending,
            2 => Self::Published,
            3 => Self::Rejected,
            _ => Self::Archived,
        }
    }

    #[must_use]
    pub const fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/// 内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ContentType {
    RichText = 0,
    Markdown = 1,
}

/// CMS 文章
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct CmsArticle {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub slug: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub content_type: i32,
    pub cover_image: Option<String>,
    pub author_id: i64,
    pub tags: Option<String>, // JSON 数组
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub is_featured: bool,
    pub is_top: bool,
    pub status: i32,
    pub reject_reason: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub seo_title: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 文章查询结果
#[derive(Debug, Clone)]
pub struct ArticleListResult {
    pub articles: Vec<CmsArticle>,
    pub total: i64,
}

/// 文章统计
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArticleStatistics {
    pub total_articles: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub published_today: i64,
    pub pending_review: i64,
}

// ============ 响应结构 ============

/// 分类响应
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub seo_title: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_description: Option<String>,
    pub status: i32,
    pub allow_attachment: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 分类树节点响应
#[derive(Debug, Serialize)]
pub struct CategoryTreeResponse {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i32,
    pub children: Vec<Self>,
}

/// 文章响应
#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub id: i64,
    pub category_id: i64,
    pub category_name: Option<String>,
    pub title: String,
    pub slug: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub content_type: String,
    pub cover_image: Option<String>,
    pub author_id: i64,
    pub author_name: Option<String>,
    pub tags: Vec<String>,
    pub view_count: i64,
    pub like_count: i64,
    pub is_featured: bool,
    pub is_top: bool,
    pub status: String,
    pub reject_reason: Option<String>,
    pub published_at: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
