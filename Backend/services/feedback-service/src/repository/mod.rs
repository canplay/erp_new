//! 反馈仓储层
//!
//! 实现反馈数据的 CRUD 操作

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

/// 反馈仓储错误
#[derive(Error, Debug)]
pub enum FeedbackRepositoryError {
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("反馈不存在" )]
    NotFound,
}

impl From<FeedbackRepositoryError> for common::AppError {
    fn from(err: FeedbackRepositoryError) -> Self {
        match err {
            FeedbackRepositoryError::NotFound => Self::FeedbackNotFound,
            FeedbackRepositoryError::Database(e) => Self::Database(e),
        }
    }
}

/// 反馈类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase" )]
pub enum FeedbackType {
    Suggestion,
    Bug,
    Complaint,
    Other,
}

/// 反馈状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase" )]
pub enum FeedbackStatus {
    Pending,
    Processing,
    Resolved,
    Rejected,
    Closed,
}

impl FeedbackType {
    #[must_use]
    pub fn from_str_static(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "suggestion" => Self::Suggestion,
            "bug" => Self::Bug,
            "complaint" => Self::Complaint,
            _ => Self::Other,
        }
    }
}

impl FeedbackStatus {
    #[must_use]
    pub fn from_str_static(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "resolved" => Self::Resolved,
            "rejected" => Self::Rejected,
            "closed" => Self::Closed,
            _ => Self::Pending,
        }
    }
}

/// 反馈实体
#[derive(Debug, Clone, Serialize)]
pub struct Feedback {
    pub id: i64,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub r#type: String,
    pub title: String,
    pub content: String,
    pub contact: Option<String>,
    pub status: String,
    pub handler_id: Option<i64>,
    pub handler_name: Option<String>,
    pub handler_reply: Option<String>,
    pub handler_time: Option<DateTime<Utc>>,
    pub rating: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 反馈查询参数
#[derive(Debug, Default)]
pub struct FeedbackQueryParams {
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
    pub handler_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 分页结果
#[derive(Debug, Serialize)]
pub struct PaginatedFeedback {
    pub list: Vec<Feedback>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 反馈统计
#[derive(Debug, Serialize)]
pub struct FeedbackStatistics {
    pub total: i64,
    pub pending: i64,
    pub processing: i64,
    pub resolved: i64,
    pub rejected: i64,
    pub avg_response_time: f64,
    pub satisfaction_rate: f64,
}

/// 类型统计
#[derive(Debug, Serialize)]
pub struct TypeStatistics {
    pub r#type: String,
    pub count: i64,
    pub percentage: f64,
}

/// 处理人
#[derive(Debug, Serialize)]
pub struct Handler {
    pub id: i64,
    pub name: String,
}

/// 反馈仓储
#[derive(Clone)]
pub struct FeedbackRepository {
    pool: PgPool,
}

impl FeedbackRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建反馈
    pub async fn create(
        &self,
        user_id: i64,
        feedback_type: &str,
        title: &str,
        content: &str,
        contact: Option<&str>,
    ) -> Result<i64, FeedbackRepositoryError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO sys_feedback (user_id, type, title, content, contact, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'pending', NOW(), NOW())
            RETURNING id
            "#,
            user_id,
            feedback_type,
            title,
            content,
            contact,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// 根据ID查询
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Feedback>, FeedbackRepositoryError> {
        let row = sqlx::query_as!(
            Feedback,
            r#"
            SELECT f.id, f.user_id, u.username AS user_name, f.type, f.title, f.content,
                   f.contact, f.status, f.handler_id, h.username AS handler_name,
                   f.handler_reply, f.handler_time, f.rating::int,
                   COALESCE(f.created_at, NOW()) AS "created_at!" ,
                   COALESCE(f.updated_at, NOW()) AS "updated_at!"
            FROM sys_feedback f
            LEFT JOIN users u ON f.user_id = u.id
            LEFT JOIN users h ON f.handler_id = h.id
            WHERE f.id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 分页查询
    pub async fn list(
        &self,
        params: &FeedbackQueryParams,
    ) -> Result<PaginatedFeedback, FeedbackRepositoryError> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(15).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let r#type = params.r#type.as_deref();
        let status = params.status.as_deref();
        let keyword = params.keyword.as_deref();

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM sys_feedback f
            WHERE ($1::text IS NULL OR f.type = $1)
              AND ($2::text IS NULL OR f.status = $2)
              AND ($3::text IS NULL OR (f.title LIKE $3 OR f.content LIKE $3))
            "#,
            r#type,
            status,
            keyword,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let list: Vec<Feedback> = sqlx::query_as!(
            Feedback,
            r#"
            SELECT f.id, f.user_id, u.username AS user_name, f.type, f.title, f.content,
                   f.contact, f.status, f.handler_id, h.username AS handler_name,
                   f.handler_reply, f.handler_time, f.rating::int,
                   COALESCE(f.created_at, NOW()) AS "created_at!" ,
                   COALESCE(f.updated_at, NOW()) AS "updated_at!"
            FROM sys_feedback f
            LEFT JOIN users u ON f.user_id = u.id
            LEFT JOIN users h ON f.handler_id = h.id
            WHERE ($1::text IS NULL OR f.type = $1)
              AND ($2::text IS NULL OR f.status = $2)
              AND ($3::text IS NULL OR (f.title LIKE $3 OR f.content LIKE $3))
            ORDER BY f.created_at DESC
            LIMIT $4::int OFFSET $5::int
            "#,
            r#type,
            status,
            keyword,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedFeedback {
            list,
            total,
            page,
            page_size,
        })
    }

    /// 处理反馈
    pub async fn handle(
        &self,
        id: i64,
        status: &str,
        reply: &str,
    ) -> Result<bool, FeedbackRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE sys_feedback
            SET status = $1, handler_reply = $2, handler_time = NOW(), updated_at = NOW()
            WHERE id = $3
            "#,
            status,
            reply,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 转交反馈
    pub async fn transfer(
        &self,
        id: i64,
        handler_id: i64,
    ) -> Result<bool, FeedbackRepositoryError> {
        let result = sqlx::query!(
            "UPDATE sys_feedback SET handler_id = $1, status = 'processing', updated_at = NOW() WHERE id = $2" ,
            handler_id,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 添加回复
    pub async fn add_reply(&self, id: i64, reply: &str) -> Result<(), FeedbackRepositoryError> {
        sqlx::query!(
            r#"
            UPDATE sys_feedback
            SET handler_reply = COALESCE(handler_reply, '') || E'\n' || $1,
                handler_time = NOW(),
                updated_at = NOW()
            WHERE id = $2
            "#,
            reply,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除反馈
    pub async fn delete(&self, id: i64) -> Result<bool, FeedbackRepositoryError> {
        let result = sqlx::query!("DELETE FROM sys_feedback WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量处理
    pub async fn batch_handle(
        &self,
        ids: &[i64],
        status: &str,
        reply: Option<&str>,
    ) -> Result<usize, FeedbackRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE sys_feedback
            SET status = $1, handler_reply = $2, handler_time = NOW(), updated_at = NOW()
            WHERE id = ANY($3)
            "#,
            status,
            reply.unwrap_or(" "),
            ids,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// 获取统计
    pub async fn get_statistics(
        &self,
        _params: &FeedbackQueryParams,
    ) -> Result<FeedbackStatistics, FeedbackRepositoryError> {
        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM sys_feedback" )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
        let pending: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_feedback WHERE status = 'pending'"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        let processing: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_feedback WHERE status = 'processing'"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        let resolved: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_feedback WHERE status = 'resolved'"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        let rejected: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_feedback WHERE status = 'rejected'"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(FeedbackStatistics {
            total,
            pending,
            processing,
            resolved,
            rejected,
            avg_response_time: 0.0,
            satisfaction_rate: 0.0,
        })
    }

    /// 获取类型统计
    pub async fn get_type_statistics(
        &self,
        _params: &FeedbackQueryParams,
    ) -> Result<Vec<TypeStatistics>, FeedbackRepositoryError> {
        let rows = sqlx::query!(
            r#"
            SELECT type, COUNT(*) AS "count!"
            FROM sys_feedback
            GROUP BY type
            ORDER BY COUNT(*) DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = rows.iter().map(|r| r.count).sum();

        let stats: Vec<TypeStatistics> = rows
            .into_iter()
            .map(|r| {
                let count: i64 = r.count;
                TypeStatistics {
                    r#type: r.r#type,
                    count,
                    percentage: if total > 0 {
                        (count as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect();

        Ok(stats)
    }

    /// 获取处理人列表
    pub async fn get_handlers(&self) -> Result<Vec<Handler>, FeedbackRepositoryError> {
        let handlers: Vec<Handler> = sqlx::query_as!(
            Handler,
            r#"
            SELECT DISTINCT u.id, u.username AS name
            FROM users u
            INNER JOIN sys_feedback f ON u.id = f.handler_id
            WHERE f.handler_id IS NOT NULL
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(handlers)
    }
}
