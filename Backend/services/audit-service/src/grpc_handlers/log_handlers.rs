//! 日志查询处理器（登录/操作/API 调用）
use super::*;

// ============== 登录日志接口 ==============

/// 获取登录日志列表
pub async fn list_login_logs(
    state: Arc<AuditAppState>,
    page: u32,
    page_size: u32,
    username: Option<String>,
    status: Option<i32>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<PaginatedLoginLogsInfo, Status> {
    let status_i16 = status.map(|s| s as i16);
    let result = state
        .repository
        .find_login_logs(
            page,
            page_size,
            username.as_deref(),
            status_i16,
            start_date.as_deref(),
            end_date.as_deref(),
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    Ok(PaginatedLoginLogsInfo {
        logs: result.0.into_iter().map(LoginLogInfo::from).collect(),
        total: result.1,
        page,
        page_size,
    })
}

/// 获取登录统计
pub async fn get_login_statistics(
    state: Arc<AuditAppState>,
) -> Result<LoginStatisticsInfo, Status> {
    state
        .repository
        .get_login_statistics()
        .await
        .map(LoginStatisticsInfo::from)
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 创建登录日志参数
#[derive(Debug, Clone)]
pub struct CreateLoginLogParams {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub login_location: Option<String>,
    pub login_status: i32,
    pub fail_reason: Option<String>,
    pub login_type: Option<String>,
}

/// 创建登录日志（内部调用）
pub async fn create_login_log(
    state: Arc<AuditAppState>,
    params: CreateLoginLogParams,
) -> Result<i64, Status> {
    let log = crate::models::CreateLoginLog {
        user_id: params.user_id,
        username: params.username,
        ip_address: params.ip_address,
        user_agent: params.user_agent,
        login_location: params.login_location,
        login_status: params.login_status as i16,
        fail_reason: params.fail_reason,
        login_type: params.login_type,
    };

    state
        .repository
        .insert_login_log(&log)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

// ============== 操作日志接口 ==============

/// 获取操作日志列表参数
#[derive(Debug, Clone)]
pub struct ListOperationLogsParams {
    pub page: u32,
    pub page_size: u32,
    pub username: Option<String>,
    pub module: Option<String>,
    pub business_type: Option<String>,
    pub status: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 获取操作日志列表
pub async fn list_operation_logs(
    state: Arc<AuditAppState>,
    params: ListOperationLogsParams,
) -> Result<PaginatedOperationLogsInfo, Status> {
    let status_i16 = params.status.map(|s| s as i16);
    let result = state
        .repository
        .find_operation_logs(
            FindOperationLogsParams {
                page: params.page,
                page_size: params.page_size,
                username: params.username.as_deref(),
                module: params.module.as_deref(),
                business_type: params.business_type.as_deref(),
                status: status_i16,
                start_date: params.start_date.as_deref(),
                end_date: params.end_date.as_deref(),
            }
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    Ok(PaginatedOperationLogsInfo {
        logs: result.0.into_iter().map(OperationLogInfo::from).collect(),
        total: result.1,
        page: params.page,
        page_size: params.page_size,
    })
}

/// 获取操作日志详情
pub async fn get_operation_log(
    state: Arc<AuditAppState>,
    id: i64,
) -> Result<Option<OperationLogInfo>, Status> {
    state
        .repository
        .find_operation_log_by_id(id)
        .await
        .map(|opt| opt.map(OperationLogInfo::from))
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 创建操作日志参数
#[derive(Debug, Clone)]
pub struct CreateOperationLogParams {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub module: Option<String>,
    pub business_type: Option<String>,
    pub method: Option<String>,
    pub request_method: Option<String>,
    pub request_url: Option<String>,
    pub request_params: Option<String>,
    pub request_body: Option<String>,
    pub response_data: Option<String>,
    pub status: i32,
    pub error_msg: Option<String>,
    pub execution_time: i32,
    pub ip_address: Option<String>,
}

/// 创建操作日志（内部调用）
pub async fn create_operation_log(
    state: Arc<AuditAppState>,
    params: CreateOperationLogParams,
) -> Result<i64, Status> {
    let log = crate::models::CreateOperationLog {
        user_id: params.user_id,
        username: params.username,
        module: params.module,
        business_type: params.business_type,
        method: params.method,
        request_method: params.request_method,
        request_url: params.request_url,
        request_params: params.request_params,
        request_body: params.request_body,
        response_data: params.response_data,
        status: params.status as i16,
        error_msg: params.error_msg,
        execution_time: params.execution_time,
        ip_address: params.ip_address,
    };

    state
        .repository
        .insert_operation_log(&log)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 批量删除操作日志
pub async fn batch_delete_operation_logs(
    state: Arc<AuditAppState>,
    ids: Vec<i64>,
) -> Result<i64, Status> {
    state
        .repository
        .batch_delete_operation_logs(&ids)
        .await
        .map(|count| count as i64)
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

// ============== API 调用日志接口 ==============

/// 获取 API 调用日志列表参数
#[derive(Debug, Clone)]
pub struct ListApiCallLogsParams {
    pub page: u32,
    pub page_size: u32,
    pub method: Option<String>,
    pub path_keyword: Option<String>,
    pub status_code: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 获取 API 调用日志列表
pub async fn list_api_call_logs(
    state: Arc<AuditAppState>,
    params: ListApiCallLogsParams,
) -> Result<(Vec<crate::models::ApiCallLog>, i64), Status> {
    let query = crate::models::ApiCallLogQuery {
        page: params.page,
        page_size: params.page_size,
        method: params.method,
        path_keyword: params.path_keyword,
        status_code: params.status_code,
        start_date: params.start_date,
        end_date: params.end_date,
        ..Default::default()
    };
    state
        .repository
        .find_api_call_logs(&query)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取 API 调用日志详情
pub async fn get_api_call_log_by_id(
    state: Arc<AuditAppState>,
    id: i64,
) -> Result<Option<crate::models::ApiCallLog>, Status> {
    let sql = r"
        SELECT id, request_id, method, path, query_params, headers,
               request_size, status_code, response_time, response_size,
               client_ip, user_id, username, error, created_at
        FROM sys_api_call_logs
        WHERE id = $1
    ";
    sqlx::query_as::<_, crate::models::ApiCallLog>(sql)
        .bind(id)
        .fetch_optional(state.repository.pool())
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 创建 API 调用日志（内部调用）
pub async fn create_api_call_log(
    state: Arc<AuditAppState>,
    params: crate::models::CreateApiCallLog,
) -> Result<i64, Status> {
    state
        .repository
        .insert_api_call_log(&params)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

