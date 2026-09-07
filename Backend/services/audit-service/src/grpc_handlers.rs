//! gRPC Service Handlers for Audit Service
//!
//! 提供审计日志管理的 gRPC 接口

use std::net::SocketAddr;
use grpc_proto::audit::audit_service_server::AuditServiceServer;
use std::sync::Arc;
use tonic::Status;

use crate::models::{LoginStatistics, SysLoginLog, SysOperationLog};
use crate::repository::AuditRepository;

/// Audit 应用状态
#[derive(Clone)]
pub struct AuditAppState {
    pub repository: AuditRepository,
}

/// 登录日志信息
#[derive(Debug, Clone)]
pub struct LoginLogInfo {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub login_location: Option<String>,
    pub login_status: Option<i16>,
    pub fail_reason: Option<String>,
    pub login_type: Option<String>,
    pub created_at: Option<String>,
}

impl From<SysLoginLog> for LoginLogInfo {
    fn from(log: SysLoginLog) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            username: log.username,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            login_location: log.login_location,
            login_status: log.login_status,
            fail_reason: log.fail_reason,
            login_type: log.login_type,
            created_at: log.created_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// 操作日志信息
#[derive(Debug, Clone)]
pub struct OperationLogInfo {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub module: Option<String>,
    pub business_type: Option<String>,
    pub method: Option<String>,
    pub request_method: Option<String>,
    pub request_url: Option<String>,
    pub request_params: Option<String>,
    pub status: Option<i16>,
    pub error_msg: Option<String>,
    pub execution_time: Option<i32>,
    pub ip_address: Option<String>,
    pub created_at: Option<String>,
}

impl From<SysOperationLog> for OperationLogInfo {
    fn from(log: SysOperationLog) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            username: log.username,
            module: log.module,
            business_type: log.business_type,
            method: log.method,
            request_method: log.request_method,
            request_url: log.request_url,
            request_params: log.request_params,
            status: log.status,
            error_msg: log.error_msg,
            execution_time: log.execution_time,
            ip_address: log.ip_address,
            created_at: log.created_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// 登录统计信息
#[derive(Debug, Clone)]
pub struct LoginStatisticsInfo {
    pub total_count: i64,
    pub success_count: i64,
    pub fail_count: i64,
    pub today_count: i64,
    pub today_success: i64,
    pub today_fail: i64,
}

impl From<LoginStatistics> for LoginStatisticsInfo {
    fn from(stats: LoginStatistics) -> Self {
        Self {
            total_count: stats.total_count,
            success_count: stats.success_count,
            fail_count: stats.fail_count,
            today_count: stats.today_count,
            today_success: stats.today_success,
            today_fail: stats.today_fail,
        }
    }
}

/// 分页登录日志响应
#[derive(Debug, Clone)]
pub struct PaginatedLoginLogsInfo {
    pub logs: Vec<LoginLogInfo>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// 分页操作日志响应
#[derive(Debug, Clone)]
pub struct PaginatedOperationLogsInfo {
    pub logs: Vec<OperationLogInfo>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

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
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
            params.page,
            params.page_size,
            params.username.as_deref(),
            params.module.as_deref(),
            params.business_type.as_deref(),
            status_i16,
            params.start_date.as_deref(),
            params.end_date.as_deref(),
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
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
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

// ============== 导出服务实现 ==============

impl AuditAppState {
    #[must_use]
    pub const fn new(repository: AuditRepository) -> Self {
        Self { repository }
    }

    #[must_use]
    pub const fn repository(&self) -> &AuditRepository {
        &self.repository
    }
}

/// Audit gRPC 服务实现
#[derive(Clone)]
pub struct AuditGrpcService {
    state: Arc<AuditAppState>,
}

impl AuditGrpcService {
    #[must_use]
    pub const fn new(state: Arc<AuditAppState>) -> Self {
        Self { state }
    }

    #[must_use]
    pub const fn state(&self) -> &Arc<AuditAppState> {
        &self.state
    }
}

#[tonic::async_trait]
impl grpc_proto::audit::audit_service_server::AuditService for AuditGrpcService {
    async fn log_action(
        &self,
        request: tonic::Request<grpc_proto::audit::LogActionRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::LogActionResponse>, tonic::Status> {
        let req = request.into_inner();
        let action_str = format!("{:?}", req.action());
        let params = CreateOperationLogParams {
            user_id: Some(req.user_id),
            username: Some(req.username.clone()),
            module: Some(req.resource_type.clone()),
            business_type: Some(action_str),
            method: None,
            request_method: None,
            request_url: None,
            request_params: None,
            request_body: None,
            response_data: None,
            status: 1,
            error_msg: None,
            execution_time: 0,
            ip_address: None,
        };
        let id = create_operation_log(self.state.clone(), params).await?;
        Ok(tonic::Response::new(grpc_proto::audit::LogActionResponse {
            id,
            success: true,
        }))
    }

    async fn list_logs(
        &self,
        request: tonic::Request<grpc_proto::audit::ListLogsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::ListLogsResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = ListOperationLogsParams {
            page: req.page as u32,
            page_size: req.page_size as u32,
            username: Some(req.keyword.clone()).filter(|s| !s.is_empty()),
            module: Some(req.resource_type.clone()).filter(|s| !s.is_empty()),
            business_type: None,
            status: None,
            start_date: Some(req.start_time.clone()).filter(|s| !s.is_empty()),
            end_date: Some(req.end_time.clone()).filter(|s| !s.is_empty()),
        };
        let result = list_operation_logs(self.state.clone(), params).await?;
        Ok(tonic::Response::new(grpc_proto::audit::ListLogsResponse {
            logs: vec![],
            total: result.total,
        }))
    }

    async fn get_log(
        &self,
        request: tonic::Request<grpc_proto::audit::GetLogRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::GetLogResponse>, tonic::Status> {
        let req = request.into_inner();
        let log = get_operation_log(self.state.clone(), req.id).await?;
        match log {
            Some(_) => Ok(tonic::Response::new(grpc_proto::audit::GetLogResponse {
                log: None,
            })),
            None => Err(tonic::Status::not_found("log not found")),
        }
    }

    async fn list_login_logs(
        &self,
        request: tonic::Request<grpc_proto::audit::ListLoginLogsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::ListLoginLogsResponse>, tonic::Status> {
        let req = request.into_inner();
        // 前端 status: 0=失败 1=成功 → DB login_status: 1=成功 2=失败
        let status_filter = match req.status {
            0 => Some(2),
            1 => Some(1),
            _ => None,
        };
        let result = list_login_logs(
            self.state.clone(),
            req.page as u32,
            req.page_size as u32,
            Some(req.keyword.clone()).filter(|s| !s.is_empty()),
            status_filter,
            Some(req.start_time.clone()).filter(|s| !s.is_empty()),
            Some(req.end_time.clone()).filter(|s| !s.is_empty()),
        ).await?;
        let logs: Vec<grpc_proto::audit::LoginLogInfo> = result.logs.into_iter().map(|l| {
            grpc_proto::audit::LoginLogInfo {
                id: l.id,
                user_id: l.user_id.unwrap_or(0),
                username: l.username.unwrap_or_default(),
                ip_address: l.ip_address.unwrap_or_default(),
                user_agent: l.user_agent.unwrap_or_default(),
                success: l.login_status.map(|s| s == 1).unwrap_or(false),
                fail_reason: l.fail_reason.unwrap_or_default(),
                login_method: l.login_type.unwrap_or_default(),
                created_at: l.created_at.map(|t| chrono::DateTime::parse_from_rfc3339(&t).map(|dt| dt.timestamp()).unwrap_or(0)).unwrap_or(0),
            }
        }).collect();
        Ok(tonic::Response::new(grpc_proto::audit::ListLoginLogsResponse {
            logs,
            total: result.total,
        }))
    }

    async fn create_login_log(
        &self,
        request: tonic::Request<grpc_proto::audit::CreateLoginLogRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::CreateLoginLogResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = CreateLoginLogParams {
            user_id: Some(req.user_id),
            username: Some(req.username.clone()),
            ip_address: Some(req.ip_address.clone()),
            user_agent: Some(req.user_agent.clone()),
            login_location: Some(req.login_location.clone()).filter(|s| !s.is_empty()),
            login_status: req.login_status,
            fail_reason: Some(req.fail_reason.clone()).filter(|s| !s.is_empty()),
            login_type: Some(req.login_type.clone()).filter(|s| !s.is_empty()),
        };
        let id = create_login_log(self.state.clone(), params).await?;
        Ok(tonic::Response::new(grpc_proto::audit::CreateLoginLogResponse {
            id,
            success: true,
        }))
    }

    async fn get_login_log(
        &self,
        _request: tonic::Request<grpc_proto::audit::GetLoginLogRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::GetLoginLogResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("get_login_log"))
    }

    async fn clear_login_logs(
        &self,
        _request: tonic::Request<grpc_proto::audit::ClearLoginLogsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::ClearLoginLogsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("clear_login_logs"))
    }

    async fn list_api_call_logs(
        &self,
        request: tonic::Request<grpc_proto::audit::ListApiCallLogsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::ListApiCallLogsResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = ListApiCallLogsParams {
            page: req.page as u32,
            page_size: req.page_size as u32,
            method: Some(req.method).filter(|s| !s.is_empty()),
            path_keyword: Some(req.path).filter(|s| !s.is_empty()),
            status_code: if req.status_code != 0 { Some(req.status_code) } else { None },
            start_date: Some(req.start_time).filter(|s| !s.is_empty()),
            end_date: Some(req.end_time).filter(|s| !s.is_empty()),
        };
        let (logs, total) = list_api_call_logs(self.state.clone(), params).await?;
        let api_logs: Vec<grpc_proto::audit::ApiCallLogInfo> = logs.into_iter().map(|log| {
            grpc_proto::audit::ApiCallLogInfo {
                id: log.id,
                user_id: log.user_id.unwrap_or(0),
                username: log.username.unwrap_or_default(),
                method: log.method.unwrap_or_default(),
                path: log.path.unwrap_or_default(),
                status_code: log.status_code.unwrap_or(0),
                duration_ms: log.response_time.unwrap_or(0) as i64,
                ip_address: log.client_ip.unwrap_or_default(),
                user_agent: String::new(),
                request_body: String::new(),
                response_body: String::new(),
                created_at: log.created_at.map(|t| t.timestamp()).unwrap_or(0),
            }
        }).collect();
        Ok(tonic::Response::new(grpc_proto::audit::ListApiCallLogsResponse {
            logs: api_logs,
            total,
        }))
    }

    async fn get_api_call_log(
        &self,
        request: tonic::Request<grpc_proto::audit::GetApiCallLogRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::GetApiCallLogResponse>, tonic::Status> {
        let req = request.into_inner();
        let log = get_api_call_log_by_id(self.state.clone(), req.id).await?;
        match log {
            Some(log) => Ok(tonic::Response::new(grpc_proto::audit::GetApiCallLogResponse {
                log: Some(grpc_proto::audit::ApiCallLogInfo {
                    id: log.id,
                    user_id: log.user_id.unwrap_or(0),
                    username: log.username.unwrap_or_default(),
                    method: log.method.unwrap_or_default(),
                    path: log.path.unwrap_or_default(),
                    status_code: log.status_code.unwrap_or(0),
                    duration_ms: log.response_time.unwrap_or(0) as i64,
                    ip_address: log.client_ip.unwrap_or_default(),
                    user_agent: String::new(),
                    request_body: String::new(),
                    response_body: String::new(),
                    created_at: log.created_at.map(|t| t.timestamp()).unwrap_or(0),
                }),
            })),
            None => Err(tonic::Status::not_found("api call log not found")),
        }
    }

    async fn get_stats(
        &self,
        _request: tonic::Request<grpc_proto::audit::GetStatsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::GetStatsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("get_stats"))
    }

    async fn archive_logs(
        &self,
        _request: tonic::Request<grpc_proto::audit::ArchiveLogsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::ArchiveLogsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("archive_logs"))
    }

    async fn export_logs(
        &self,
        _request: tonic::Request<grpc_proto::audit::ExportLogsRequest>,
    ) -> Result<tonic::Response<grpc_proto::audit::ExportLogsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("export_logs"))
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for AuditGrpcService {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().expect("invalid grpc addr");
        let server = AuditServiceServer::new(AuditGrpcService::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
