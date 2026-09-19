//! 审计数据库操作

use common::AppError;
use common::AppResult;
use crate::models::{CreateLoginLog, SysLoginLog, LoginStatistics, CreateOperationLog, SysOperationLog, ApiCallLogQuery, ApiCallLog, ApiCallStatistics, ApiEndpointStatistics, ApiTrendPoint, ApiResponseTimeDistribution, CreateApiCallLog};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 查询操作日志参数
#[derive(Debug, Clone)]
pub struct FindOperationLogsParams<'a> {
    pub page: u32,
    pub page_size: u32,
    pub username: Option<&'a str>,
    pub module: Option<&'a str>,
    pub business_type: Option<&'a str>,
    pub status: Option<i16>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
}

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

        // 查询总数
        let total: i64 = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM sys_login_logs
               WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
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
            WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
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
            .ok_or_else(|| AppError::InvalidParam("无效的 HMS 时间".to_string()))?;

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

    // ============ 操作日志 ============

    /// 插入操作日志
    pub async fn insert_operation_log(&self, log: &CreateOperationLog) -> AppResult<i64> {
        let result = sqlx::query_scalar::<_, i64>(r#"
            INSERT INTO sys_operation_logs (
                user_id, username, module, business_type, method,
                request_method, request_url, request_params, request_body,
                response_data, status, error_msg, execution_time, ip_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#)
            .bind(log.user_id)
            .bind(log.username.as_deref())
            .bind(log.module.as_deref())
            .bind(log.business_type.as_deref())
            .bind(log.method.as_deref())
            .bind(log.request_method.as_deref())
            .bind(log.request_url.as_deref())
            .bind(log.request_params.as_deref())
            .bind(log.request_body.as_deref())
            .bind(log.response_data.as_deref())
            .bind(log.status)
            .bind(log.error_msg.as_deref())
            .bind(log.execution_time)
            .bind(log.ip_address.as_deref())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// 查询操作日志列表
    pub async fn find_operation_logs(
        &self,
        params: FindOperationLogsParams<'_>,
    ) -> AppResult<(Vec<SysOperationLog>, i64)> {
        let offset = (params.page.saturating_sub(1)) * params.page_size;
        let start_dt = params.start_date.and_then(parse_datetime);
        let end_dt = params.end_date.and_then(parse_datetime);

        let total: i64 = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM sys_operation_logs
               WHERE ($1 = '' OR username ILIKE '%' || $1 || '%')
                 AND ($2 = '' OR module = $2)
                 AND ($3 = '' OR business_type = $3)
                 AND ($4::smallint IS NULL OR status = $4)
                 AND ($5::timestamptz IS NULL OR created_at >= $5)
                 AND ($6::timestamptz IS NULL OR created_at <= $6)"#)
            .bind(params.username.unwrap_or(" "))
            .bind(params.module.unwrap_or(""))
            .bind(params.business_type.unwrap_or(""))
            .bind(params.status)
            .bind(start_dt)
            .bind(end_dt)
        .fetch_one(&self.pool)
        .await?;

        let logs = sqlx::query_as::<_, SysOperationLog>(r#"
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
            "#)
            .bind(params.username.unwrap_or(" "))
            .bind(params.module.unwrap_or(""))
            .bind(params.business_type.unwrap_or(""))
            .bind(params.status)
            .bind(start_dt)
            .bind(end_dt)
            .bind(i64::from(params.page_size))
            .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }

    /// 根据ID查询操作日志
    pub async fn find_operation_log_by_id(&self, id: i64) -> AppResult<Option<SysOperationLog>> {
        let log = sqlx::query_as::<_, SysOperationLog>(r#"
            SELECT id, user_id, username, module, business_type, method,
                   request_method, request_url, request_params, request_body,
                   response_data, status, error_msg, execution_time, ip_address, created_at
            FROM sys_operation_logs
            WHERE id = $1
            "#)
            .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }

    /// 批量删除操作日志
    pub async fn batch_delete_operation_logs(&self, ids: &[i64]) -> AppResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query("DELETE FROM sys_operation_logs WHERE id = ANY($1)")
            .bind(ids)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    // ============ API 调用日志 ============

    /// 查询 API 调用日志列表
    pub async fn find_api_call_logs(
        &self,
        query: &ApiCallLogQuery,
    ) -> AppResult<(Vec<ApiCallLog>, i64)> {
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

        let total: i64 = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM sys_api_call_logs
               WHERE ($1 = '' OR method = $1)
                 AND ($2 = '' OR path ILIKE '%' || $2 || '%')
                 AND ($3::int IS NULL OR response_time >= $3)
                 AND ($4::int IS NULL OR response_time <= $4)
                 AND ($5::timestamptz IS NULL OR created_at >= $5)
                 AND ($6::timestamptz IS NULL OR created_at <= $6)
                 AND ($7::int IS NULL OR status_code >= $7)
                 AND ($8::int IS NULL OR status_code < $8)
                 AND ($9 = false OR status_code >= 400)"#)
            .bind(query.method.as_deref().unwrap_or(" "))
            .bind(path_keyword.as_deref().unwrap_or(""))
            .bind(query.min_response_time)
            .bind(query.max_response_time)
            .bind(start_dt)
            .bind(end_dt)
            .bind(sc_low)
            .bind(sc_high)
            .bind(errors_only)
        .fetch_one(&self.pool)
        .await?;

        let logs = sqlx::query_as::<_, ApiCallLog>(r#"SELECT id, request_id, method, path, query_params, headers,
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
               LIMIT $10 OFFSET $11"#)
            .bind(query.method.as_deref().unwrap_or(" "))
            .bind(path_keyword.as_deref().unwrap_or(""))
            .bind(query.min_response_time)
            .bind(query.max_response_time)
            .bind(start_dt)
            .bind(end_dt)
            .bind(sc_low)
            .bind(sc_high)
            .bind(errors_only)
            .bind(i64::from(query.page_size))
            .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }

    /// 获取 API 调用统计
    pub async fn get_api_call_statistics(&self) -> AppResult<ApiCallStatistics> {
        let total_calls: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?;
        let success_calls: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_api_call_logs WHERE status_code >= 200 AND status_code < 400")
            .fetch_one(&self.pool)
            .await?;
        let failed_calls = total_calls - success_calls;
        let error_rate = if total_calls > 0 {
            (failed_calls as f64 / total_calls as f64) * 100.0
        } else {
            0.0
        };
        let avg_response_time: Option<f64> = sqlx::query_scalar::<_, Option<f64>>("SELECT AVG(response_time)::float8 FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?
            ;
        let max_response_time: Option<i64> = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(response_time)::bigint FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?
            ;
        let min_response_time: Option<i64> = sqlx::query_scalar::<_, Option<i64>>("SELECT MIN(response_time)::bigint FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?
            ;
        let total_data_size: Option<i64> = sqlx::query_scalar::<_, Option<i64>>("SELECT COALESCE(SUM(response_size), 0)::bigint FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?
            ;

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
    pub async fn get_api_endpoint_statistics(&self) -> AppResult<Vec<ApiEndpointStatistics>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64, i64, Option<f64>, Option<i64>)>(r#"
               SELECT COALESCE(path, '') AS path,
                      COALESCE(method, '') AS method,
                      COUNT(*) AS call_count,
                      COUNT(*) FILTER (WHERE status_code >= 200 AND status_code < 400) AS success_count,
                      COUNT(*) FILTER (WHERE status_code >= 400) AS failed_count,
                      AVG(response_time)::float8 AS avg_rt,
                      MAX(response_time) AS max_rt
               FROM sys_api_call_logs
               GROUP BY path, method
               ORDER BY call_count DESC
               LIMIT 50"#)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let error_rate = if r.2 > 0 {
                    (r.4 as f64 / r.2 as f64) * 100.0
                } else {
                    0.0
                };
                let category = categorize_path(&r.0);
                ApiEndpointStatistics {
                    path: r.0,
                    method: r.1,
                    call_count: r.2,
                    success_count: r.3,
                    failed_count: r.4,
                    avg_response_time: r.5.unwrap_or(0.0) as i64,
                    p95_response_time: i64::from(r.6.unwrap_or(0)),
                    error_rate,
                    category,
                }
            })
            .collect())
    }

    /// 获取 API 调用趋势（按小时）
    pub async fn get_api_call_trend(&self) -> AppResult<Vec<ApiTrendPoint>> {
        let rows = sqlx::query_as::<_, (Option<DateTime<Utc>>, i64, i64, Option<f64>, Option<i64>)>(r#"
               SELECT date_trunc('hour', created_at) AS ts,
                      COUNT(*) AS call_count,
                      COUNT(*) FILTER (WHERE status_code >= 400) AS error_count,
                      AVG(response_time)::float8 AS avg_rt,
                      MAX(response_time) AS max_rt
               FROM sys_api_call_logs
               WHERE created_at >= NOW() - INTERVAL '24 hours'
               GROUP BY ts
               ORDER BY ts"#)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| ApiTrendPoint {
                timestamp: r.0.map_or(0, |t| t.timestamp()),
                call_count: r.1,
                error_count: r.2,
                avg_response_time: r.3.unwrap_or(0.0) as i64,
                p95_response_time: i64::from(r.4.unwrap_or(0)),
            })
            .collect())
    }

    /// 获取响应时间分布
    pub async fn get_api_response_distribution(
        &self,
    ) -> AppResult<Vec<ApiResponseTimeDistribution>> {
        let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_api_call_logs")
            .fetch_one(&self.pool)
            .await?;
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
            let count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_api_call_logs WHERE response_time >= $1 AND response_time < $2")
                .bind(min)
                .bind(max)
                .fetch_one(&self.pool).await?;
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
    pub async fn insert_api_call_log(&self, log: &CreateApiCallLog) -> AppResult<i64> {
        let result = sqlx::query_scalar::<_, i64>(r#"INSERT INTO sys_api_call_logs
               (request_id, method, path, query_params, headers, request_size,
                status_code, response_time, response_size, client_ip, user_id, username, error)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
               RETURNING id"#)
            .bind(log.request_id.as_deref())
            .bind(log.method.as_deref())
            .bind(log.path.as_deref())
            .bind(log.query_params.as_deref())
            .bind(log.headers.as_deref())
            .bind(log.request_size)
            .bind(log.status_code)
            .bind(log.response_time)
            .bind(log.response_size)
            .bind(log.client_ip.as_deref())
            .bind(log.user_id)
            .bind(log.username.as_deref())
            .bind(log.error.as_deref())
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
