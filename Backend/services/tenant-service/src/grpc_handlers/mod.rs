//! gRPC Service Handlers for Tenant Service
//!
//! 提供租户管理的 gRPC 接口

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tonic::Status;
use std::net::SocketAddr;

use crate::repository::{TenantDetail, TenantListItem, TenantRepository, TenantUserRecord};
use crate::lifecycle::{TenantLifecycleService, TenantState};
use grpc_proto::tenant::tenant_service_server::TenantServiceServer;

/// Tenant 应用状态
#[derive(Clone)]
pub struct TenantAppState {
    pub repository: TenantRepository,
    pub lifecycle: TenantLifecycleService,
}

/// 租户信息 gRPC 响应结构
#[derive(Debug, Clone)]
pub struct TenantInfo {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub max_users: i32,
    pub max_storage: i64,
    pub status: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TenantDetail> for TenantInfo {
    fn from(t: TenantDetail) -> Self {
        Self {
            id: t.id,
            name: t.name,
            code: t.code,
            domain: t.domain,
            description: t.description,
            max_users: t.max_users,
            max_storage: t.max_storage,
            status: t.status,
            expires_at: t.expires_at,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

/// 租户列表项
#[derive(Debug, Clone)]
pub struct TenantListItemInfo {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub status: i32,
    pub max_users: i64,
    pub current_users: i64,
    pub created_at: DateTime<Utc>,
}

impl From<TenantListItem> for TenantListItemInfo {
    fn from(t: TenantListItem) -> Self {
        Self {
            id: t.id,
            name: t.name,
            code: t.code,
            status: t.status,
            max_users: t.max_users,
            current_users: t.current_users,
            created_at: t.created_at,
        }
    }
}

/// 分页租户响应
#[derive(Debug, Clone)]
pub struct PaginatedTenantsInfo {
    pub tenants: Vec<TenantListItemInfo>,
    pub total: i64,
}

/// 租户用户信息
#[derive(Debug, Clone)]
pub struct TenantUserInfo {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub department: Option<String>,
    pub position: Option<String>,
    pub status: i32,
    pub joined_at: String,
}

impl From<TenantUserRecord> for TenantUserInfo {
    fn from(u: TenantUserRecord) -> Self {
        Self {
            id: u.id,
            user_id: u.user_id,
            username: u.username,
            email: u.email,
            role: u.role,
            department: u.department,
            position: u.position,
            status: u.status,
            joined_at: u.joined_at,
        }
    }
}

/// 使用统计信息
#[derive(Debug, Clone)]
pub struct UsageStatsInfo {
    pub total_users: i64,
    pub active_users: i64,
    pub used_storage: i64,
    pub max_storage: i64,
    pub monthly_api_calls: i64,
    pub api_call_limit: i64,
}


mod lifecycle_handlers;
mod service;
mod tenant_handlers;

pub use lifecycle_handlers::*;
pub use service::TenantGrpcService;
pub use tenant_handlers::*;
