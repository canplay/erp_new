//! Messaging 数据仓储层
//!
//! 实现消息、公告的数据库 CRUD 操作

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 消息模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub msg_type: String,
    pub title: String,
    pub content: String,
    pub sender_id: Option<i64>,
    pub sender_name: Option<String>,
    pub priority: i32,
    pub attachment_urls: Option<serde_json::Value>,
    pub target_type: String,
    pub target_ids: Option<serde_json::Value>,
    pub expire_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 用户消息关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUser {
    pub id: i64,
    pub message_id: i64,
    pub user_id: i64,
    pub is_read: bool,
    pub read_time: Option<DateTime<Utc>>,
    pub is_starred: bool,
    pub is_deleted: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
}

/// 公告模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub announcement_type: String,
    pub priority: i32,
    pub is_pinned: bool,
    pub is_active: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 消息模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub id: i64,
    pub name: String,
    pub template_type: String,
    pub title_template: String,
    pub content_template: String,
    pub variables: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message Repository trait
pub trait MessageRepository: Send + Sync {
    /// 发送消息
    fn send_message(
        &self,
        msg: &Message,
        user_ids: &[i64],
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 获取用户消息列表
    fn list_user_messages(
        &self,
        user_id: i64,
        msg_type: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<Message>, i64, i64), sqlx::Error>> + Send;

    /// 获取消息详情
    fn get_message(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<Option<Message>, sqlx::Error>> + Send;

    /// 标记消息已读
    fn mark_as_read(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 标记所有消息已读
    fn mark_all_as_read(
        &self,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<i64, sqlx::Error>> + Send;

    /// 删除消息
    fn delete_message(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 获取未读消息数
    fn get_unread_count(
        &self,
        user_id: i64,
        msg_type: Option<&str>,
    ) -> impl std::future::Future<Output = Result<i64, sqlx::Error>> + Send;
}

/// Announcement Repository trait
pub trait AnnouncementRepository: Send + Sync {
    /// 创建公告
    fn create(
        &self,
        ann: &Announcement,
    ) -> impl std::future::Future<Output = Result<i64, sqlx::Error>> + Send;

    /// 更新公告
    fn update(
        &self,
        ann: &Announcement,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 删除公告
    fn delete(&self, id: i64) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 获取公告详情
    fn find_by_id(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<Announcement>, sqlx::Error>> + Send;

    /// 列出公告
    fn list(
        &self,
        is_active: Option<bool>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<Announcement>, i64), sqlx::Error>> + Send;

    /// 获取当前有效公告
    fn get_active(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Announcement>, sqlx::Error>> + Send;
}

/// Template Repository trait
pub trait TemplateRepository: Send + Sync {
    /// 创建模板
    fn create(
        &self,
        tmpl: &MessageTemplate,
    ) -> impl std::future::Future<Output = Result<i64, sqlx::Error>> + Send;

    /// 更新模板
    fn update(
        &self,
        tmpl: &MessageTemplate,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 删除模板
    fn delete(&self, id: i64) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;

    /// 获取模板详情
    fn find_by_id(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<MessageTemplate>, sqlx::Error>> + Send;

    /// 列出模板
    fn list(
        &self,
        tmpl_type: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<MessageTemplate>, i64), sqlx::Error>> + Send;
}

/// `PostgreSQL` Message Repository
pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl MessageRepository for PostgresMessageRepository {
    async fn send_message(&self, msg: &Message, user_ids: &[i64]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 插入消息
        let message_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO sys_message (type, title, content, sender_id, sender_name, priority,
                attachment_urls, target_type, target_ids, expire_time, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
            RETURNING id
            "#,
            &msg.msg_type,
            &msg.title,
            &msg.content,
            msg.sender_id,
            msg.sender_name.as_deref(),
            msg.priority as i16,
            msg.attachment_urls.as_ref(),
            &msg.target_type,
            msg.target_ids.as_ref(),
            msg.expire_time,
            msg.created_at,
        )
        .fetch_one(&mut *tx)
        .await?;

        // N+1 FIX: Batch INSERT for user-message associations using UNNEST
        if !user_ids.is_empty() {
            sqlx::query!(
                r#"INSERT INTO sys_message_user (message_id, user_id, is_read, is_deleted, is_archived, created_at)
                  SELECT $1, unnest($2::bigint[]), 0, 0, 0, NOW()"#,
                message_id,
                user_ids as &[i64],
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn list_user_messages(
        &self,
        user_id: i64,
        msg_type: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Message>, i64, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let rows = sqlx::query!(
            r#"
            SELECT m.id, m.type AS msg_type, m.title, m.content, m.sender_id, m.sender_name,
                   m.priority::int AS "priority!" , m.attachment_urls, m.target_type, m.target_ids,
                   m.expire_time, COALESCE(m.created_at, NOW()) AS "created_at!"
            FROM sys_message m
            JOIN sys_message_user mu ON m.id = mu.message_id
            WHERE mu.user_id = $1 AND mu.is_deleted = 0 AND ($2::varchar IS NULL OR m.type = $2)
            ORDER BY m.priority DESC, m.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            msg_type,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let messages: Vec<Message> = rows
            .into_iter()
            .map(|r| Message {
                id: r.id,
                msg_type: r.msg_type,
                title: r.title,
                content: r.content,
                sender_id: r.sender_id,
                sender_name: r.sender_name,
                priority: r.priority,
                attachment_urls: r.attachment_urls,
                target_type: r.target_type,
                target_ids: r.target_ids,
                expire_time: r.expire_time,
                created_at: r.created_at,
            })
            .collect();

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM sys_message m
            JOIN sys_message_user mu ON m.id = mu.message_id
            WHERE mu.user_id = $1 AND mu.is_deleted = 0 AND ($2::varchar IS NULL OR m.type = $2)
            "#,
            user_id,
            msg_type,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let unread = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_message_user WHERE user_id = $1 AND is_read = 0 AND is_deleted = 0" ,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok((messages, count, unread))
    }

    async fn get_message(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> Result<Option<Message>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT m.id, m.type AS msg_type, m.title, m.content, m.sender_id, m.sender_name,
                   m.priority::int AS "priority!" , m.attachment_urls, m.target_type, m.target_ids,
                   m.expire_time, COALESCE(m.created_at, NOW()) AS "created_at!"
            FROM sys_message m
            JOIN sys_message_user mu ON m.id = mu.message_id
            WHERE m.id = $1 AND mu.user_id = $2 AND mu.is_deleted = 0
            "#,
            message_id,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Message {
            id: r.id,
            msg_type: r.msg_type,
            title: r.title,
            content: r.content,
            sender_id: r.sender_id,
            sender_name: r.sender_name,
            priority: r.priority,
            attachment_urls: r.attachment_urls,
            target_type: r.target_type,
            target_ids: r.target_ids,
            expire_time: r.expire_time,
            created_at: r.created_at,
        }))
    }

    async fn mark_as_read(&self, message_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE sys_message_user SET is_read = 1, read_time = NOW() WHERE message_id = $1 AND user_id = $2" ,
            message_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_all_as_read(&self, user_id: i64) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE sys_message_user SET is_read = 1, read_time = NOW() WHERE user_id = $1 AND is_read = 0" ,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    async fn delete_message(&self, message_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE sys_message_user SET is_deleted = 1 WHERE message_id = $1 AND user_id = $2" ,
            message_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_unread_count(
        &self,
        user_id: i64,
        msg_type: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM sys_message m
            JOIN sys_message_user mu ON m.id = mu.message_id
            WHERE mu.user_id = $1 AND mu.is_read = 0 AND mu.is_deleted = 0
              AND ($2::varchar IS NULL OR m.type = $2)
            "#,
            user_id,
            msg_type,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }
}

/// `PostgreSQL` Announcement Repository
pub struct PostgresAnnouncementRepository {
    pool: PgPool,
}

impl PostgresAnnouncementRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AnnouncementRepository for PostgresAnnouncementRepository {
    async fn create(&self, ann: &Announcement) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO announcements (title, content, announcement_type, priority, is_pinned, is_active,
                start_time, end_time, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
            RETURNING id
            "#,
            &ann.title,
            &ann.content,
            &ann.announcement_type,
            ann.priority,
            ann.is_pinned,
            ann.is_active,
            ann.start_time,
            ann.end_time,
            ann.created_by,
            ann.created_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    async fn update(&self, ann: &Announcement) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE announcements SET 
                title = $2, content = $3, announcement_type = $4, priority = $5,
                is_pinned = $6, is_active = $7, start_time = $8, end_time = $9,
                updated_at = NOW()
            WHERE id = $1
            "#,
            ann.id,
            &ann.title,
            &ann.content,
            &ann.announcement_type,
            ann.priority,
            ann.is_pinned,
            ann.is_active,
            ann.start_time,
            ann.end_time,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM announcements WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Announcement>, sqlx::Error> {
        let row = sqlx::query_as!(
            Announcement,
            r#"
            SELECT id, title, content,
                   COALESCE(announcement_type, '') AS "announcement_type!" ,
                   COALESCE(priority, 0) AS "priority!" ,
                   COALESCE(is_pinned, false) AS "is_pinned!" ,
                   COALESCE(is_active, false) AS "is_active!" ,
                   start_time, end_time, created_by,
                   COALESCE(created_at, NOW()) AS "created_at!" ,
                   COALESCE(updated_at, NOW()) AS "updated_at!"
            FROM announcements
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list(
        &self,
        is_active: Option<bool>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Announcement>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;
        let active = if is_active == Some(true) { Some(true) } else { None };

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM announcements
            WHERE ($1::boolean IS NULL OR (is_active = $1 AND (start_time IS NULL OR start_time <= NOW())
                AND (end_time IS NULL OR end_time >= NOW())))
            "#,
            active,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            Announcement,
            r#"
            SELECT id, title, content,
                   COALESCE(announcement_type, '') AS "announcement_type!" ,
                   COALESCE(priority, 0) AS "priority!" ,
                   COALESCE(is_pinned, false) AS "is_pinned!" ,
                   COALESCE(is_active, false) AS "is_active!" ,
                   start_time, end_time, created_by,
                   COALESCE(created_at, NOW()) AS "created_at!" ,
                   COALESCE(updated_at, NOW()) AS "updated_at!"
            FROM announcements
            WHERE ($1::boolean IS NULL OR (is_active = $1 AND (start_time IS NULL OR start_time <= NOW())
                AND (end_time IS NULL OR end_time >= NOW())))
            ORDER BY is_pinned DESC, priority DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            active,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((rows, count))
    }

    async fn get_active(&self) -> Result<Vec<Announcement>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Announcement,
            r#"
            SELECT id, title, content,
                   COALESCE(announcement_type, '') AS "announcement_type!" ,
                   COALESCE(priority, 0) AS "priority!" ,
                   COALESCE(is_pinned, false) AS "is_pinned!" ,
                   COALESCE(is_active, false) AS "is_active!" ,
                   start_time, end_time, created_by,
                   COALESCE(created_at, NOW()) AS "created_at!" ,
                   COALESCE(updated_at, NOW()) AS "updated_at!"
            FROM announcements 
            WHERE is_active = true 
            AND (start_time IS NULL OR start_time <= NOW()) 
            AND (end_time IS NULL OR end_time >= NOW())
            ORDER BY is_pinned DESC, priority DESC, created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
