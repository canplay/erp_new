// ============== 用户服务 gRPC 调用封装 ==============

impl UserGrpcClient {
    pub async fn get_user(
        &mut self,
        user_id: i64,
    ) -> Result<grpc_proto::user::GetUserResponse, tonic::Status> {
        let request = grpc_proto::user::GetUserRequest { user_id };
        Ok(self.inner.get_user(request).await?.into_inner())
    }

    pub async fn list_users(
        &mut self,
        page: i32,
        page_size: i32,
        keyword: String,
    ) -> Result<grpc_proto::user::ListUsersResponse, tonic::Status> {
        let request = grpc_proto::user::ListUsersRequest { page, page_size, keyword };
        Ok(self.inner.list_users(request).await?.into_inner())
    }

    pub async fn create_user(
        &mut self,
        username: String,
        password: String,
        email: String,
        nickname: String,
        phone: String,
        gender: i32,
    ) -> Result<grpc_proto::user::CreateUserResponse, tonic::Status> {
        let request = grpc_proto::user::CreateUserRequest {
            username, password, email, nickname, phone, gender,
        };
        Ok(self.inner.create_user(request).await?.into_inner())
    }

    pub async fn update_user(
        &mut self,
        user_id: i64,
        nickname: String,
        avatar: String,
        gender: i32,
        address: String,
    ) -> Result<grpc_proto::user::UpdateUserResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateUserRequest {
            user_id, nickname, avatar, gender, address,
        };
        Ok(self.inner.update_user(request).await?.into_inner())
    }

    pub async fn delete_user(
        &mut self,
        user_id: i64,
    ) -> Result<grpc_proto::user::DeleteUserResponse, tonic::Status> {
        let request = grpc_proto::user::DeleteUserRequest { user_id };
        Ok(self.inner.delete_user(request).await?.into_inner())
    }

    pub async fn update_user_status(
        &mut self,
        user_id: i64,
        status: i32,
        lock_hours: i32,
    ) -> Result<grpc_proto::user::UpdateUserStatusResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateUserStatusRequest { user_id, status, lock_hours };
        Ok(self.inner.update_user_status(request).await?.into_inner())
    }

    pub async fn update_user_role(
        &mut self,
        user_id: i64,
        role: String,
    ) -> Result<grpc_proto::user::UpdateUserRoleResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateUserRoleRequest { user_id, role };
        Ok(self.inner.update_user_role(request).await?.into_inner())
    }

    pub async fn reset_password(
        &mut self,
        user_id: i64,
        new_password: String,
    ) -> Result<grpc_proto::user::ResetPasswordResponse, tonic::Status> {
        let request = grpc_proto::user::ResetPasswordRequest { user_id, new_password };
        Ok(self.inner.reset_password(request).await?.into_inner())
    }

    pub async fn batch_update_user_role(
        &mut self,
        user_ids: Vec<i64>,
        role: String,
    ) -> Result<grpc_proto::user::BatchUpdateUserRoleResponse, tonic::Status> {
        let request = grpc_proto::user::BatchUpdateUserRoleRequest { user_ids, role };
        Ok(self.inner.batch_update_user_role(request).await?.into_inner())
    }

    pub async fn batch_update_user_status(
        &mut self,
        user_ids: Vec<i64>,
        status: i32,
    ) -> Result<grpc_proto::user::BatchUpdateUserStatusResponse, tonic::Status> {
        let request = grpc_proto::user::BatchUpdateUserStatusRequest { user_ids, status };
        Ok(self.inner.batch_update_user_status(request).await?.into_inner())
    }

    pub async fn batch_delete_users(
        &mut self,
        user_ids: Vec<i64>,
    ) -> Result<grpc_proto::user::BatchDeleteUsersResponse, tonic::Status> {
        let request = grpc_proto::user::BatchDeleteUsersRequest { user_ids };
        Ok(self.inner.batch_delete_users(request).await?.into_inner())
    }

    pub async fn list_roles(
        &mut self,
        page: i32,
        page_size: i32,
        keyword: String,
        r#type: String,
        status: i32,
    ) -> Result<grpc_proto::user::ListRolesResponse, tonic::Status> {
        let request = grpc_proto::user::ListRolesRequest { page, page_size, keyword, r#type, status };
        Ok(self.inner.list_roles(request).await?.into_inner())
    }

    pub async fn get_role(
        &mut self,
        name: String,
    ) -> Result<grpc_proto::user::GetRoleResponse, tonic::Status> {
        let request = grpc_proto::user::GetRoleRequest { name };
        Ok(self.inner.get_role(request).await?.into_inner())
    }

    pub async fn create_role(
        &mut self,
        name: String,
        description: String,
        r#type: String,
    ) -> Result<grpc_proto::user::CreateRoleResponse, tonic::Status> {
        let request = grpc_proto::user::CreateRoleRequest { name, description, r#type };
        Ok(self.inner.create_role(request).await?.into_inner())
    }

    pub async fn update_role(
        &mut self,
        name: String,
        description: String,
        r#type: String,
        status: i32,
    ) -> Result<grpc_proto::user::UpdateRoleResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateRoleRequest { name, description, r#type, status };
        Ok(self.inner.update_role(request).await?.into_inner())
    }

    pub async fn delete_role(
        &mut self,
        name: String,
    ) -> Result<grpc_proto::user::DeleteRoleResponse, tonic::Status> {
        let request = grpc_proto::user::DeleteRoleRequest { name };
        Ok(self.inner.delete_role(request).await?.into_inner())
    }

    pub async fn get_role_permissions(
        &mut self,
        name: String,
    ) -> Result<grpc_proto::user::GetRolePermissionsResponse, tonic::Status> {
        let request = grpc_proto::user::GetRolePermissionsRequest { name };
        Ok(self.inner.get_role_permissions(request).await?.into_inner())
    }

    pub async fn set_role_permissions(
        &mut self,
        name: String,
        permissions: Vec<String>,
    ) -> Result<grpc_proto::user::SetRolePermissionsResponse, tonic::Status> {
        let request = grpc_proto::user::SetRolePermissionsRequest { name, permissions };
        Ok(self.inner.set_role_permissions(request).await?.into_inner())
    }

    pub async fn get_role_users(
        &mut self,
        name: String,
        page: i32,
        page_size: i32,
    ) -> Result<grpc_proto::user::GetRoleUsersResponse, tonic::Status> {
        let request = grpc_proto::user::GetRoleUsersRequest { name, page, page_size };
        Ok(self.inner.get_role_users(request).await?.into_inner())
    }

    pub async fn get_role_permission_config(&mut self, role_name: String) -> Result<grpc_proto::user::GetRolePermissionConfigResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::GetRolePermissionConfigRequest { role_name });
        Ok(self.inner.get_role_permission_config(request).await?.into_inner())
    }
    pub async fn update_role_permission_config(&mut self, role_name: String, data_permissions: Vec<grpc_proto::user::DataPermissionInfo>, field_permissions: Vec<grpc_proto::user::FieldPermissionInfo>) -> Result<grpc_proto::user::UpdateRolePermissionConfigResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::UpdateRolePermissionConfigRequest { role_name, data_permissions, field_permissions });
        Ok(self.inner.update_role_permission_config(request).await?.into_inner())
    }
    pub async fn get_role_data_permissions(&mut self, role_name: String) -> Result<grpc_proto::user::GetRoleDataPermissionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::GetRoleDataPermissionsRequest { role_name });
        Ok(self.inner.get_role_data_permissions(request).await?.into_inner())
    }
    pub async fn set_role_data_permissions(&mut self, role_name: String, data_permissions: Vec<grpc_proto::user::DataPermissionInfo>) -> Result<grpc_proto::user::SetRoleDataPermissionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::SetRoleDataPermissionsRequest { role_name, data_permissions });
        Ok(self.inner.set_role_data_permissions(request).await?.into_inner())
    }
    pub async fn get_role_field_permissions(&mut self, role_name: String) -> Result<grpc_proto::user::GetRoleFieldPermissionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::GetRoleFieldPermissionsRequest { role_name });
        Ok(self.inner.get_role_field_permissions(request).await?.into_inner())
    }
    pub async fn set_role_field_permissions(&mut self, role_name: String, field_permissions: Vec<grpc_proto::user::FieldPermissionInfo>) -> Result<grpc_proto::user::SetRoleFieldPermissionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::SetRoleFieldPermissionsRequest { role_name, field_permissions });
        Ok(self.inner.set_role_field_permissions(request).await?.into_inner())
    }
    pub async fn get_role_inherit_chain(&mut self, role_name: String) -> Result<grpc_proto::user::GetRoleInheritChainResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::GetRoleInheritChainRequest { role_name });
        Ok(self.inner.get_role_inherit_chain(request).await?.into_inner())
    }
    pub async fn set_role_inherit(&mut self, role_name: String, inherit_from: Vec<String>) -> Result<grpc_proto::user::SetRoleInheritResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::SetRoleInheritRequest { role_name, inherit_from });
        Ok(self.inner.set_role_inherit(request).await?.into_inner())
    }
    pub async fn remove_role_inherit(&mut self, role_name: String) -> Result<grpc_proto::user::RemoveRoleInheritResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::RemoveRoleInheritRequest { role_name });
        Ok(self.inner.remove_role_inherit(request).await?.into_inner())
    }
    pub async fn get_accessible_departments(&mut self, role_name: String, page: i32, page_size: i32) -> Result<grpc_proto::user::GetAccessibleDepartmentsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::GetAccessibleDepartmentsRequest { role_name, page, page_size });
        Ok(self.inner.get_accessible_departments(request).await?.into_inner())
    }
    pub async fn get_accessible_tenants(&mut self, role_name: String, page: i32, page_size: i32) -> Result<grpc_proto::user::GetAccessibleTenantsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::GetAccessibleTenantsRequest { role_name, page, page_size });
        Ok(self.inner.get_accessible_tenants(request).await?.into_inner())
    }
    pub async fn list_permission_definitions(&mut self, category: String, page: i32, page_size: i32) -> Result<grpc_proto::user::ListPermissionDefinitionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::ListPermissionDefinitionsRequest { category, page, page_size });
        Ok(self.inner.list_permission_definitions(request).await?.into_inner())
    }
    pub async fn create_permission_definition(&mut self, key: String, name: String, description: String, category: String, entity_type: String, action: String, is_sensitive: bool, sort: i32) -> Result<grpc_proto::user::CreatePermissionDefinitionResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::CreatePermissionDefinitionRequest { key, name, description, category, entity_type, action, is_sensitive, sort });
        Ok(self.inner.create_permission_definition(request).await?.into_inner())
    }
    pub async fn update_permission_definition(&mut self, key: String, name: String, description: String, category: String, entity_type: String, action: String, is_sensitive: bool, sort: i32) -> Result<grpc_proto::user::UpdatePermissionDefinitionResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::UpdatePermissionDefinitionRequest { key, name, description, category, entity_type, action, is_sensitive, sort });
        Ok(self.inner.update_permission_definition(request).await?.into_inner())
    }
    pub async fn delete_permission_definition(&mut self, key: String) -> Result<grpc_proto::user::DeletePermissionDefinitionResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::DeletePermissionDefinitionRequest { key });
        Ok(self.inner.delete_permission_definition(request).await?.into_inner())
    }
    pub async fn batch_create_permission_definitions(&mut self, permissions: Vec<grpc_proto::user::CreatePermissionDefinitionRequest>) -> Result<grpc_proto::user::BatchCreatePermissionDefinitionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::BatchCreatePermissionDefinitionsRequest { permissions });
        Ok(self.inner.batch_create_permission_definitions(request).await?.into_inner())
    }
    pub async fn batch_assign_permissions(&mut self, role_name: String, permission_keys: Vec<String>) -> Result<grpc_proto::user::BatchAssignPermissionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::BatchAssignPermissionsRequest { role_name, permission_keys });
        Ok(self.inner.batch_assign_permissions(request).await?.into_inner())
    }
    pub async fn copy_role_permissions(&mut self, from_role: String, to_role: String) -> Result<grpc_proto::user::CopyRolePermissionsResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::CopyRolePermissionsRequest { from_role, to_role });
        Ok(self.inner.copy_role_permissions(request).await?.into_inner())
    }
    pub async fn validate_data_permission(&mut self, permission: String, entity_type: String, resource_owner: i64, resource_dept: i64) -> Result<grpc_proto::user::ValidateDataPermissionResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::ValidateDataPermissionRequest { permission, entity_type, resource_owner, resource_dept });
        Ok(self.inner.validate_data_permission(request).await?.into_inner())
    }
    pub async fn check_sensitive_permission(&mut self, permission: String) -> Result<grpc_proto::user::CheckSensitivePermissionResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::user::CheckSensitivePermissionRequest { permission });
        Ok(self.inner.check_sensitive_permission(request).await?.into_inner())
    }

    pub async fn list_departments(
        &mut self,
        page: i32,
        page_size: i32,
        keyword: String,
    ) -> Result<grpc_proto::user::ListDepartmentsResponse, tonic::Status> {
        let request = grpc_proto::user::ListDepartmentsRequest { page, page_size, keyword };
        Ok(self.inner.list_departments(request).await?.into_inner())
    }

    pub async fn get_department(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::GetDepartmentResponse, tonic::Status> {
        let request = grpc_proto::user::GetDepartmentRequest { id };
        Ok(self.inner.get_department(request).await?.into_inner())
    }

    pub async fn create_department(
        &mut self,
        name: String,
        parent_id: i64,
        description: String,
        sort_order: i32,
    ) -> Result<grpc_proto::user::CreateDepartmentResponse, tonic::Status> {
        let request = grpc_proto::user::CreateDepartmentRequest { name, parent_id, description, sort_order };
        Ok(self.inner.create_department(request).await?.into_inner())
    }

    pub async fn update_department(
        &mut self,
        id: i64,
        name: String,
        parent_id: i64,
        description: String,
        sort_order: i32,
        status: i32,
    ) -> Result<grpc_proto::user::UpdateDepartmentResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateDepartmentRequest { id, name, parent_id, description, sort_order, status };
        Ok(self.inner.update_department(request).await?.into_inner())
    }

    pub async fn delete_department(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::DeleteDepartmentResponse, tonic::Status> {
        let request = grpc_proto::user::DeleteDepartmentRequest { id };
        Ok(self.inner.delete_department(request).await?.into_inner())
    }

    pub async fn get_department_tree(
        &mut self,
        parent_id: i64,
    ) -> Result<grpc_proto::user::GetDepartmentTreeResponse, tonic::Status> {
        let request = grpc_proto::user::GetDepartmentTreeRequest { parent_id };
        Ok(self.inner.get_department_tree(request).await?.into_inner())
    }

    pub async fn get_department_users(
        &mut self,
        department_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<grpc_proto::user::GetDepartmentUsersResponse, tonic::Status> {
        let request = grpc_proto::user::GetDepartmentUsersRequest { department_id, page, page_size };
        Ok(self.inner.get_department_users(request).await?.into_inner())
    }

    pub async fn list_dictionary_types(
        &mut self,
        page: i32,
        page_size: i32,
        keyword: String,
        status: i32,
    ) -> Result<grpc_proto::user::ListDictionaryTypesResponse, tonic::Status> {
        let request = grpc_proto::user::ListDictionaryTypesRequest { page, page_size, keyword, status };
        Ok(self.inner.list_dictionary_types(request).await?.into_inner())
    }

    pub async fn create_dictionary_type(
        &mut self,
        code: String,
        name: String,
        description: String,
        sort: i32,
    ) -> Result<grpc_proto::user::CreateDictionaryTypeResponse, tonic::Status> {
        let request = grpc_proto::user::CreateDictionaryTypeRequest { code, name, description, sort };
        Ok(self.inner.create_dictionary_type(request).await?.into_inner())
    }

    pub async fn update_dictionary_type(
        &mut self,
        id: i64,
        name: String,
        description: String,
        sort: i32,
        status: i32,
    ) -> Result<grpc_proto::user::UpdateDictionaryTypeResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateDictionaryTypeRequest { id, name, description, sort, status };
        Ok(self.inner.update_dictionary_type(request).await?.into_inner())
    }

    pub async fn delete_dictionary_type(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::DeleteDictionaryTypeResponse, tonic::Status> {
        let request = grpc_proto::user::DeleteDictionaryTypeRequest { id };
        Ok(self.inner.delete_dictionary_type(request).await?.into_inner())
    }

    pub async fn list_dictionary_items(
        &mut self,
        type_id: i64,
        type_code: String,
        keyword: String,
        status: i32,
    ) -> Result<grpc_proto::user::ListDictionaryItemsResponse, tonic::Status> {
        let request = grpc_proto::user::ListDictionaryItemsRequest { type_id, type_code, keyword, status };
        Ok(self.inner.list_dictionary_items(request).await?.into_inner())
    }

    pub async fn create_dictionary_item(
        &mut self,
        type_id: i64,
        label: String,
        value: String,
        sort: i32,
        status: i32,
        is_default: bool,
        remark: String,
    ) -> Result<grpc_proto::user::CreateDictionaryItemResponse, tonic::Status> {
        let request = grpc_proto::user::CreateDictionaryItemRequest { type_id, label, value, sort, status, is_default, remark };
        Ok(self.inner.create_dictionary_item(request).await?.into_inner())
    }

    pub async fn update_dictionary_item(
        &mut self,
        id: i64,
        label: String,
        value: String,
        sort: i32,
        status: i32,
        is_default: bool,
        remark: String,
    ) -> Result<grpc_proto::user::UpdateDictionaryItemResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateDictionaryItemRequest { id, label, value, sort, status, is_default, remark };
        Ok(self.inner.update_dictionary_item(request).await?.into_inner())
    }

    pub async fn delete_dictionary_item(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::DeleteDictionaryItemResponse, tonic::Status> {
        let request = grpc_proto::user::DeleteDictionaryItemRequest { id };
        Ok(self.inner.delete_dictionary_item(request).await?.into_inner())
    }

    pub async fn get_dictionary_type(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::GetDictionaryTypeResponse, tonic::Status> {
        let request = grpc_proto::user::GetDictionaryTypeRequest { id };
        Ok(self.inner.get_dictionary_type(request).await?.into_inner())
    }

    pub async fn get_dictionary_item(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::GetDictionaryItemResponse, tonic::Status> {
        let request = grpc_proto::user::GetDictionaryItemRequest { id };
        Ok(self.inner.get_dictionary_item(request).await?.into_inner())
    }

    pub async fn import_users(
        &mut self,
        data_base64: String,
        format: String,
        headers: Vec<String>,
        update_mode: String,
    ) -> Result<grpc_proto::user::ImportUsersResponse, tonic::Status> {
        let request = grpc_proto::user::ImportUsersRequest { filename: String::new(), format, headers, data_base64, update_mode };
        Ok(self.inner.import_users(request).await?.into_inner())
    }

    pub async fn export_users(
        &mut self,
        format: String,
        keyword: String,
        status: i32,
        role: String,
    ) -> Result<grpc_proto::user::ExportUsersResponse, tonic::Status> {
        let request = grpc_proto::user::ExportUsersRequest { format, keyword, status, role };
        Ok(self.inner.export_users(request).await?.into_inner())
    }

    pub async fn list_announcements(
        &mut self,
        page: i32,
        page_size: i32,
        is_active: i32,
    ) -> Result<grpc_proto::user::ListAnnouncementsResponse, tonic::Status> {
        let request = grpc_proto::user::ListAnnouncementsRequest { page, page_size, is_active };
        Ok(self.inner.list_announcements(request).await?.into_inner())
    }

    pub async fn get_announcement(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::GetAnnouncementResponse, tonic::Status> {
        let request = grpc_proto::user::GetAnnouncementRequest { id };
        Ok(self.inner.get_announcement(request).await?.into_inner())
    }

    pub async fn create_announcement(
        &mut self,
        title: String,
        content: String,
        r#type: String,
        priority: i32,
        is_pinned: bool,
        is_active: bool,
        start_time: String,
        end_time: String,
        created_by: String,
    ) -> Result<grpc_proto::user::CreateAnnouncementResponse, tonic::Status> {
        let request = grpc_proto::user::CreateAnnouncementRequest { title, content, r#type, priority, is_pinned, is_active, start_time, end_time, created_by };
        Ok(self.inner.create_announcement(request).await?.into_inner())
    }

    pub async fn update_announcement(
        &mut self,
        id: i64,
        title: String,
        content: String,
        r#type: String,
        priority: i32,
        is_pinned: bool,
        is_active: bool,
        start_time: String,
        end_time: String,
    ) -> Result<grpc_proto::user::UpdateAnnouncementResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateAnnouncementRequest { id, title, content, r#type, priority, is_pinned, is_active, start_time, end_time };
        Ok(self.inner.update_announcement(request).await?.into_inner())
    }

    pub async fn delete_announcement(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::user::DeleteAnnouncementResponse, tonic::Status> {
        let request = grpc_proto::user::DeleteAnnouncementRequest { id };
        Ok(self.inner.delete_announcement(request).await?.into_inner())
    }

    pub async fn list_system_configs(
        &mut self,
        category: String,
    ) -> Result<grpc_proto::user::ListSystemConfigsResponse, tonic::Status> {
        let request = grpc_proto::user::ListSystemConfigsRequest { category };
        Ok(self.inner.list_system_configs(request).await?.into_inner())
    }

    pub async fn update_system_config(
        &mut self,
        key: String,
        value: String,
    ) -> Result<grpc_proto::user::UpdateSystemConfigResponse, tonic::Status> {
        let request = grpc_proto::user::UpdateSystemConfigRequest { key, value };
        Ok(self.inner.update_system_config(request).await?.into_inner())
    }

    pub async fn batch_update_system_configs(
        &mut self,
        configs: Vec<grpc_proto::user::UpdateSystemConfigRequest>,
    ) -> Result<grpc_proto::user::BatchUpdateSystemConfigsResponse, tonic::Status> {
        let request = grpc_proto::user::BatchUpdateSystemConfigsRequest { configs };
        Ok(self.inner.batch_update_system_configs(request).await?.into_inner())
    }
}

