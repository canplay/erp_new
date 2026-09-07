//! 公告和系统配置仓储层

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

/// 公告仓储错误类型
#[derive(Error, Debug)]
pub enum AnnouncementRepositoryError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("公告不存在")]
    NotFound,
}

/// 公告信息
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
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 公告列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementListItem {
    pub id: i64,
    pub title: String,
    pub announcement_type: String,
    pub priority: i32,
    pub is_pinned: bool,
    pub is_active: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 系统配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub id: i64,
    pub category: String,
    pub config_key: String,
    pub config_value: Option<String>,
    pub value_type: String,
    pub label: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 登录日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub login_status: i32,
    pub fail_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 分页结果
pub struct PaginatedAnnouncements {
    pub announcements: Vec<AnnouncementListItem>,
    pub total: i64,
}

pub struct PaginatedLoginLogs {
    pub logs: Vec<LoginLog>,
    pub total: i64,
}

/// 数据字典类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据字典项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryItem {
    pub id: i64,
    pub type_id: i64,
    pub label: String,
    pub value: String,
    pub sort: i32,
    pub status: i32,
    pub is_default: bool,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 字典类型分页
pub struct PaginatedDictionaryTypes {
    pub types: Vec<DictionaryType>,
    pub total: i64,
}

/// 字典项分页
pub struct PaginatedDictionaryItems {
    pub items: Vec<DictionaryItem>,
    pub total: i64,
}

/// 公告和配置仓储
#[derive(Clone)]
pub struct AnnouncementRepository {
    pool: PgPool,
}

impl AnnouncementRepository {
    /// 创建新的仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ==================== 公告管理 ====================

    /// 创建公告
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        title: &str,
        content: &str,
        announcement_type: &str,
        priority: i32,
        is_pinned: bool,
        is_active: bool,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        created_by: Option<i64>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO announcements
               (title, content, announcement_type, priority, is_pinned, is_active, start_time, end_time, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id",
            title,
            content,
            announcement_type,
            priority,
            is_pinned,
            is_active,
            start_time,
            end_time,
            created_by,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 获取公告详情
    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Announcement>, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r#"SELECT a.id, a.title, a.content,
                      COALESCE(a.announcement_type, '') AS "announcement_type!",
                      COALESCE(a.priority, 0) AS "priority!",
                      COALESCE(a.is_pinned, false) AS "is_pinned!",
                      COALESCE(a.is_active, false) AS "is_active!",
                      a.start_time, a.end_time,
                      a.created_by,
                      COALESCE(a.created_at, NOW()) AS "created_at!",
                      COALESCE(a.updated_at, NOW()) AS "updated_at!",
                      u.nickname as created_by_name
               FROM announcements a
               LEFT JOIN users u ON a.created_by = u.id
               WHERE a.id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Announcement {
            id: r.id,
            title: r.title,
            content: r.content,
            announcement_type: r.announcement_type,
            priority: r.priority,
            is_pinned: r.is_pinned,
            is_active: r.is_active,
            start_time: r.start_time,
            end_time: r.end_time,
            created_by: r.created_by,
            created_by_name: r.created_by_name,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 更新公告
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
        announcement_type: Option<String>,
        priority: Option<i32>,
        is_pinned: Option<bool>,
        is_active: Option<bool>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE announcements
               SET title = COALESCE($1, title),
                   content = COALESCE($2, content),
                   announcement_type = COALESCE($3, announcement_type),
                   priority = COALESCE($4, priority),
                   is_pinned = COALESCE($5, is_pinned),
                   is_active = COALESCE($6, is_active),
                   start_time = COALESCE($7, start_time),
                   end_time = COALESCE($8, end_time),
                   updated_at = NOW()
               WHERE id = $9",
            title.as_deref(),
            content.as_deref(),
            announcement_type.as_deref(),
            priority,
            is_pinned,
            is_active,
            start_time,
            end_time,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除公告
    pub async fn delete(&self, id: i64) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM announcements WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页查询公告列表
    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        is_active: Option<bool>,
    ) -> Result<PaginatedAnnouncements, AnnouncementRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            AnnouncementListItem,
            r#"SELECT a.id, a.title,
                      COALESCE(a.announcement_type, '') AS "announcement_type!",
                      COALESCE(a.priority, 0) AS "priority!",
                      COALESCE(a.is_pinned, false) AS "is_pinned!",
                      COALESCE(a.is_active, false) AS "is_active!",
                      a.start_time, a.end_time,
                      COALESCE(a.created_at, NOW()) AS "created_at!",
                      u.nickname as created_by_name
               FROM announcements a
               LEFT JOIN users u ON a.created_by = u.id
               WHERE ($1::boolean IS NULL OR a.is_active = $1)
               ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC
               LIMIT $2 OFFSET $3"#,
            is_active,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        // 获取总数
        let total_row = sqlx::query!(
            "SELECT COUNT(*) as count FROM announcements WHERE ($1::boolean IS NULL OR is_active = $1)",
            is_active,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let announcements: Vec<AnnouncementListItem> = rows
            .into_iter()
            .map(|row| AnnouncementListItem {
                id: row.id,
                title: row.title,
                announcement_type: row.announcement_type,
                priority: row.priority,
                is_pinned: row.is_pinned,
                is_active: row.is_active,
                start_time: row.start_time,
                end_time: row.end_time,
                created_by_name: row.created_by_name,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedAnnouncements {
            announcements,
            total,
        })
    }

    /// 获取活跃公告（公开接口）
    pub async fn get_active(
        &self,
    ) -> Result<Vec<AnnouncementListItem>, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            AnnouncementListItem,
            r#"SELECT a.id, a.title,
                      COALESCE(a.announcement_type, '') AS "announcement_type!",
                      COALESCE(a.priority, 0) AS "priority!",
                      COALESCE(a.is_pinned, false) AS "is_pinned!",
                      COALESCE(a.is_active, false) AS "is_active!",
                      a.start_time, a.end_time,
                      COALESCE(a.created_at, NOW()) AS "created_at!",
                      u.nickname as created_by_name
               FROM announcements a
               LEFT JOIN users u ON a.created_by = u.id
               WHERE a.is_active = TRUE
                 AND (a.start_time IS NULL OR a.start_time <= NOW())
                 AND (a.end_time IS NULL OR a.end_time >= NOW())
               ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let announcements: Vec<AnnouncementListItem> = rows
            .into_iter()
            .map(|row| AnnouncementListItem {
                id: row.id,
                title: row.title,
                announcement_type: row.announcement_type,
                priority: row.priority,
                is_pinned: row.is_pinned,
                is_active: row.is_active,
                start_time: row.start_time,
                end_time: row.end_time,
                created_by_name: row.created_by_name,
                created_at: row.created_at,
            })
            .collect();

        Ok(announcements)
    }

    // ==================== 系统配置管理 ====================

    /// 获取所有配置（按分类）
    pub async fn get_all_configs(&self) -> Result<Vec<SystemConfig>, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            SystemConfig,
            r#"SELECT id, category, config_key, config_value,
                      COALESCE(value_type, '') AS "value_type!",
                      COALESCE(label, '') AS "label!",
                      description,
                      COALESCE(sort_order, 0) AS "sort_order!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM system_configs
               WHERE status = 1
               ORDER BY category, sort_order"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let configs: Vec<SystemConfig> = rows
            .into_iter()
            .map(|row| SystemConfig {
                id: row.id,
                category: row.category,
                config_key: row.config_key,
                config_value: row.config_value,
                value_type: row.value_type,
                label: row.label,
                description: row.description,
                sort_order: row.sort_order,
                status: row.status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(configs)
    }

    /// 获取单个配置
    pub async fn get_config(
        &self,
        key: &str,
    ) -> Result<Option<SystemConfig>, AnnouncementRepositoryError> {
        let row = sqlx::query_as!(
            SystemConfig,
            r#"SELECT id, category, config_key, config_value,
                      COALESCE(value_type, '') AS "value_type!",
                      COALESCE(label, '') AS "label!",
                      description,
                      COALESCE(sort_order, 0) AS "sort_order!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM system_configs
               WHERE config_key = $1"#,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SystemConfig {
            id: r.id,
            category: r.category,
            config_key: r.config_key,
            config_value: r.config_value,
            value_type: r.value_type,
            label: r.label,
            description: r.description,
            sort_order: r.sort_order,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 更新配置
    pub async fn update_config(
        &self,
        key: &str,
        value: &str,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            "UPDATE system_configs SET config_value = $1, updated_at = NOW() WHERE config_key = $2",
            value,
            key,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 重置配置到默认值（从初始数据重新获取）
    pub async fn reset_config(&self, key: &str) -> Result<bool, AnnouncementRepositoryError> {
        // 这里假设有默认值存储，简化处理：设置为 NULL
        let result = sqlx::query!(
            "UPDATE system_configs SET config_value = NULL, updated_at = NOW() WHERE config_key = $1",
            key,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== 登录日志管理 ====================

    /// 记录登录日志
    pub async fn create_login_log(
        &self,
        user_id: Option<i64>,
        username: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        login_status: i32,
        fail_reason: Option<&str>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO sys_login_logs
               (user_id, username, ip_address, user_agent, login_status, fail_reason)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id",
            user_id,
            username,
            ip_address,
            user_agent,
            i16::try_from(login_status).unwrap_or(0),
            fail_reason,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 分页查询登录日志
    pub async fn list_login_logs(
        &self,
        page: i32,
        page_size: i32,
        user_id: Option<i64>,
        success: Option<bool>,
    ) -> Result<PaginatedLoginLogs, AnnouncementRepositoryError> {
        let offset = (page - 1).max(0) * page_size;
        let login_status = success.map(i32::from);

        let rows = sqlx::query_as!(
            LoginLog,
            r#"SELECT id, user_id, username, ip_address, user_agent,
                      login_status::int AS "login_status!",
                      fail_reason,
                      COALESCE(created_at, NOW()) AS "created_at!"
               FROM sys_login_logs
               WHERE ($1::bigint IS NULL OR user_id = $1)
                 AND ($2::integer IS NULL OR login_status = $2)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
            user_id,
            login_status,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        // 获取总数
        let total_row = sqlx::query!(
            "SELECT COUNT(*) as count FROM sys_login_logs WHERE ($1::bigint IS NULL OR user_id = $1) AND ($2::integer IS NULL OR login_status = $2)",
            user_id,
            login_status,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let logs: Vec<LoginLog> = rows
            .into_iter()
            .map(|row| LoginLog {
                id: row.id,
                user_id: row.user_id,
                username: row.username,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                login_status: row.login_status,
                fail_reason: row.fail_reason,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedLoginLogs { logs, total })
    }

    // ==================== 数据字典类型管理 ====================

    /// 分页查询字典类型
    pub async fn list_dictionary_types(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
        status: Option<i32>,
    ) -> Result<PaginatedDictionaryTypes, AnnouncementRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            DictionaryType,
            r#"SELECT id, code, name, description,
                      COALESCE(sort, 0) AS "sort!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM dictionary_types
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')
                 AND ($2::integer IS NULL OR status = $2)
               ORDER BY sort, created_at DESC
               LIMIT $3 OFFSET $4"#,
            keyword,
            status,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!(
            r"SELECT COUNT(*) as count FROM dictionary_types
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')
                 AND ($2::integer IS NULL OR status = $2)",
            keyword,
            status,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let types: Vec<DictionaryType> = rows
            .into_iter()
            .map(|row| DictionaryType {
                id: row.id,
                code: row.code,
                name: row.name,
                description: row.description,
                sort: row.sort,
                status: row.status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PaginatedDictionaryTypes { types, total })
    }

    /// 根据 ID 获取字典类型
    pub async fn find_dictionary_type_by_id(
        &self,
        id: i64,
    ) -> Result<Option<DictionaryType>, AnnouncementRepositoryError> {
        let row = sqlx::query_as!(
            DictionaryType,
            r#"SELECT id, code, name, description,
                      COALESCE(sort, 0) AS "sort!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM dictionary_types WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DictionaryType {
            id: r.id,
            code: r.code,
            name: r.name,
            description: r.description,
            sort: r.sort,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 创建字典类型
    pub async fn create_dictionary_type(
        &self,
        code: &str,
        name: &str,
        description: Option<&str>,
        sort: Option<i32>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO dictionary_types (code, name, description, sort, status)
               VALUES ($1, $2, $3, $4, 1)
               RETURNING id",
            code,
            name,
            description,
            sort,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 更新字典类型
    pub async fn update_dictionary_type(
        &self,
        id: i64,
        name: Option<String>,
        description: Option<String>,
        sort: Option<i32>,
        status: Option<i32>,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE dictionary_types
               SET name = COALESCE($1, name),
                   description = COALESCE($2, description),
                   sort = COALESCE($3, sort),
                   status = COALESCE($4, status),
                   updated_at = NOW()
               WHERE id = $5",
             name.as_deref(),
             description.as_deref(),
             sort,
            status,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除字典类型
    pub async fn delete_dictionary_type(&self, id: i64) -> Result<bool, AnnouncementRepositoryError> {
        // 先删除该类型下的所有字典项
        sqlx::query!("DELETE FROM dictionary_items WHERE type_id = $1", id)
            .execute(&self.pool)
            .await?;

        let result = sqlx::query!("DELETE FROM dictionary_types WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== 数据字典项管理 ====================

    /// 分页查询字典项
    pub async fn list_dictionary_items(
        &self,
        type_id: Option<i64>,
        type_code: Option<&str>,
        keyword: Option<&str>,
        status: Option<i32>,
    ) -> Result<PaginatedDictionaryItems, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            DictionaryItem,
            r#"SELECT di.id, di.type_id, di.label, di.value,
                      COALESCE(di.sort, 0) AS "sort!",
                      COALESCE(di.status, 1) AS "status!",
                      COALESCE(di.is_default, false) AS "is_default!",
                      di.remark,
                      COALESCE(di.created_at, NOW()) AS "created_at!",
                      COALESCE(di.updated_at, NOW()) AS "updated_at!"
               FROM dictionary_items di
               JOIN dictionary_types dt ON di.type_id = dt.id
               WHERE ($1::bigint IS NULL OR di.type_id = $1)
                 AND ($2::text IS NULL OR dt.code = $2)
                 AND ($3::text IS NULL OR di.label ILIKE '%' || $3 || '%' OR di.value ILIKE '%' || $3 || '%')
                 AND ($4::integer IS NULL OR di.status = $4)
               ORDER BY di.sort, di.created_at DESC"#,
            type_id,
            type_code,
            keyword,
            status,
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!(
            r"SELECT COUNT(*) as count
               FROM dictionary_items di
               JOIN dictionary_types dt ON di.type_id = dt.id
               WHERE ($1::bigint IS NULL OR di.type_id = $1)
                 AND ($2::text IS NULL OR dt.code = $2)
                 AND ($3::text IS NULL OR di.label ILIKE '%' || $3 || '%' OR di.value ILIKE '%' || $3 || '%')
                 AND ($4::integer IS NULL OR di.status = $4)",
            type_id,
            type_code,
            keyword,
            status,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let items: Vec<DictionaryItem> = rows
            .into_iter()
            .map(|row| DictionaryItem {
                id: row.id,
                type_id: row.type_id,
                label: row.label,
                value: row.value,
                sort: row.sort,
                status: row.status,
                is_default: row.is_default,
                remark: row.remark,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PaginatedDictionaryItems { items, total })
    }

    /// 根据 ID 获取字典项
    pub async fn find_dictionary_item_by_id(
        &self,
        id: i64,
    ) -> Result<Option<DictionaryItem>, AnnouncementRepositoryError> {
        let row = sqlx::query_as!(
            DictionaryItem,
            r#"SELECT id, type_id, label, value,
                      COALESCE(sort, 0) AS "sort!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(is_default, false) AS "is_default!",
                      remark,
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM dictionary_items WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DictionaryItem {
            id: r.id,
            type_id: r.type_id,
            label: r.label,
            value: r.value,
            sort: r.sort,
            status: r.status,
            is_default: r.is_default,
            remark: r.remark,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 创建字典项
    pub async fn create_dictionary_item(
        &self,
        type_id: i64,
        label: &str,
        value: &str,
        sort: Option<i32>,
        status: Option<i32>,
        is_default: Option<bool>,
        remark: Option<&str>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO dictionary_items (type_id, label, value, sort, status, is_default, remark)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id",
            type_id,
            label,
            value,
            sort,
            status,
            is_default,
            remark,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 更新字典项
    pub async fn update_dictionary_item(
        &self,
        id: i64,
        label: Option<String>,
        value: Option<String>,
        sort: Option<i32>,
        status: Option<i32>,
        is_default: Option<bool>,
        remark: Option<String>,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE dictionary_items
               SET label = COALESCE($1, label),
                   value = COALESCE($2, value),
                   sort = COALESCE($3, sort),
                   status = COALESCE($4, status),
                   is_default = COALESCE($5, is_default),
                   remark = COALESCE($6, remark),
                   updated_at = NOW()
               WHERE id = $7",
            label.as_deref(),
            value.as_deref(),
            sort,
            status,
            is_default,
            remark.as_deref(),
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除字典项
    pub async fn delete_dictionary_item(
        &self,
        id: i64,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!("DELETE FROM dictionary_items WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== 系统配置增强 ====================

    /// 获取系统配置列表（全量，支持分类过滤）
    pub async fn list_system_configs(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<SystemConfig>, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            SystemConfig,
            r#"SELECT id, category, config_key, config_value,
                      COALESCE(value_type, '') AS "value_type!",
                      COALESCE(label, '') AS "label!",
                      description,
                      COALESCE(sort_order, 0) AS "sort_order!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM system_configs
               WHERE ($1::text IS NULL OR category = $1)
                 AND status = 1
               ORDER BY category, sort_order"#,
            category,
        )
        .fetch_all(&self.pool)
        .await?;

        let configs: Vec<SystemConfig> = rows
            .into_iter()
            .map(|row| SystemConfig {
                id: row.id,
                category: row.category,
                config_key: row.config_key,
                config_value: row.config_value,
                value_type: row.value_type,
                label: row.label,
                description: row.description,
                sort_order: row.sort_order,
                status: row.status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(configs)
    }

    /// 批量更新系统配置
    pub async fn batch_update_system_configs(
        &self,
        configs: &[(String, String)],
    ) -> Result<bool, AnnouncementRepositoryError> {
        let mut tx = self.pool.begin().await?;

        for (key, value) in configs {
            sqlx::query!(
                "UPDATE system_configs SET config_value = $1, updated_at = NOW() WHERE config_key = $2",
                value,
                key,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(true)
    }
}
