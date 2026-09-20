//! 登录日志管理操作
use super::AnnouncementRepository;
use super::types::*;

impl AnnouncementRepository {
    // ==================== 登录日志管理 ====================

    /// 记录登录日志
    pub(crate) async fn create_login_log(
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
               RETURNING id" ,
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
    pub(crate) async fn list_login_logs(
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
                      login_status::int AS "login_status!" ,
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
            "SELECT COUNT(*) as count FROM sys_login_logs WHERE ($1::bigint IS NULL OR user_id = $1) AND ($2::integer IS NULL OR login_status = $2)" ,
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

}
