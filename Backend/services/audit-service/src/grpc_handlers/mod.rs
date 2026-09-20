//! gRPC Service Handlers for Audit Service
//!
//! 提供审计日志管理的 gRPC 接口

use std::net::SocketAddr;
use grpc_proto::audit::audit_service_server::AuditServiceServer;
use std::sync::Arc;
use tonic::Status;

use crate::models::{LoginStatistics, SysLoginLog, SysOperationLog};
use crate::repository::{AuditRepository, FindOperationLogsParams};

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


mod log_handlers;
mod service;

pub use log_handlers::*;
pub use service::AuditGrpcService;
