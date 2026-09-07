// ============== 租户服务 gRPC 调用封装 ==============

impl TenantGrpcClient {
    pub async fn list_tenants(
        &mut self,
        page: i32,
        page_size: i32,
        keyword: String,
        status: i32,
        plan: String,
    ) -> Result<grpc_proto::tenant::ListTenantsResponse, tonic::Status> {
        let request = grpc_proto::tenant::ListTenantsRequest {
            page, page_size, keyword,
            status: grpc_proto::tenant::TenantStatus::try_from(status)
                .unwrap_or(grpc_proto::tenant::TenantStatus::Active).into(),
            plan,
        };
        Ok(self.inner.list_tenants(request).await?.into_inner())
    }

    pub async fn get_tenant(
        &mut self,
        id: i64,
        code: String,
    ) -> Result<grpc_proto::tenant::GetTenantResponse, tonic::Status> {
        let request = grpc_proto::tenant::GetTenantRequest { id, code };
        Ok(self.inner.get_tenant(request).await?.into_inner())
    }

    pub async fn create_tenant(
        &mut self,
        name: String,
        code: String,
        logo: String,
        description: String,
        plan: String,
        max_users: i64,
        config: std::collections::HashMap<String, String>,
    ) -> Result<grpc_proto::tenant::CreateTenantResponse, tonic::Status> {
        let request = grpc_proto::tenant::CreateTenantRequest {
            name, code, logo, description, plan, max_users, config,
        };
        Ok(self.inner.create_tenant(request).await?.into_inner())
    }

    pub async fn update_tenant(
        &mut self,
        id: i64,
        name: String,
        logo: String,
        description: String,
        status: i32,
        plan: String,
        max_users: i64,
        expires_at: i64,
    ) -> Result<grpc_proto::tenant::UpdateTenantResponse, tonic::Status> {
        let request = grpc_proto::tenant::UpdateTenantRequest {
            id, name, logo, description,
            status: grpc_proto::tenant::TenantStatus::try_from(status)
                .unwrap_or(grpc_proto::tenant::TenantStatus::Active).into(),
            plan, max_users, expires_at,
        };
        Ok(self.inner.update_tenant(request).await?.into_inner())
    }

    pub async fn delete_tenant(
        &mut self,
        id: i64,
        force: bool,
    ) -> Result<grpc_proto::tenant::DeleteTenantResponse, tonic::Status> {
        let request = grpc_proto::tenant::DeleteTenantRequest { id, force };
        Ok(self.inner.delete_tenant(request).await?.into_inner())
    }

    pub async fn list_tenant_users(
        &mut self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<grpc_proto::tenant::ListTenantUsersResponse, tonic::Status> {
        let request = grpc_proto::tenant::ListTenantUsersRequest { tenant_id, page, page_size };
        Ok(self.inner.list_tenant_users(request).await?.into_inner())
    }

    pub async fn add_tenant_user(
        &mut self,
        tenant_id: i64,
        user_id: i64,
        role: String,
    ) -> Result<grpc_proto::tenant::AddTenantUserResponse, tonic::Status> {
        let request = grpc_proto::tenant::AddTenantUserRequest { tenant_id, user_id, role };
        Ok(self.inner.add_tenant_user(request).await?.into_inner())
    }

    pub async fn remove_tenant_user(
        &mut self,
        tenant_id: i64,
        user_id: i64,
    ) -> Result<grpc_proto::tenant::RemoveTenantUserResponse, tonic::Status> {
        let request = grpc_proto::tenant::RemoveTenantUserRequest { tenant_id, user_id };
        Ok(self.inner.remove_tenant_user(request).await?.into_inner())
    }

    pub async fn get_tenant_config(
        &mut self,
        tenant_id: i64,
    ) -> Result<grpc_proto::tenant::GetTenantConfigResponse, tonic::Status> {
        let request = grpc_proto::tenant::GetTenantConfigRequest { tenant_id };
        Ok(self.inner.get_tenant_config(request).await?.into_inner())
    }

    pub async fn update_tenant_config(
        &mut self,
        tenant_id: i64,
        config: std::collections::HashMap<String, String>,
    ) -> Result<grpc_proto::tenant::UpdateTenantConfigResponse, tonic::Status> {
        let request = grpc_proto::tenant::UpdateTenantConfigRequest { tenant_id, config };
        Ok(self.inner.update_tenant_config(request).await?.into_inner())
    }

    pub async fn get_usage_stats(
        &mut self,
        tenant_id: i64,
        period_start: i64,
        period_end: i64,
    ) -> Result<grpc_proto::tenant::GetUsageStatsResponse, tonic::Status> {
        let request = grpc_proto::tenant::GetUsageStatsRequest { tenant_id, period_start, period_end };
        Ok(self.inner.get_usage_stats(request).await?.into_inner())
    }
}

