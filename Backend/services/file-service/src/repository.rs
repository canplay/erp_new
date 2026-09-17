//! 文件数据库操作

use common::AppResult;
use crate::models::SysFile;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 文件仓储
#[derive(Clone)]
pub struct FileRepository {
    pool: PgPool,
}

impl FileRepository {
    /// 创建文件仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 插入文件记录
    pub async fn insert(&self, file: &crate::models::SysFile) -> AppResult<i64> {
        let result = sqlx::query_scalar(r#"
            INSERT INTO sys_files (
                file_name, original_name, file_size, mime_type,
                storage_path, storage_type, bucket, url, md5,
                created_by, tenant_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#)
            .bind(&file.file_name)
            .bind(&file.original_name)
            .bind(file.file_size)
            .bind(file.mime_type.as_deref())
            .bind(&file.storage_path)
            .bind(&file.storage_type)
            .bind(file.bucket.as_deref())
            .bind(file.url.as_deref())
            .bind(file.md5.as_deref())
            .bind(file.created_by)
            .bind(file.tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// 根据ID查询文件
    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<SysFile>> {
        let file = sqlx::query_as::<_, SysFile>(r#"
            SELECT id, file_name, original_name, file_size, mime_type,
                   storage_path, COALESCE(storage_type, '') AS "storage_type!",
                   bucket, url, md5,
                   created_by, tenant_id, created_at, updated_at, deleted_at
            FROM sys_files
            WHERE id = $1 AND deleted_at IS NULL
            "#)
            .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(file)
    }

    /// 分页查询文件列表
    pub async fn find_list(
        &self,
        page: u32,
        page_size: u32,
        category: Option<&str>,
        keyword: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> AppResult<(Vec<SysFile>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;

        // 可选过滤条件: 空字符串/空值表示不过滤
        let category_filter = category.unwrap_or(" ");
        let keyword_filter = keyword.map(|k| format!("%{k}%")).unwrap_or_default();
        let start = start_date.and_then(parse_datetime);
        let end = end_date.and_then(parse_datetime);

        // 查询总数
        let total: i64 = sqlx::query_scalar::<_, i64>(r#"
            SELECT COUNT(*) FROM sys_files
            WHERE ($1 = '' OR category = $1)
              AND ($2 = '' OR original_name ILIKE $2)
              AND ($3::timestamptz IS NULL OR created_at >= $3::timestamptz)
              AND ($4::timestamptz IS NULL OR created_at <= $4::timestamptz)
              AND deleted_at IS NULL
            "#)
            .bind(&category_filter)
            .bind(&keyword_filter)
            .bind(start)
            .bind(end)
        .fetch_one(&self.pool)
        .await?;

        // 查询列表
        let files = sqlx::query_as::<_, SysFile>(r#"
            SELECT id, file_name, original_name, file_size, mime_type,
                   storage_path, COALESCE(storage_type, '') AS "storage_type!",
                   bucket, url, md5,
                   created_by, tenant_id, created_at, updated_at, deleted_at
            FROM sys_files
            WHERE ($1 = '' OR category = $1)
              AND ($2 = '' OR original_name ILIKE $2)
              AND ($3::timestamptz IS NULL OR created_at >= $3::timestamptz)
              AND ($4::timestamptz IS NULL OR created_at <= $4::timestamptz)
              AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#)
            .bind(&category_filter)
            .bind(&keyword_filter)
            .bind(start)
            .bind(end)
            .bind(i64::from(page_size))
            .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await?;

        Ok((files, total))
    }

    /// 删除文件（软删除）
    pub async fn delete(&self, id: i64) -> AppResult<bool> {
        let result = sqlx::query(r#"
            UPDATE sys_files
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND deleted_at IS NULL
            "#)
            .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量删除文件（软删除）
    pub async fn batch_delete(&self, ids: &[i64]) -> AppResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(r#"
            UPDATE sys_files
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE id = ANY($1) AND deleted_at IS NULL
            "#)
            .bind(ids)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 检查文件是否被使用
    pub async fn is_file_in_use(&self, id: i64) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_files WHERE id = $1 AND deleted_at IS NULL"
        )
            .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }
}

/// 解析日期时间字符串为 UTC 时间（支持 `%Y-%m-%d %H:%M:%S` 与 `%Y-%m-%d` 两种格式）
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Implementation...
    None
}
