    // ============ 权限管理扩展 ============

    async fn get_role_permission_config(
        &self,
        _request: Request<GetRolePermissionConfigRequest>,
    ) -> Result<Response<GetRolePermissionConfigResponse>, Status> {
        Ok(Response::new(GetRolePermissionConfigResponse {
            data_permissions: vec![],
            field_permissions: vec![],
        }))
    }

    async fn update_role_permission_config(
        &self,
        _request: Request<UpdateRolePermissionConfigRequest>,
    ) -> Result<Response<UpdateRolePermissionConfigResponse>, Status> {
        Ok(Response::new(UpdateRolePermissionConfigResponse { success: true }))
    }

    async fn get_role_data_permissions(
        &self,
        _request: Request<GetRoleDataPermissionsRequest>,
    ) -> Result<Response<GetRoleDataPermissionsResponse>, Status> {
        Ok(Response::new(GetRoleDataPermissionsResponse {
            data_permissions: vec![],
        }))
    }

    async fn set_role_data_permissions(
        &self,
        _request: Request<SetRoleDataPermissionsRequest>,
    ) -> Result<Response<SetRoleDataPermissionsResponse>, Status> {
        Ok(Response::new(SetRoleDataPermissionsResponse { success: true }))
    }

    async fn get_role_field_permissions(
        &self,
        _request: Request<GetRoleFieldPermissionsRequest>,
    ) -> Result<Response<GetRoleFieldPermissionsResponse>, Status> {
        Ok(Response::new(GetRoleFieldPermissionsResponse {
            field_permissions: vec![],
        }))
    }

    async fn set_role_field_permissions(
        &self,
        _request: Request<SetRoleFieldPermissionsRequest>,
    ) -> Result<Response<SetRoleFieldPermissionsResponse>, Status> {
        Ok(Response::new(SetRoleFieldPermissionsResponse { success: true }))
    }

    async fn get_role_inherit_chain(
        &self,
        _request: Request<GetRoleInheritChainRequest>,
    ) -> Result<Response<GetRoleInheritChainResponse>, Status> {
        Ok(Response::new(GetRoleInheritChainResponse { info: None }))
    }

    async fn set_role_inherit(
        &self,
        _request: Request<SetRoleInheritRequest>,
    ) -> Result<Response<SetRoleInheritResponse>, Status> {
        Ok(Response::new(SetRoleInheritResponse { success: true }))
    }

    async fn remove_role_inherit(
        &self,
        _request: Request<RemoveRoleInheritRequest>,
    ) -> Result<Response<RemoveRoleInheritResponse>, Status> {
        Ok(Response::new(RemoveRoleInheritResponse { success: true }))
    }

    async fn get_accessible_departments(
        &self,
        _request: Request<GetAccessibleDepartmentsRequest>,
    ) -> Result<Response<GetAccessibleDepartmentsResponse>, Status> {
        Ok(Response::new(GetAccessibleDepartmentsResponse {
            departments: vec![],
            total: 0,
        }))
    }

    async fn get_accessible_tenants(
        &self,
        _request: Request<GetAccessibleTenantsRequest>,
    ) -> Result<Response<GetAccessibleTenantsResponse>, Status> {
        Ok(Response::new(GetAccessibleTenantsResponse {
            tenants: vec![],
            total: 0,
        }))
    }

    async fn list_permission_definitions(
        &self,
        _request: Request<ListPermissionDefinitionsRequest>,
    ) -> Result<Response<ListPermissionDefinitionsResponse>, Status> {
        Ok(Response::new(ListPermissionDefinitionsResponse {
            permissions: vec![],
            total: 0,
        }))
    }

    async fn create_permission_definition(
        &self,
        _request: Request<CreatePermissionDefinitionRequest>,
    ) -> Result<Response<CreatePermissionDefinitionResponse>, Status> {
        Ok(Response::new(CreatePermissionDefinitionResponse {
            permission: None,
        }))
    }

    async fn update_permission_definition(
        &self,
        _request: Request<UpdatePermissionDefinitionRequest>,
    ) -> Result<Response<UpdatePermissionDefinitionResponse>, Status> {
        Ok(Response::new(UpdatePermissionDefinitionResponse { success: true }))
    }

    async fn delete_permission_definition(
        &self,
        _request: Request<DeletePermissionDefinitionRequest>,
    ) -> Result<Response<DeletePermissionDefinitionResponse>, Status> {
        Ok(Response::new(DeletePermissionDefinitionResponse { success: true }))
    }

    async fn batch_create_permission_definitions(
        &self,
        _request: Request<BatchCreatePermissionDefinitionsRequest>,
    ) -> Result<Response<BatchCreatePermissionDefinitionsResponse>, Status> {
        Ok(Response::new(BatchCreatePermissionDefinitionsResponse { count: 0 }))
    }

    async fn batch_assign_permissions(
        &self,
        _request: Request<BatchAssignPermissionsRequest>,
    ) -> Result<Response<BatchAssignPermissionsResponse>, Status> {
        Ok(Response::new(BatchAssignPermissionsResponse { success: true, affected: 0 }))
    }

    async fn copy_role_permissions(
        &self,
        _request: Request<CopyRolePermissionsRequest>,
    ) -> Result<Response<CopyRolePermissionsResponse>, Status> {
        Ok(Response::new(CopyRolePermissionsResponse { success: true }))
    }

    async fn validate_data_permission(
        &self,
        _request: Request<ValidateDataPermissionRequest>,
    ) -> Result<Response<ValidateDataPermissionResponse>, Status> {
        Ok(Response::new(ValidateDataPermissionResponse { allowed: true }))
    }

    async fn check_sensitive_permission(
        &self,
        _request: Request<CheckSensitivePermissionRequest>,
    ) -> Result<Response<CheckSensitivePermissionResponse>, Status> {
        Ok(Response::new(CheckSensitivePermissionResponse { sensitive: false }))
    }
