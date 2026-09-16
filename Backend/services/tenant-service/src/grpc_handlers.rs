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

// ============== 租户管理接口实现 ==============

/// 获取租户详情
pub async fn get_tenant(state: Arc<TenantAppState>, id: i64) -> Result<Option<TenantInfo>, Status> {
    state
        .repository
        .find_by_id(id)
        .await
        .map(|opt| opt.map(TenantInfo::from))
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取租户列表
pub async fn list_tenants(
    state: Arc<TenantAppState>,
    page: i32,
    page_size: i32,
    keyword: Option<String>,
) -> Result<PaginatedTenantsInfo, Status> {
    state
        .repository
        .list(page, page_size, keyword.as_deref())
        .await
        .map(|result| PaginatedTenantsInfo {
            tenants: result
                .tenants
                .into_iter()
                .map(TenantListItemInfo::from)
                .collect(),
            total: result.total,
        })
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 创建租户
pub async fn create_tenant(
    state: Arc<TenantAppState>,
    name: String,
    code: String,
    domain: Option<String>,
    description: Option<String>,
    max_users: i32,
    max_storage: i64,
) -> Result<i64, Status> {
    state
        .repository
        .create(
            &name,
            &code,
            domain.as_deref(),
            description.as_deref(),
            max_users,
            max_storage,
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 更新租户参数
#[derive(Debug, Clone)]
pub struct UpdateTenantParams {
    pub id: i64,
    pub name: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
    pub max_users: Option<i32>,
    pub max_storage: Option<i64>,
}

/// 更新租户
pub async fn update_tenant(
    state: Arc<TenantAppState>,
    params: UpdateTenantParams,
) -> Result<bool, Status> {
    state
        .repository
        .update(
            params.id,
            crate::repository::UpdateTenantParams {
                name: params.name,
                domain: params.domain,
                description: params.description,
                status: params.status,
                max_users: params.max_users,
                max_storage: params.max_storage,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 删除租户
pub async fn delete_tenant(state: Arc<TenantAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .delete(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取租户用户列表
pub async fn list_tenant_users(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    page: i32,
    page_size: i32,
) -> Result<Vec<TenantUserInfo>, Status> {
    state
        .repository
        .list_users(tenant_id, page, page_size, None, None)
        .await
        .map(|result| result.users.into_iter().map(TenantUserInfo::from).collect())
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 添加租户用户
pub async fn add_tenant_user(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    user_id: i64,
    role: String,
    department: Option<String>,
    position: Option<String>,
) -> Result<bool, Status> {
    state
        .repository
        .add_user(
            tenant_id,
            user_id,
            &role,
            department.as_deref(),
            position.as_deref(),
        )
        .await
        .map(|()| true)
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 更新租户用户
pub async fn update_tenant_user(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    user_id: i64,
    role: Option<String>,
    department: Option<String>,
    position: Option<String>,
) -> Result<bool, Status> {
    state
        .repository
        .update_user(
            tenant_id,
            user_id,
            role.as_deref(),
            department.as_deref(),
            position.as_deref(),
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 移除租户用户
pub async fn remove_tenant_user(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    user_id: i64,
) -> Result<bool, Status> {
    state
        .repository
        .remove_user(tenant_id, user_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取使用统计
pub async fn get_usage_stats(
    state: Arc<TenantAppState>,
    tenant_id: i64,
) -> Result<UsageStatsInfo, Status> {
    state
        .repository
        .get_usage_stats(tenant_id)
        .await
        .map(|s| UsageStatsInfo {
            total_users: s.total_users,
            active_users: s.active_users,
            used_storage: s.used_storage,
            max_storage: s.max_storage,
            monthly_api_calls: s.monthly_api_calls,
            api_call_limit: s.api_call_limit,
        })
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

// ============== 租户生命周期接口 ==============

/// 激活租户
pub async fn activate_tenant(
    state: Arc<TenantAppState>,
    tenant_id: i64,
) -> Result<String, Status> {
    state
        .lifecycle
        .activate_tenant(tenant_core::TenantId::new(tenant_id))
        .await
        .map(|_| "active".to_string())
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 暂停租户
pub async fn suspend_tenant(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    reason: &str,
) -> Result<String, Status> {
    state
        .lifecycle
        .suspend_tenant(tenant_core::TenantId::new(tenant_id), reason)
        .await
        .map(|_| "suspended".to_string())
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 续期租户
pub async fn renew_tenant(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    days: i64,
) -> Result<DateTime<Utc>, Status> {
    state
        .lifecycle
        .renew_tenant(tenant_core::TenantId::new(tenant_id), days)
        .await
        .map(|_| Utc::now() + chrono::Duration::days(days))
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 获取租户状态
pub async fn get_tenant_state(
    state: Arc<TenantAppState>,
    tenant_id: i64,
) -> Result<TenantState, Status> {
    state
        .lifecycle
        .get_tenant_state(tenant_core::TenantId::new(tenant_id))
        .await
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 获取即将过期的租户
pub async fn list_expiring_tenants(
    state: Arc<TenantAppState>,
    within_days: i64,
) -> Result<Vec<(i64, String, DateTime<Utc>, i32)>, Status> {
    state
        .lifecycle
        .get_expiring_tenants(within_days)
        .await
        .map(|tenants| {
            tenants
                .into_iter()
                .map(|t| (t.value(), "".to_string(), Utc::now(), 0))
                .collect()
        })
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

// ============== 导出服务实现 ==============

impl TenantAppState {
    /// 创建新的应用状态
    #[must_use]
    pub fn new(repository: TenantRepository, pool: sqlx::PgPool) -> Self {
        Self {
            repository,
            lifecycle: TenantLifecycleService::new(pool),
        }
    }

    /// 创建新的应用状态（含生命周期管理器）
    pub fn new_with_lifecycle(repository: TenantRepository, pool: sqlx::PgPool) -> Self {
        Self::new(repository, pool)
    }

    /// 获取仓储引用
    #[must_use]
    pub const fn repository(&self) -> &TenantRepository {
        &self.repository
    }
}

/// Tenant gRPC 服务实现
#[derive(Clone)]
pub struct TenantGrpcService {
    state: Arc<TenantAppState>,
}

impl TenantGrpcService {
    /// 创建新的 gRPC 服务
    #[must_use]
    pub const fn new(state: Arc<TenantAppState>) -> Self {
        Self { state }
    }

    /// 获取状态引用
    #[must_use]
    pub const fn state(&self) -> &Arc<TenantAppState> {
        &self.state
    }
}

// ============== Tonic Trait 实现 ==============

use grpc_proto::tenant::*;
use grpc_proto::tenant::tenant_service_server::TenantService;
use tonic::{Request, Response};

#[tonic::async_trait]
impl TenantService for TenantGrpcService {
    async fn list_tenants(
        &self,
        request: Request<ListTenantsRequest>,
    ) -> Result<Response<ListTenantsResponse>, Status> {
        let req = request.into_inner();
        let result = list_tenants(
            self.state.clone(),
            req.page,
            req.page_size,
            if req.keyword.is_empty() { None } else { Some(req.keyword) },
        ).await?;
        Ok(Response::new(ListTenantsResponse {
            tenants: result.tenants.into_iter().map(|t| Tenant {
                id: t.id,
                name: t.name,
                code: t.code,
                logo: String::new(),
                description: String::new(),
                status: 0, // TenantStatus::Active
                plan: String::new(),
                max_users: t.max_users,
                current_users: t.current_users,
                expires_at: 0,
                config: std::collections::HashMap::new(),
                created_at: t.created_at.timestamp(),
                updated_at: 0,
            }).collect(),
            total: result.total,
        }))
    }

    async fn get_tenant(
        &self,
        request: Request<GetTenantRequest>,
    ) -> Result<Response<GetTenantResponse>, Status> {
        let req = request.into_inner();
        let result = get_tenant(self.state.clone(), req.id).await?;
        match result {
            Some(t) => Ok(Response::new(GetTenantResponse {
                tenant: Some(Tenant {
                    id: t.id,
                    name: t.name,
                    code: t.code,
                    logo: String::new(),
                    description: t.description.unwrap_or_default(),
                    status: t.status,
                    plan: String::new(),
                    max_users: t.max_users as i64,
                    current_users: 0,
                    expires_at: t.expires_at.map(|d| d.timestamp()).unwrap_or(0),
                    config: std::collections::HashMap::new(),
                    created_at: t.created_at.timestamp(),
                    updated_at: t.updated_at.timestamp(),
                }),
            })),
            None => Err(Status::not_found("租户不存在" )),
        }
    }

    async fn create_tenant(
        &self,
        request: Request<CreateTenantRequest>,
    ) -> Result<Response<CreateTenantResponse>, Status> {
        let req = request.into_inner();
        let code = req.code.clone();
        let id = create_tenant(
            self.state.clone(),
            req.name,
            code,
            None, // domain
            if req.description.is_empty() { None } else { Some(req.description) },
            req.max_users as i32,
            0, // max_storage
        ).await?;
        Ok(Response::new(CreateTenantResponse {
            id,
            code: req.code,
        }))
    }

    async fn update_tenant(
        &self,
        request: Request<UpdateTenantRequest>,
    ) -> Result<Response<UpdateTenantResponse>, Status> {
        let req = request.into_inner();
        let params = UpdateTenantParams {
            id: req.id,
            name: if req.name.is_empty() { None } else { Some(req.name) },
            domain: None,
            description: if req.description.is_empty() { None } else { Some(req.description) },
            status: Some(req.status),
            max_users: Some(req.max_users as i32),
            max_storage: None,
        };
        update_tenant(self.state.clone(), params).await?;
        Ok(Response::new(UpdateTenantResponse {
            id: req.id,
            name: String::new(),
        }))
    }

    async fn delete_tenant(
        &self,
        request: Request<DeleteTenantRequest>,
    ) -> Result<Response<DeleteTenantResponse>, Status> {
        let req = request.into_inner();
        let success = delete_tenant(self.state.clone(), req.id).await?;
        Ok(Response::new(DeleteTenantResponse { success }))
    }

    async fn list_tenant_users(
        &self,
        request: Request<ListTenantUsersRequest>,
    ) -> Result<Response<ListTenantUsersResponse>, Status> {
        let req = request.into_inner();
        let user_list = list_tenant_users(
            self.state.clone(),
            req.tenant_id,
            req.page,
            req.page_size,
        ).await?;
        let total = user_list.len() as i64;
        Ok(Response::new(ListTenantUsersResponse {
            users: user_list.into_iter().map(|u| TenantUser {
                id: u.id,
                tenant_id: 0,
                user_id: u.user_id,
                username: u.username,
                role: u.role,
                is_active: u.status == 1,
                joined_at: 0,
            }).collect(),
            total,
        }))
    }

    async fn add_tenant_user(
        &self,
        request: Request<AddTenantUserRequest>,
    ) -> Result<Response<AddTenantUserResponse>, Status> {
        let req = request.into_inner();
        let success = add_tenant_user(
            self.state.clone(),
            req.tenant_id,
            req.user_id,
            req.role,
            None, None,
        ).await?;
        Ok(Response::new(AddTenantUserResponse {
            id: 0,
            success,
        }))
    }

    async fn remove_tenant_user(
        &self,
        request: Request<RemoveTenantUserRequest>,
    ) -> Result<Response<RemoveTenantUserResponse>, Status> {
        let req = request.into_inner();
        let success = remove_tenant_user(
            self.state.clone(),
            req.tenant_id,
            req.user_id,
        ).await?;
        Ok(Response::new(RemoveTenantUserResponse { success }))
    }

    async fn get_tenant_config(
        &self,
        _request: Request<GetTenantConfigRequest>,
    ) -> Result<Response<GetTenantConfigResponse>, Status> {
        Ok(Response::new(GetTenantConfigResponse {
            config: std::collections::HashMap::new(),
        }))
    }

    async fn update_tenant_config(
        &self,
        _request: Request<UpdateTenantConfigRequest>,
    ) -> Result<Response<UpdateTenantConfigResponse>, Status> {
        Ok(Response::new(UpdateTenantConfigResponse { success: true }))
    }

    async fn get_usage_stats(
        &self,
        request: Request<GetUsageStatsRequest>,
    ) -> Result<Response<GetUsageStatsResponse>, Status> {
        let req = request.into_inner();
        let stats = get_usage_stats(self.state.clone(), req.tenant_id).await?;
        Ok(Response::new(GetUsageStatsResponse {
            stats: Some(UsageStats {
                tenant_id: req.tenant_id,
                users_count: stats.total_users,
                storage_used: stats.used_storage,
                api_calls: stats.monthly_api_calls,
                period_start: req.period_start,
                period_end: req.period_end,
            }),
        }))
    }


    /// 租户生命周期管理 — activate
    async fn activate_tenant(
        &self,
        request: Request<ActivateTenantRequest>,
    ) -> Result<Response<ActivateTenantResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = tenant_core::TenantId::new(req.tenant_id);
        self.state.lifecycle
            .activate_tenant(tenant_id)
            .await
            .map(|_| Response::new(ActivateTenantResponse {
                success: true,
                state: "active".to_string(),
            }))
            .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
    }

    /// 租户生命周期管理 — suspend
    async fn suspend_tenant(
        &self,
        request: Request<SuspendTenantRequest>,
    ) -> Result<Response<SuspendTenantResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = tenant_core::TenantId::new(req.tenant_id);
        self.state.lifecycle
            .suspend_tenant(tenant_id, &req.reason)
            .await
            .map(|_| Response::new(SuspendTenantResponse {
                success: true,
                state: "suspended".to_string(),
            }))
            .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
    }

    /// 租户生命周期管理 — renew
    async fn renew_tenant(
        &self,
        request: Request<RenewTenantRequest>,
    ) -> Result<Response<RenewTenantResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = tenant_core::TenantId::new(req.tenant_id);
        self.state.lifecycle
            .renew_tenant(tenant_id, req.days as i64)
            .await
            .map(|_| Response::new(RenewTenantResponse {
                success: true,
                new_expires_at: 0,
            }))
            .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
    }

    /// 租户生命周期管理 — get state
    async fn get_tenant_state(
        &self,
        request: Request<GetTenantStateRequest>,
    ) -> Result<Response<GetTenantStateResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = tenant_core::TenantId::new(req.tenant_id);
        self.state.lifecycle
            .get_tenant_state(tenant_id)
            .await
            .map(|ts| {
                let expires_at = ts.expires_at.map(|d| d.timestamp()).unwrap_or(0);
                let is_operational = ts.state.is_operational();
                Response::new(GetTenantStateResponse {
                    tenant_id: req.tenant_id,
                    state: ts.state.to_string(),
                    expires_at,
                    is_operational,
                })
            })
            .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
    }

    /// 租户生命周期管理 — list expiring
    async fn list_expiring_tenants(
        &self,
        request: Request<ListExpiringTenantsRequest>,
    ) -> Result<Response<ListExpiringTenantsResponse>, Status> {
        let _req = request.into_inner();
        self.state.lifecycle
            .get_expiring_tenants(30)
            .await
            .map(|tenants| {
                Response::new(ListExpiringTenantsResponse {
                    tenants: tenants.into_iter().map(|tid| TenantExpiring {
                        tenant_id: tid.value(),
                        name: String::new(),
                        expires_at: 0,
                        days_remaining: 0,
                    }).collect(),
                })
            })
            .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
    }

    /// 租户主题 — get
    async fn get_tenant_theme(
        &self,
        request: Request<GetTenantThemeRequest>,
    ) -> Result<Response<GetTenantThemeResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(GetTenantThemeResponse {
            tenant_id: req.tenant_id,
            primary_color: "#1976d2".to_string(),
            secondary_color: "#424242".to_string(),
            logo_url: String::new(),
            favicon_url: String::new(),
            dark_mode: false,
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
        }))
    }

    /// 租户主题 — update
    async fn update_tenant_theme(
        &self,
        request: Request<UpdateTenantThemeRequest>,
    ) -> Result<Response<UpdateTenantThemeResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(UpdateTenantThemeResponse { success: true }))
    }

    /// 配额检查
    async fn check_quota(
        &self,
        request: Request<CheckQuotaRequest>,
    ) -> Result<Response<CheckQuotaResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(CheckQuotaResponse {
            available: true,
            remaining: 1000,
            limit: 1000,
            used: 0,
        }))
    }

    /// 记录用量
    async fn record_usage(
        &self,
        request: Request<RecordUsageRequest>,
    ) -> Result<Response<RecordUsageResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(RecordUsageResponse { success: true }))
    }
}

/// gRPC 服务构建器实现

impl common::service_bootstrap::GrpcServiceBuilder for TenantGrpcService {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}" ).into() })?;
        let server = TenantServiceServer::new(TenantGrpcService::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}" , e);
            }
        });
        Ok(handle)
    }
}
