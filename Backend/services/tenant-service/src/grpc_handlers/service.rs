//! 服务实现与 TenantService trait 实现
use super::*;
use super::tenant_handlers::*;

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
