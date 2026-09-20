//! User Service gRPC Handlers

use std::net::SocketAddr;
use grpc_proto::user::user_service_server::UserServiceServer;
use std::sync::Arc;

use crate::repository::{AnnouncementRepository, DepartmentRepository, RoleRepository, UserRepository};

mod announcement_handlers;
mod department_handlers;
mod role_handlers;
mod user_handlers;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub user_repository: UserRepository,
    pub role_repository: RoleRepository,
    pub department_repository: DepartmentRepository,
    pub announcement_repository: AnnouncementRepository,
}

impl AppState {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            user_repository: UserRepository::new(pool.clone()),
            role_repository: RoleRepository::new(pool.clone()),
            department_repository: DepartmentRepository::new(pool.clone()),
            announcement_repository: AnnouncementRepository::new(pool),
        }
    }
}

/// `UserService` 实现
#[derive(Clone)]
pub struct UserServiceImpl {
    pub(crate) state: Arc<AppState>,
}

impl UserServiceImpl {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for UserServiceImpl {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}").into() })?;
        let server = UserServiceServer::new(UserServiceImpl::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}

#[tonic::async_trait]
impl grpc_proto::user::user_service_server::UserService for UserServiceImpl {
    async fn get_user(&self, request: tonic::Request<grpc_proto::user::GetUserRequest>) -> Result<tonic::Response<grpc_proto::user::GetUserResponse>, tonic::Status> { user_handlers::get_user(&self, request).await }
    async fn create_user(&self, request: tonic::Request<grpc_proto::user::CreateUserRequest>) -> Result<tonic::Response<grpc_proto::user::CreateUserResponse>, tonic::Status> { user_handlers::create_user(&self, request).await }
    async fn update_user(&self, request: tonic::Request<grpc_proto::user::UpdateUserRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateUserResponse>, tonic::Status> { user_handlers::update_user(&self, request).await }
    async fn delete_user(&self, request: tonic::Request<grpc_proto::user::DeleteUserRequest>) -> Result<tonic::Response<grpc_proto::user::DeleteUserResponse>, tonic::Status> { user_handlers::delete_user(&self, request).await }
    async fn list_users(&self, request: tonic::Request<grpc_proto::user::ListUsersRequest>) -> Result<tonic::Response<grpc_proto::user::ListUsersResponse>, tonic::Status> { user_handlers::list_users(&self, request).await }
    async fn update_user_status(&self, request: tonic::Request<grpc_proto::user::UpdateUserStatusRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateUserStatusResponse>, tonic::Status> { user_handlers::update_user_status(&self, request).await }
    async fn update_user_role(&self, request: tonic::Request<grpc_proto::user::UpdateUserRoleRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateUserRoleResponse>, tonic::Status> { user_handlers::update_user_role(&self, request).await }
    async fn reset_password(&self, request: tonic::Request<grpc_proto::user::ResetPasswordRequest>) -> Result<tonic::Response<grpc_proto::user::ResetPasswordResponse>, tonic::Status> { user_handlers::reset_password(&self, request).await }
    async fn batch_update_user_role(&self, request: tonic::Request<grpc_proto::user::BatchUpdateUserRoleRequest>) -> Result<tonic::Response<grpc_proto::user::BatchUpdateUserRoleResponse>, tonic::Status> { user_handlers::batch_update_user_role(&self, request).await }
    async fn batch_update_user_status(&self, request: tonic::Request<grpc_proto::user::BatchUpdateUserStatusRequest>) -> Result<tonic::Response<grpc_proto::user::BatchUpdateUserStatusResponse>, tonic::Status> { user_handlers::batch_update_user_status(&self, request).await }
    async fn batch_delete_users(&self, request: tonic::Request<grpc_proto::user::BatchDeleteUsersRequest>) -> Result<tonic::Response<grpc_proto::user::BatchDeleteUsersResponse>, tonic::Status> { user_handlers::batch_delete_users(&self, request).await }
    async fn import_users(&self, request: tonic::Request<grpc_proto::user::ImportUsersRequest>) -> Result<tonic::Response<grpc_proto::user::ImportUsersResponse>, tonic::Status> { user_handlers::import_users(&self, request).await }
    async fn export_users(&self, request: tonic::Request<grpc_proto::user::ExportUsersRequest>) -> Result<tonic::Response<grpc_proto::user::ExportUsersResponse>, tonic::Status> { user_handlers::export_users(&self, request).await }
    async fn list_roles(&self, request: tonic::Request<grpc_proto::user::ListRolesRequest>) -> Result<tonic::Response<grpc_proto::user::ListRolesResponse>, tonic::Status> { role_handlers::list_roles(&self, request).await }
    async fn get_role(&self, request: tonic::Request<grpc_proto::user::GetRoleRequest>) -> Result<tonic::Response<grpc_proto::user::GetRoleResponse>, tonic::Status> { role_handlers::get_role(&self, request).await }
    async fn create_role(&self, request: tonic::Request<grpc_proto::user::CreateRoleRequest>) -> Result<tonic::Response<grpc_proto::user::CreateRoleResponse>, tonic::Status> { role_handlers::create_role(&self, request).await }
    async fn update_role(&self, request: tonic::Request<grpc_proto::user::UpdateRoleRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateRoleResponse>, tonic::Status> { role_handlers::update_role(&self, request).await }
    async fn delete_role(&self, request: tonic::Request<grpc_proto::user::DeleteRoleRequest>) -> Result<tonic::Response<grpc_proto::user::DeleteRoleResponse>, tonic::Status> { role_handlers::delete_role(&self, request).await }
    async fn get_role_permissions(&self, request: tonic::Request<grpc_proto::user::GetRolePermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::GetRolePermissionsResponse>, tonic::Status> { role_handlers::get_role_permissions(&self, request).await }
    async fn set_role_permissions(&self, request: tonic::Request<grpc_proto::user::SetRolePermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::SetRolePermissionsResponse>, tonic::Status> { role_handlers::set_role_permissions(&self, request).await }
    async fn get_role_users(&self, request: tonic::Request<grpc_proto::user::GetRoleUsersRequest>) -> Result<tonic::Response<grpc_proto::user::GetRoleUsersResponse>, tonic::Status> { role_handlers::get_role_users(&self, request).await }
    async fn list_departments(&self, request: tonic::Request<grpc_proto::user::ListDepartmentsRequest>) -> Result<tonic::Response<grpc_proto::user::ListDepartmentsResponse>, tonic::Status> { department_handlers::list_departments(&self, request).await }
    async fn get_department(&self, request: tonic::Request<grpc_proto::user::GetDepartmentRequest>) -> Result<tonic::Response<grpc_proto::user::GetDepartmentResponse>, tonic::Status> { department_handlers::get_department(&self, request).await }
    async fn create_department(&self, request: tonic::Request<grpc_proto::user::CreateDepartmentRequest>) -> Result<tonic::Response<grpc_proto::user::CreateDepartmentResponse>, tonic::Status> { department_handlers::create_department(&self, request).await }
    async fn update_department(&self, request: tonic::Request<grpc_proto::user::UpdateDepartmentRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateDepartmentResponse>, tonic::Status> { department_handlers::update_department(&self, request).await }
    async fn delete_department(&self, request: tonic::Request<grpc_proto::user::DeleteDepartmentRequest>) -> Result<tonic::Response<grpc_proto::user::DeleteDepartmentResponse>, tonic::Status> { department_handlers::delete_department(&self, request).await }
    async fn get_department_tree(&self, request: tonic::Request<grpc_proto::user::GetDepartmentTreeRequest>) -> Result<tonic::Response<grpc_proto::user::GetDepartmentTreeResponse>, tonic::Status> { department_handlers::get_department_tree(&self, request).await }
    async fn get_department_users(&self, request: tonic::Request<grpc_proto::user::GetDepartmentUsersRequest>) -> Result<tonic::Response<grpc_proto::user::GetDepartmentUsersResponse>, tonic::Status> { department_handlers::get_department_users(&self, request).await }
    async fn list_announcements(&self, request: tonic::Request<grpc_proto::user::ListAnnouncementsRequest>) -> Result<tonic::Response<grpc_proto::user::ListAnnouncementsResponse>, tonic::Status> { announcement_handlers::list_announcements(&self, request).await }
    async fn get_announcement(&self, request: tonic::Request<grpc_proto::user::GetAnnouncementRequest>) -> Result<tonic::Response<grpc_proto::user::GetAnnouncementResponse>, tonic::Status> { announcement_handlers::get_announcement(&self, request).await }
    async fn create_announcement(&self, request: tonic::Request<grpc_proto::user::CreateAnnouncementRequest>) -> Result<tonic::Response<grpc_proto::user::CreateAnnouncementResponse>, tonic::Status> { announcement_handlers::create_announcement(&self, request).await }
    async fn update_announcement(&self, request: tonic::Request<grpc_proto::user::UpdateAnnouncementRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateAnnouncementResponse>, tonic::Status> { announcement_handlers::update_announcement(&self, request).await }
    async fn delete_announcement(&self, request: tonic::Request<grpc_proto::user::DeleteAnnouncementRequest>) -> Result<tonic::Response<grpc_proto::user::DeleteAnnouncementResponse>, tonic::Status> { announcement_handlers::delete_announcement(&self, request).await }
    async fn list_system_configs(&self, request: tonic::Request<grpc_proto::user::ListSystemConfigsRequest>) -> Result<tonic::Response<grpc_proto::user::ListSystemConfigsResponse>, tonic::Status> { announcement_handlers::list_system_configs(&self, request).await }
    async fn update_system_config(&self, request: tonic::Request<grpc_proto::user::UpdateSystemConfigRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateSystemConfigResponse>, tonic::Status> { announcement_handlers::update_system_config(&self, request).await }
    async fn batch_update_system_configs(&self, request: tonic::Request<grpc_proto::user::BatchUpdateSystemConfigsRequest>) -> Result<tonic::Response<grpc_proto::user::BatchUpdateSystemConfigsResponse>, tonic::Status> { announcement_handlers::batch_update_system_configs(&self, request).await }
    async fn get_role_permission_config(&self, request: tonic::Request<grpc_proto::user::GetRolePermissionConfigRequest>) -> Result<tonic::Response<grpc_proto::user::GetRolePermissionConfigResponse>, tonic::Status> { announcement_handlers::get_role_permission_config(&self, request).await }
    async fn update_role_permission_config(&self, request: tonic::Request<grpc_proto::user::UpdateRolePermissionConfigRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateRolePermissionConfigResponse>, tonic::Status> { announcement_handlers::update_role_permission_config(&self, request).await }
    async fn get_role_data_permissions(&self, request: tonic::Request<grpc_proto::user::GetRoleDataPermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::GetRoleDataPermissionsResponse>, tonic::Status> { announcement_handlers::get_role_data_permissions(&self, request).await }
    async fn set_role_data_permissions(&self, request: tonic::Request<grpc_proto::user::SetRoleDataPermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::SetRoleDataPermissionsResponse>, tonic::Status> { announcement_handlers::set_role_data_permissions(&self, request).await }
    async fn get_role_field_permissions(&self, request: tonic::Request<grpc_proto::user::GetRoleFieldPermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::GetRoleFieldPermissionsResponse>, tonic::Status> { announcement_handlers::get_role_field_permissions(&self, request).await }
    async fn set_role_field_permissions(&self, request: tonic::Request<grpc_proto::user::SetRoleFieldPermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::SetRoleFieldPermissionsResponse>, tonic::Status> { announcement_handlers::set_role_field_permissions(&self, request).await }
    async fn get_role_inherit_chain(&self, request: tonic::Request<grpc_proto::user::GetRoleInheritChainRequest>) -> Result<tonic::Response<grpc_proto::user::GetRoleInheritChainResponse>, tonic::Status> { announcement_handlers::get_role_inherit_chain(&self, request).await }
    async fn set_role_inherit(&self, request: tonic::Request<grpc_proto::user::SetRoleInheritRequest>) -> Result<tonic::Response<grpc_proto::user::SetRoleInheritResponse>, tonic::Status> { announcement_handlers::set_role_inherit(&self, request).await }
    async fn remove_role_inherit(&self, request: tonic::Request<grpc_proto::user::RemoveRoleInheritRequest>) -> Result<tonic::Response<grpc_proto::user::RemoveRoleInheritResponse>, tonic::Status> { announcement_handlers::remove_role_inherit(&self, request).await }
    async fn get_accessible_departments(&self, request: tonic::Request<grpc_proto::user::GetAccessibleDepartmentsRequest>) -> Result<tonic::Response<grpc_proto::user::GetAccessibleDepartmentsResponse>, tonic::Status> { announcement_handlers::get_accessible_departments(&self, request).await }
    async fn get_accessible_tenants(&self, request: tonic::Request<grpc_proto::user::GetAccessibleTenantsRequest>) -> Result<tonic::Response<grpc_proto::user::GetAccessibleTenantsResponse>, tonic::Status> { announcement_handlers::get_accessible_tenants(&self, request).await }
    async fn list_permission_definitions(&self, request: tonic::Request<grpc_proto::user::ListPermissionDefinitionsRequest>) -> Result<tonic::Response<grpc_proto::user::ListPermissionDefinitionsResponse>, tonic::Status> { announcement_handlers::list_permission_definitions(&self, request).await }
    async fn create_permission_definition(&self, request: tonic::Request<grpc_proto::user::CreatePermissionDefinitionRequest>) -> Result<tonic::Response<grpc_proto::user::CreatePermissionDefinitionResponse>, tonic::Status> { announcement_handlers::create_permission_definition(&self, request).await }
    async fn update_permission_definition(&self, request: tonic::Request<grpc_proto::user::UpdatePermissionDefinitionRequest>) -> Result<tonic::Response<grpc_proto::user::UpdatePermissionDefinitionResponse>, tonic::Status> { announcement_handlers::update_permission_definition(&self, request).await }
    async fn delete_permission_definition(&self, request: tonic::Request<grpc_proto::user::DeletePermissionDefinitionRequest>) -> Result<tonic::Response<grpc_proto::user::DeletePermissionDefinitionResponse>, tonic::Status> { announcement_handlers::delete_permission_definition(&self, request).await }
    async fn batch_create_permission_definitions(&self, request: tonic::Request<grpc_proto::user::BatchCreatePermissionDefinitionsRequest>) -> Result<tonic::Response<grpc_proto::user::BatchCreatePermissionDefinitionsResponse>, tonic::Status> { announcement_handlers::batch_create_permission_definitions(&self, request).await }
    async fn batch_assign_permissions(&self, request: tonic::Request<grpc_proto::user::BatchAssignPermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::BatchAssignPermissionsResponse>, tonic::Status> { announcement_handlers::batch_assign_permissions(&self, request).await }
    async fn copy_role_permissions(&self, request: tonic::Request<grpc_proto::user::CopyRolePermissionsRequest>) -> Result<tonic::Response<grpc_proto::user::CopyRolePermissionsResponse>, tonic::Status> { announcement_handlers::copy_role_permissions(&self, request).await }
    async fn validate_data_permission(&self, request: tonic::Request<grpc_proto::user::ValidateDataPermissionRequest>) -> Result<tonic::Response<grpc_proto::user::ValidateDataPermissionResponse>, tonic::Status> { announcement_handlers::validate_data_permission(&self, request).await }
    async fn check_sensitive_permission(&self, request: tonic::Request<grpc_proto::user::CheckSensitivePermissionRequest>) -> Result<tonic::Response<grpc_proto::user::CheckSensitivePermissionResponse>, tonic::Status> { announcement_handlers::check_sensitive_permission(&self, request).await }
    async fn list_dictionary_types(&self, request: tonic::Request<grpc_proto::user::ListDictionaryTypesRequest>) -> Result<tonic::Response<grpc_proto::user::ListDictionaryTypesResponse>, tonic::Status> { announcement_handlers::list_dictionary_types(&self, request).await }
    async fn create_dictionary_type(&self, request: tonic::Request<grpc_proto::user::CreateDictionaryTypeRequest>) -> Result<tonic::Response<grpc_proto::user::CreateDictionaryTypeResponse>, tonic::Status> { announcement_handlers::create_dictionary_type(&self, request).await }
    async fn update_dictionary_type(&self, request: tonic::Request<grpc_proto::user::UpdateDictionaryTypeRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateDictionaryTypeResponse>, tonic::Status> { announcement_handlers::update_dictionary_type(&self, request).await }
    async fn delete_dictionary_type(&self, request: tonic::Request<grpc_proto::user::DeleteDictionaryTypeRequest>) -> Result<tonic::Response<grpc_proto::user::DeleteDictionaryTypeResponse>, tonic::Status> { announcement_handlers::delete_dictionary_type(&self, request).await }
    async fn list_dictionary_items(&self, request: tonic::Request<grpc_proto::user::ListDictionaryItemsRequest>) -> Result<tonic::Response<grpc_proto::user::ListDictionaryItemsResponse>, tonic::Status> { announcement_handlers::list_dictionary_items(&self, request).await }
    async fn create_dictionary_item(&self, request: tonic::Request<grpc_proto::user::CreateDictionaryItemRequest>) -> Result<tonic::Response<grpc_proto::user::CreateDictionaryItemResponse>, tonic::Status> { announcement_handlers::create_dictionary_item(&self, request).await }
    async fn update_dictionary_item(&self, request: tonic::Request<grpc_proto::user::UpdateDictionaryItemRequest>) -> Result<tonic::Response<grpc_proto::user::UpdateDictionaryItemResponse>, tonic::Status> { announcement_handlers::update_dictionary_item(&self, request).await }
    async fn delete_dictionary_item(&self, request: tonic::Request<grpc_proto::user::DeleteDictionaryItemRequest>) -> Result<tonic::Response<grpc_proto::user::DeleteDictionaryItemResponse>, tonic::Status> { announcement_handlers::delete_dictionary_item(&self, request).await }
    async fn get_dictionary_type(&self, request: tonic::Request<grpc_proto::user::GetDictionaryTypeRequest>) -> Result<tonic::Response<grpc_proto::user::GetDictionaryTypeResponse>, tonic::Status> { announcement_handlers::get_dictionary_type(&self, request).await }
    async fn get_dictionary_item(&self, request: tonic::Request<grpc_proto::user::GetDictionaryItemRequest>) -> Result<tonic::Response<grpc_proto::user::GetDictionaryItemResponse>, tonic::Status> { announcement_handlers::get_dictionary_item(&self, request).await }
}
