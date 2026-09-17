import os

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

content = '''//! 审计数据库操作

use common::AppError;
use common::AppResult;
use crate::models::{CreateLoginLog, SysLoginLog, LoginStatistics, CreateOperationLog, SysOperationLog, ApiCallLogQuery, ApiCallLog, ApiCallStatistics, ApiEndpointStatistics, ApiTrendPoint, ApiResponseTimeDistribution, CreateApiCallLog};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 审计仓储
#[derive(Clone)]
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    /// 创建审计仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取数据库连接池引用
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ============ 登录日志 ============

    /// 插入登录日志
    pub async fn insert_login_log(&self, log: &CreateLoginLog) -> AppResult<i64> {
        let result = sqlx::query_scalar::<_, i64>(r#"
            INSERT INTO sys_login_logs (
                user_id, username, ip_address, user_agent,
                login_location, login_status, fail_reason, login_type
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#)
            .bind(log.user_id)
            .bind(log.username.as_deref())
            .bind(log.ip_address.as_deref())
            .bind(log.user_agent.as_deref())
            .bind(log.login_location.as_deref())
            .bind(log.login_status)
            .bind(log.fail_reason.as_deref())
            .bind(log.login_type.as_deref())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// 查询登录日志列表
    pub async fn find_login_logs(
        &self,
        page: u32,
        page_size: u32,
        username: Option<&str>,
        status: Option<i16>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> AppResult<(Vec<SysLoginLog>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;
        let start_dt = start_date.and_then(parse_datetime);
        let end_dt = end_date.and_then(parse_datetime);

        let total: i64 = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM sys_login_logs
               WHERE ($1 = \\'\\' OR username ILIKE \\'%\\' || $1 || \\'%\\')
                 AND ($2::smallint IS NULL OR login_status = $2)
                 AND ($3::timestamptz IS NULL OR created_at >= $3)
                 AND ($4::timestamptz IS NULL OR created_at <= $4)"#)
            .bind(username.unwrap_or(" "))
            .bind(status)
            .bind(start_dt)
            .bind(end_dt)
        .fetch_one(&self.pool)
        .await?;

        let logs = sqlx::query_as::<_, SysLoginLog>(r#"
            SELECT id, user_id, username, ip_address, user_agent,
                   login_location, login_status, fail_reason, login_type, created_at
            FROM sys_login_logs
            WHERE ($1 = \\'\\' OR username ILIKE \\'%\\' || $1 || \\'%\\')
              AND ($2::smallint IS NULL OR login_status = $2)
              AND ($3::timestamptz IS NULL OR created_at >= $3)
              AND ($4::timestamptz IS NULL OR created_at <= $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#)
            .bind(username.unwrap_or(" "))
            .bind(status)
            .bind(start_dt)
            .bind(end_dt)
            .bind(i64::from(page_size))
            .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }

    /// 获取登录统计
    pub async fn get_login_statistics(&self) -> AppResult<LoginStatistics> {
        let total_count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_login_logs")
            .fetch_one(&self.pool)
            .await?;

        let success_count: i64 =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 1")
                .fetch_one(&self.pool)
                .await?;

        let fail_count: i64 =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 2")
                .fetch_one(&self.pool)
                .await?;

        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc))
            .ok_or_else(|| AppError::InvalidParam("Invalid time".to_string()))?;

        let today_count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_login_logs WHERE created_at >= $1")
            .bind(today_start)
            .fetch_one(&self.pool)
            .await?;

        let today_success: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 1 AND created_at >= $1")
            .bind(today_start)
            .fetch_one(&self.pool)
            .await?;

        let today_fail: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 2 AND created_at >= $1")
            .bind(today_start)
            .fetch_one(&self.pool)
            .await?;

        Ok(LoginStatistics {
            total_count,
            success_count,
            fail_count,
            today_count,
            today_success,
            today_fail,
        })
    }
}
'''

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
