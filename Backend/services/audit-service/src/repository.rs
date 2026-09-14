//! 审计数据库操作

use crate::error::{AuditError, AuditResult};
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
    pub async fn insert_login_log(&self, log: &CreateLoginLog) -> AuditResult<i64> {
        let result = sqlx::query_scalar!(
            r#"
            INSERT INTO sys_login_logs (
                user_id, username, ip_address, user_agent,
                login_location, login_status, fail_reason, login_type
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            log.user_id,
            log.username.as_deref(),
            log.ip_address.as_deref(),
            log.user_agent.as_deref(),
            log.login_location.as_deref(),
            log.login_status,
            log.fail_reason.as_deref(),
            log.login_type.as_deref(),
        )
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
    ) -> AuditResult<(Vec<SysLoginLog>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;
        let start_dt = start_date.and_then(parse_datetime);
        let end_dt = end_date.and_then(parse_datetime);

        // 查询总数
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM sys_login_logs
               WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
                 AND ($2::smallint IS NULL OR login_status = $2)
                 AND ($3::timestamptz IS NULL OR created_at >= $3)
                 AND ($4::timestamptz IS NULL OR created_at <= $4)"#,
            username.unwrap_or(""),
            status,
            start_dt,
            end_dt,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // 查询列表（LIMIT/OFFSET 使用条件参数之后的编号）
        let logs = sqlx::query_as!(
            SysLoginLog,
            r#"
            SELECT id, user_id, username, ip_address, user_agent,
                   login_location, login_status, fail_reason, login_type, created_at
            FROM sys_login_logs
            WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
              AND ($2::smallint IS NULL OR login_status = $2)
              AND ($3::timestamptz IS NULL OR created_at >= $3)
              AND ($4::timestamptz IS NULL OR created_at <= $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            username.unwrap_or(""),
            status,
            start_dt,
            end_dt,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }

    /// 获取登录统计
    pub async fn get_login_statistics(&self) -> AuditResult<LoginStatistics> {
        let total_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM sys_login_logs")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        let success_count: i64 =
            sqlx::query_scalar!("SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 1")
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0);

        let fail_count: i64 =
            sqlx::query_scalar!("SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 2")
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0);

        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc))
            .ok_or_else(|| AuditError::InvalidParam("无效的 HMS 时间".to_string()))?;
        let today_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_login_logs WHERE created_at >= $1",
            today_start,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let today_success: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 1 AND created_at >= $1",
            today_start,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let today_fail: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_login_logs WHERE login_status = 2 AND created_at >= $1",
            today_start,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(LoginStatistics {
            total_count,
            success_count,
            fail_count,
            today_count,
            today_success,
            today_fail,
        })
    }

    // ============ 操作日志 ============

    /// 插入操作日志
    pub async fn insert_operation_log(&self, log: &CreateOperationLog) -> AuditResult<i64> {
        let result = sqlx::query_scalar!(
            r#"
            INSERT INTO sys_operation_logs (
                user_id, username, module, business_type, method,
                request_method, request_url, request_params, request_body,
                response_data, status, error_msg, execution_time, ip_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#,
            log.user_id,
            log.username.as_deref(),
            log.module.as_deref(),
            log.business_type.as_deref(),
            log.method.as_deref(),
            log.request_method.as_deref(),
            log.request_url.as_deref(),
            log.request_params.as_deref(),
            log.request_body.as_deref(),
            log.response_data.as_deref(),
            log.status,
            log.error_msg.as_deref(),
            log.execution_time,
            log.ip_address.as_deref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// 查询操作日志列表
    #[allow(clippy::too_many_arguments)]
    pub async fn find_operation_logs(
        &self,
        page: u32,
        page_size: u32,
        username: Option<&str>,
        module: Option<&str>,
        business_type: Option<&str>,
        status: Option<i16>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> AuditResult<(Vec<SysOperationLog>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;
        let start_dt = start_date.and_then(parse_datetime);
        let end_dt = end_date.and_then(parse_datetime);

        // 查询总数
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM sys_operation_logs
               WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
                 AND ($2 = '' OR module = $2)
                 AND ($3 = '' OR business_type = $3)
                 AND ($4::smallint IS NULL OR status = $4)
                 AND ($5::timestamptz IS NULL OR created_at >= $5)
                 AND ($6::timestamptz IS NULL OR created_at <= $6)"#,
            username.unwrap_or(""),
            module.unwrap_or(""),
            business_type.unwrap_or(""),
            status,
            start_dt,
            end_dt,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // 查询列表
        let logs = sqlx::query_as!(
            SysOperationLog,
            r#"
            SELECT id, user_id, username, module, business_type, method,
                   request_method, request_url, request_params, request_body,
                   response_data, status, error_msg, execution_time, ip_address, created_at
            FROM sys_operation_logs
            WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
              AND ($2 = '' OR module = $2)
              AND ($3 = '' OR business_type = $3)
              AND ($4::smallint IS NULL OR status = $4)
              AND ($5::timestamptz IS NULL OR created_at >= $5)
              AND ($6::timestamptz IS NULL OR created_at <= $6)
            ORDER BY created_at DESC
            LIMIT $7 OFFSET $8
            "#,
            username.unwrap_or(""),
            module.unwrap_or(""),
            business_type.unwrap_or(""),
            status,
            start_dt,
            end_dt,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }

    /// 根据ID查询操作日志
    pub async fn find_operation_log_by_id(&self, id: i64) -> AuditResult<Option<SysOperationLog>> {
        let log = sqlx::query_as!(
            SysOperationLog,
            r#"
            SELECT id, user_id, username, module, business_type, method,
                   request_method, request_url, request_params, request_body,
                   response_data, status, error_msg, execution_time, ip_address, created_at
            FROM sys_operation_logs
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }

    /// 批量删除操作日志
    pub async fn batch_delete_operation_logs(&self, ids: &[i64]) -> AuditResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query!("DELETE FROM sys_operation_logs WHERE id = ANY($1)", ids)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    // ============ API 调用日志 ============

    /// 查询 API 调用日志列表
    pub async fn find_api_call_logs(
        &self,
        query: &ApiCallLogQuery,
    ) -> AuditResult<(Vec<ApiCallLog>, i64)> {
        let offset = (query.page.saturating_sub(1)) * query.page_size;
        let start_dt = query.start_date.as_deref().and_then(parse_datetime);
        let end_dt = query.end_date.as_deref().and_then(parse_datetime);
        let path_keyword = query.path_keyword.as_deref().map(|p| format!("%{p}%"));
        let (sc_low, sc_high) = match query.status_code {
            Some(2) => (Some(200), Some(300)),
            Some(4) => (Some(400), Some(500)),
            Some(5) => (Some(500), None),
            _ => (None, None),
        };
        let errors_only = query.errors_only.unwrap_or(false);

        // 查询总数
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM sys_api_call_logs
               WHERE ($1 = '' OR method = $1)
                 AND ($2 = '' OR path ILIKE '%' || $2 || '%')
                 AND ($3::int IS NULL OR response_time >= $3)
                 AND ($4::int IS NULL OR response_time <= $4)
                 AND ($5::timestamptz IS NULL OR created_at >= $5)
                 AND ($6::timestamptz IS NULL OR created_at <= $6)
                 AND ($7::int IS NULL OR status_code >= $7)
                 AND ($8::int IS NULL OR status_code < $8)
                 AND ($9 = false OR status_code >= 400)"#,
            query.method.as_deref().unwrap_or(""),
            path_keyword.as_deref().unwrap_or(""),
            query.min_response_time,
            query.max_response_time,
            start_dt,
            end_dt,
            sc_low,
            sc_high,
            errors_only,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // 查询列表
        let logs = sqlx::query_as!(
            ApiCallLog,
            r#"SELECT id, request_id, method, path, query_params, headers,
                      request_size, status_code, response_time, response_size,
                      client_ip, user_id, username, error, created_at
               FROM sys_api_call_logs
               WHERE ($1 = '' OR method = $1)
                 AND ($2 = '' OR path ILIKE '%' || $2 || '%')
                 AND ($3::int IS NULL OR response_time >= $3)
                 AND ($4::int IS NULL OR response_time <= $4)
                 AND ($5::timestamptz IS NULL OR created_at >= $5)
                 AND ($6::timestamptz IS NULL OR created_at <= $6)
                 AND ($7::int IS NULL OR status_code >= $7)
                 AND ($8::int IS NULL OR status_code < $8)
                 AND ($9 = false OR status_code >= 400)
               ORDER BY created_at DESC
               LIMIT $10 OFFSET $11"#,
            query.method.as_deref().unwrap_or(""),
            path_keyword.as_deref().unwrap_or(""),
            query.min_response_time,
            query.max_response_time,
            start_dt,
            end_dt,
            sc_low,
            sc_high,
            errors_only,
            i64::from(query.page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }

    /// 获取 API 调用统计
    pub async fn get_api_call_statistics(&self) -> AuditResult<ApiCallStatistics> {
        let total_calls: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
        let success_calls: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_api_call_logs WHERE status_code >= 200 AND status_code < 400",
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        let failed_calls = total_calls - success_calls;
        let error_rate = if total_calls > 0 {
            (failed_calls as f64 / total_calls as f64) * 100.0
        } else {
            0.0
        };
        let avg_response_time: Option<f64> = sqlx::query_scalar!(
            "SELECT AVG(response_time)::float8 FROM sys_api_call_logs",
        )
        .fetch_one(&self.pool)
        .await?;
        let max_response_time: Option<i64> = sqlx::query_scalar!(
            "SELECT MAX(response_time)::bigint FROM sys_api_call_logs",
        )
        .fetch_one(&self.pool)
        .await?;
        let min_response_time: Option<i64> = sqlx::query_scalar!(
            "SELECT MIN(response_time)::bigint FROM sys_api_call_logs",
        )
        .fetch_one(&self.pool)
        .await?;
        let total_data_size: Option<i64> = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(response_size), 0)::bigint FROM sys_api_call_logs",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ApiCallStatistics {
            total_calls,
            success_calls,
            failed_calls,
            error_rate,
            avg_response_time: avg_response_time.unwrap_or(0.0) as i64,
            p50_response_time: 0,
            p90_response_time: 0,
            p95_response_time: 0,
            p99_response_time: 0,
            max_response_time: max_response_time.unwrap_or(0),
            min_response_time: min_response_time.unwrap_or(0),
            total_data_size: total_data_size.unwrap_or(0),
            qps: 0.0,
        })
    }

    /// 获取 API 端点统计
    pub async fn get_api_endpoint_statistics(&self) -> AuditResult<Vec<ApiEndpointStatistics>> {
        let rows = sqlx::query!(
            r#"SELECT COALESCE(path, '') AS "path!",
                      COALESCE(method, '') AS "method!",
                      COUNT(*) AS "call_count!",
                      COUNT(*) FILTER (WHERE status_code >= 200 AND status_code < 400) AS "success_count!",
                      COUNT(*) FILTER (WHERE status_code >= 400) AS "failed_count!",
                      AVG(response_time)::float8 AS avg_rt,
                      MAX(response_time) AS max_rt
               FROM sys_api_call_logs
               GROUP BY path, method
               ORDER BY "call_count!" DESC
               LIMIT 50"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let error_rate = if r.call_count > 0 {
                    (r.failed_count as f64 / r.call_count as f64) * 100.0
                } else {
                    0.0
                };
                let category = categorize_path(&r.path);
                ApiEndpointStatistics {
                    path: r.path,
                    method: r.method,
                    call_count: r.call_count,
                    success_count: r.success_count,
                    failed_count: r.failed_count,
                    avg_response_time: r.avg_rt.unwrap_or(0.0) as i64,
                    p95_response_time: i64::from(r.max_rt.unwrap_or(0)),
                    error_rate,
                    category,
                }
            })
            .collect())
    }

    /// 获取 API 调用趋势（按小时）
    pub async fn get_api_call_trend(&self) -> AuditResult<Vec<ApiTrendPoint>> {
        let rows = sqlx::query!(
            r#"SELECT date_trunc('hour', created_at) AS ts,
                      COUNT(*) AS "call_count!",
                      COUNT(*) FILTER (WHERE status_code >= 400) AS "error_count!",
                      AVG(response_time)::float8 AS avg_rt,
                      MAX(response_time) AS max_rt
               FROM sys_api_call_logs
               WHERE created_at >= NOW() - INTERVAL '24 hours'
               GROUP BY ts
               ORDER BY ts"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ApiTrendPoint {
                timestamp: r.ts.map_or(0, |t| t.timestamp()),
                call_count: r.call_count,
                error_count: r.error_count,
                avg_response_time: r.avg_rt.unwrap_or(0.0) as i64,
                p95_response_time: i64::from(r.max_rt.unwrap_or(0)),
            })
            .collect())
    }

    /// 获取响应时间分布
    pub async fn get_api_response_distribution(
        &self,
    ) -> AuditResult<Vec<ApiResponseTimeDistribution>> {
        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
        if total == 0 {
            return Ok(vec![]);
        }

        let buckets = vec![
            ("< 100ms", 0, 100),
            ("100-300ms", 100, 300),
            ("300-500ms", 300, 500),
            ("500ms-1s", 500, 1000),
            ("1-2s", 1000, 2000),
            ("> 2s", 2000, i32::MAX),
        ];

        let mut result = Vec::new();
        for (label, min, max) in &buckets {
            let count: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM sys_api_call_logs WHERE response_time >= $1 AND response_time < $2",
                min,
                max,
            )
            .fetch_one(&self.pool).await?
            .unwrap_or(0);
            result.push(ApiResponseTimeDistribution {
                bucket: label.to_string(),
                min: *min,
                max: if *max == i32::MAX { -1 } else { *max },
                count,
                percentage: (count as f64 / total as f64) * 100.0,
            });
        }

        Ok(result)
    }

    /// 插入 API 调用日志
    pub async fn insert_api_call_log(&self, log: &CreateApiCallLog) -> AuditResult<i64> {
        let result = sqlx::query_scalar!(
            r#"INSERT INTO sys_api_call_logs
               (request_id, method, path, query_params, headers, request_size,
                status_code, response_time, response_size, client_ip, user_id, username, error)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
               RETURNING id"#,
            log.request_id.as_deref(),
            log.method.as_deref(),
            log.path.as_deref(),
            log.query_params.as_deref(),
            log.headers.as_deref(),
            log.request_size,
            log.status_code,
            log.response_time,
            log.response_size,
            log.client_ip.as_deref(),
            log.user_id,
            log.username.as_deref(),
            log.error.as_deref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}

/// 解析日期字符串（%Y-%m-%d %H:%M:%S 或 %Y-%m-%d）为 UTC 时间，解析失败返回 None
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
        .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc))
}

/// 根据路径分类 API 端点
fn categorize_path(path: &str) -> String {
    if path.contains("/user") || path.contains("/auth") {
        "用户管理".to_string()
    } else if path.contains("/role") || path.contains("/permission") {
        "角色权限".to_string()
    } else if path.contains("/cms") || path.contains("/article") {
        "内容管理".to_string()
    } else if path.contains("/message")
        || path.contains("/notification")
        || path.contains("/announcement")
    {
        "消息通知".to_string()
    } else if path.contains("/file") || path.contains("/upload") {
        "文件管理".to_string()
    } else if path.contains("/log") || path.contains("/audit") {
        "日志审计".to_string()
    } else if path.contains("/workflow") || path.contains("/task") {
        "工作流".to_string()
    } else if path.contains("/report") || path.contains("/dashboard") {
        "报表统计".to_string()
    } else if path.contains("/config") || path.contains("/dict") || path.contains("/system") {
        "系统配置".to_string()
    } else {
        "其他".to_string()
    }
}
