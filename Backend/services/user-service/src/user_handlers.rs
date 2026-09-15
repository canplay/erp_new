//! User Service gRPC Handlers - User Management

use std::net::SocketAddr;
use chrono::Utc;
use grpc_proto::user::*;
use grpc_proto::user::user_service_server::UserService;
use grpc_proto::user::user_service_server::UserServiceServer;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::repository::{
    AnnouncementRepository, DepartmentRepository, RoleRepository, UserRepository,
};

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

#[derive(Clone)]
#[allow(clippy::too_many_arguments)]
pub struct UserServiceImpl {
    state: Arc<AppState>,
}

impl UserServiceImpl {
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        let user = self.state.user_repository.find_by_id(req.user_id).await.map_err(|e| Status::internal(e.to_string()))?;
        match user {
            Some(u) => Ok(Response::new(GetUserResponse {
                id: u.id, username: u.username, nickname: u.nickname.unwrap_or_default(), avatar: u.avatar.unwrap_or_default(), phone: u.phone.unwrap_or_default(), email: u.email.unwrap_or_default(), gender: u.gender.unwrap_or(0), address: u.address.unwrap_or_default(), role: u.role, status: u.status, created_at: u.created_at.timestamp(), updated_at: u.updated_at.timestamp(),
            })),
            None => Err(Status::not_found("User not found")),
        }
    }

    async fn create_user(&self, request: Request<CreateUserRequest>) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        if req.username.is_empty() { return Err(Status::invalid_argument("用户名不能为空")); }
        if req.username.len() > 50 { return Err(Status::invalid_argument("用户名长度不能超过50个字符")); }
        if req.password.len() < 8 { return Err(Status::invalid_argument("密码长度至少8位")); }
        let username = req.username.clone();
        let nickname = if req.nickname.is_empty() { None } else { Some(req.nickname.clone()) };
        let email = if req.email.is_empty() { None } else { Some(req.email) };
        let phone = if req.phone.is_empty() { None } else { Some(req.phone) };
        let gender = if req.gender == 0 { None } else { Some(req.gender) };
        let user_id = self.state.user_repository.create(&req.username, &req.password, email, nickname, phone, gender).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreateUserResponse { id: user_id, username, nickname: if req.nickname.is_empty() { String::new() } else { req.nickname } }))
    }

    async fn update_user(&self, request: Request<UpdateUserRequest>) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();
        let user = self.state.user_repository.update(req.user_id, Some(req.nickname).filter(|s| !s.is_empty()), Some(req.gender).filter(|&g| g != 0), Some(req.address).filter(|s| !s.is_empty()), Some(req.avatar).filter(|s| !s.is_empty())).await.map_err(|e| Status::internal(e.to_string()))?;
        match user { Some(u) => Ok(Response::new(UpdateUserResponse { id: u.id, username: u.username, nickname: u.nickname.unwrap_or_default() })), None => Err(Status::not_found("User not found")) }
    }

    async fn delete_user(&self, request: Request<DeleteUserRequest>) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        let success = self.state.user_repository.delete(req.user_id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteUserResponse { success }))
    }

    async fn list_users(&self, request: Request<ListUsersRequest>) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner(); let page = req.page.max(1); let page_size = req.page_size.clamp(1, 100);
        let result = self.state.user_repository.list(page, page_size).await.map_err(|e| Status::internal(e.to_string()))?;
        let users: Vec<GetUserResponse> = result.users.into_iter().map(|u| GetUserResponse {
            id: u.id, username: u.username, nickname: u.nickname.unwrap_or_default(), avatar: u.avatar.unwrap_or_default(), phone: u.phone.unwrap_or_default(), email: u.email.unwrap_or_default(), gender: u.gender.unwrap_or(0), address: u.address.unwrap_or_default(), role: u.role, status: u.status, created_at: u.created_at.timestamp(), updated_at: u.updated_at.timestamp(),
        }).collect();
        Ok(Response::new(ListUsersResponse { users, total: result.total }))
    }

    async fn update_user_status(&self, request: Request<UpdateUserStatusRequest>) -> Result<Response<UpdateUserStatusResponse>, Status> {
        let req = request.into_inner();
        let found = self.state.user_repository.update_status(req.user_id, req.status).await.map_err(|e| Status::internal(e.to_string()))?;
        if !found { return Err(Status::not_found("用户不存在")); }
        Ok(Response::new(UpdateUserStatusResponse { success: true }))
    }

    async fn update_user_role(&self, request: Request<UpdateUserRoleRequest>) -> Result<Response<UpdateUserRoleResponse>, Status> {
        let req = request.into_inner(); if req.role.is_empty() { return Err(Status::invalid_argument("角色不能为空")); }
        let found = self.state.user_repository.update_role(req.user_id, &req.role).await.map_err(|e| Status::internal(e.to_string()))?;
        if !found { return Err(Status::not_found("用户不存在")); }
        Ok(Response::new(UpdateUserRoleResponse { success: true }))
    }

    async fn reset_password(&self, request: Request<ResetPasswordRequest>) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();
        use argon2::password_hash::{SaltString, rand_core::OsRng};
        use argon2::PasswordHasher;
        let pw = if req.new_password.is_empty() { "Password@123" } else { &req.new_password };
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = argon2::Argon2::default();
        let hash = argon2.hash_password(pw.as_bytes(), &salt).map_err(|e| Status::internal(format!("密码哈希失败: {e}")))?.to_string();
        let found = self.state.user_repository.update_password(req.user_id, &hash).await.map_err(|e| Status::internal(e.to_string()))?;
        if !found { return Err(Status::not_found("用户不存在")); }
        Ok(Response::new(ResetPasswordResponse { success: true }))
    }

    async fn batch_update_user_role(&self, request: Request<BatchUpdateUserRoleRequest>) -> Result<Response<BatchUpdateUserRoleResponse>, Status> {
        let req = request.into_inner();
        if req.user_ids.is_empty() { return Ok(Response::new(BatchUpdateUserRoleResponse { success: true, affected: 0 })); }
        let result = self.state.user_repository.batch_update_role(&req.user_ids, &req.role).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchUpdateUserRoleResponse { success: result.fail_count == 0, affected: result.success_count as i32 }))
    }

    async fn batch_update_user_status(&self, request: Request<BatchUpdateUserStatusRequest>) -> Result<Response<BatchUpdateUserStatusResponse>, Status> {
        let req = request.into_inner();
        if req.user_ids.is_empty() { return Ok(Response::new(BatchUpdateUserStatusResponse { success: true, affected: 0 })); }
        let result = self.state.user_repository.batch_update_status(&req.user_ids, req.status).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchUpdateUserStatusResponse { success: result.fail_count == 0, affected: result.success_count as i32 }))
    }

    async fn batch_delete_users(&self, request: Request<BatchDeleteUsersRequest>) -> Result<Response<BatchDeleteUsersResponse>, Status> {
        let req = request.into_inner();
        if req.user_ids.is_empty() { return Ok(Response::new(BatchDeleteUsersResponse { success: true, deleted: 0 })); }
        let result = self.state.user_repository.batch_delete(&req.user_ids).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchDeleteUsersResponse { success: result.fail_count == 0, deleted: result.success_count as i32 }))
    }

    async fn import_users(&self, request: Request<ImportUsersRequest>) -> Result<Response<ImportUsersResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.user_repository.batch_create_from_csv(&req.data_base64, &req.format, &req.update_mode).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ImportUsersResponse { total: result.total as i32, success_count: result.success_count as i32, fail_count: result.fail_count as i32, errors: result.errors }))
    }

    async fn export_users(&self, request: Request<ExportUsersRequest>) -> Result<Response<ExportUsersResponse>, Status> {
        let req = request.into_inner();
        let data = self.state.user_repository.export_to_csv(&req.keyword, req.status, &req.role, &req.format).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ExportUsersResponse { data: data.0, filename: data.1, format: req.format }))
    }

    async fn list_roles(&self, request: Request<ListRolesRequest>) -> Result<Response<ListRolesResponse>, Status> {
        let req = request.into_inner(); let page = req.page.max(1); let page_size = req.page_size.clamp(1, 100);
        let keyword = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
        let result = self.state.role_repository.list(page, page_size, keyword).await.map_err(|e| Status::internal(e.to_string()))?;
        let roles: Vec<RoleInfo> = result.roles.into_iter().map(|r| RoleInfo { id: r.id, name: r.name, description: r.description.unwrap_or_default(), r#type: r.role_type, status: r.status, created_at: r.created_at.timestamp(), updated_at: 0, user_count: r.user_count }).collect();
        Ok(Response::new(ListRolesResponse { roles, total: result.total }))
    }

    async fn get_role(&self, request: Request<GetRoleRequest>) -> Result<Response<GetRoleResponse>, Status> {
        let req = request.into_inner();
        let role = self.state.role_repository.find_by_code(&req.name).await.map_err(|e| Status::internal(e.to_string()))?;
        match role { Some(r) => Ok(Response::new(GetRoleResponse { role: Some(RoleInfo { id: r.id, name: r.name, description: r.description.unwrap_or_default(), r#type: r.role_type, status: r.status, created_at: r.created_at.timestamp(), updated_at: r.updated_at.timestamp(), user_count: 0 }) })), None => Err(Status::not_found("角色不存在")) }
    }

    async fn create_role(&self, request: Request<CreateRoleRequest>) -> Result<Response<CreateRoleResponse>, Status> {
        let req = request.into_inner(); if req.name.is_empty() { return Err(Status::invalid_argument("角色名称不能为空")); }
        let code = req.name.to_lowercase().replace(' ', "-");
        let id = self.state.role_repository.create(&req.name, &code, Some(req.description.clone()), &req.r#type, None).await.map_err(|e| match e { crate::repository::RoleRepositoryError::AlreadyExists => Status::already_exists("角色已存在"), o => Status::internal(format!("{:?}", o)) })?;
        Ok(Response::new(CreateRoleResponse { role: Some(RoleInfo { id, name: req.name, description: req.description, r#type: req.r#type, status: 1, created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp(), user_count: 0 }) }))
    }

    async fn update_role(&self, request: Request<UpdateRoleRequest>) -> Result<Response<UpdateRoleResponse>, Status> {
        let req = request.into_inner();
        match self.state.role_repository.find_by_code(&req.name).await.map_err(|e| Status::internal(e.to_string()))? {
            Some(_) => { self.state.role_repository.update(&req.name, Some(req.description).filter(|s| !s.is_empty()), None, Some(req.status)).await.map_err(|e| Status::internal(e.to_string()))?; Ok(Response::new(UpdateRoleResponse { success: true })) }
            None => Err(Status::not_found("角色不存在")),
        }
    }

    async fn delete_role(&self, request: Request<DeleteRoleRequest>) -> Result<Response<DeleteRoleResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.role_repository.delete(&req.name).await.map_err(|e| match e { crate::repository::RoleRepositoryError::HasAssociatedUsers => Status::failed_precondition("角色有用户关联"), crate::repository::RoleRepositoryError::HasChildRoles => Status::failed_precondition("角色有子角色"), o => Status::internal(format!("{:?}", o)) })?;
        Ok(Response::new(DeleteRoleResponse { success: result }))
    }

    async fn get_role_permissions(&self, request: Request<GetRolePermissionsRequest>) -> Result<Response<GetRolePermissionsResponse>, Status> {
        let req = request.into_inner();
        let role = self.state.role_repository.find_by_code(&req.name).await.map_err(|e| Status::internal(e.to_string()))?.ok_or_else(|| Status::not_found("角色不存在"))?;
        let perms = self.state.role_repository.get_permissions(role.id).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetRolePermissionsResponse { permissions: perms.into_iter().map(|p| p.code).collect() }))
    }

    async fn set_role_permissions(&self, request: Request<SetRolePermissionsRequest>) -> Result<Response<SetRolePermissionsResponse>, Status> {
        let req = request.into_inner();
        let role = self.state.role_repository.find_by_code(&req.name).await.map_err(|e| Status::internal(e.to_string()))?.ok_or_else(|| Status::not_found("角色不存在"))?;
        let ids = self.state.role_repository.resolve_permission_ids(&req.permissions).await.map_err(|e| Status::internal(e.to_string()))?;
        self.state.role_repository.set_permissions(role.id, &ids).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SetRolePermissionsResponse { success: true }))
    }

    async fn get_role_users(&self, request: Request<GetRoleUsersRequest>) -> Result<Response<GetRoleUsersResponse>, Status> {
        let req = request.into_inner(); let page = req.page.max(1); let page_size = req.page_size.clamp(1, 100);
        let role = self.state.role_repository.find_by_code(&req.name).await.map_err(|e| Status::internal(e.to_string()))?.ok_or_else(|| Status::not_found("角色不存在"))?;
        let (ids, total) = self.state.role_repository.get_users(role.id, page, page_size).await.map_err(|e| Status::internal(e.to_string()))?;
        let mut users = Vec::new();
        for uid in ids { if let Ok(Some(u)) = self.state.user_repository.find_by_id(uid).await { users.push(GetUserResponse { id: u.id, username: u.username, nickname: u.nickname.unwrap_or_default(), avatar: u.avatar.unwrap_or_default(), phone: u.phone.unwrap_or_default(), email: u.email.unwrap_or_default(), gender: u.gender.unwrap_or(0), address: u.address.unwrap_or_default(), role: u.role, status: u.status, created_at: u.created_at.timestamp(), updated_at: u.updated_at.timestamp() }); } }
        Ok(Response::new(GetRoleUsersResponse { users, total }))
    }

    async fn list_departments(&self, request: Request<ListDepartmentsRequest>) -> Result<Response<ListDepartmentsResponse>, Status> {
        let req = request.into_inner(); let page = req.page.max(1); let page_size = req.page_size.clamp(1, 100);
        let kw = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
        let result = self.state.department_repository.list(page, page_size, kw).await.map_err(|e| Status::internal(e.to_string()))?;
        let depts: Vec<DepartmentInfo> = result.departments.into_iter().map(|d| DepartmentInfo { id: d.id, name: d.name, parent_id: d.parent_id.unwrap_or(0), description: String::new(), sort_order: d.sort_order, status: d.status, created_at: d.created_at.timestamp(), updated_at: 0 }).collect();
        Ok(Response::new(ListDepartmentsResponse { departments: depts, total: result.total }))
    }

    async fn get_department(&self, request: Request<GetDepartmentRequest>) -> Result<Response<GetDepartmentResponse>, Status> {
        let req = request.into_inner();
        let dept = self.state.department_repository.find_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))?;
        match dept { Some(d) => Ok(Response::new(GetDepartmentResponse { department: Some(DepartmentInfo { id: d.id, name: d.name, parent_id: d.parent_id.unwrap_or(0), description: d.description.unwrap_or_default(), sort_order: d.sort_order, status: d.status, created_at: d.created_at.timestamp(), updated_at: d.updated_at.timestamp() }) })), None => Err(Status::not_found("部门不存在")) }
    }

    async fn create_department(&self, request: Request<CreateDepartmentRequest>) -> Result<Response<CreateDepartmentResponse>, Status> {
        let req = request.into_inner(); if req.name.is_empty() { return Err(Status::invalid_argument("部门名称不能为空")); }
        let parent_id = if req.parent_id == 0 { None } else { Some(req.parent_id) };
        let desc = if req.description.is_empty() { None } else { Some(req.description.as_str()) };
        let sort = if req.sort_order == 0 { None } else { Some(req.sort_order) };
        let id = self.state.department_repository.create(&req.name, None, parent_id, None, desc, sort).await.map_err(|e| match e { crate::repository::DepartmentRepositoryError::AlreadyExists => Status::already_exists("部门代码已存在"), o => Status::internal(format!("{:?}", o)) })?;
        Ok(Response::new(CreateDepartmentResponse { department: Some(DepartmentInfo { id, name: req.name, parent_id: req.parent_id, description: req.description, sort_order: sort.unwrap_or(0), status: 1, created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp() }) }))
    }

    async fn update_department(&self, request: Request<UpdateDepartmentRequest>) -> Result<Response<UpdateDepartmentResponse>, Status> {
        let req = request.into_inner();
        if self.state.department_repository.find_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))?.is_none() { return Err(Status::not_found("部门不存在")); }
        self.state.department_repository.update(req.id, Some(req.name).filter(|s| !s.is_empty()), None, Some(req.parent_id).filter(|&id| id != 0), None, Some(req.description).filter(|s| !s.is_empty()), Some(req.sort_order).filter(|&s| s != 0), Some(req.status)).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateDepartmentResponse { success: true }))
    }

    async fn delete_department(&self, request: Request<DeleteDepartmentRequest>) -> Result<Response<DeleteDepartmentResponse>, Status> {
        let req = request.into_inner();
        let result = self.state.department_repository.delete(req.id).await.map_err(|e| match e { crate::repository::DepartmentRepositoryError::HasChildDepartments => Status::failed_precondition("部门有子部门"), crate::repository::DepartmentRepositoryError::HasAssociatedUsers => Status::failed_precondition("部门有用户关联"), o => Status::internal(format!("{:?}", o)) })?;
        Ok(Response::new(DeleteDepartmentResponse { success: result }))
    }

    async fn get_department_tree(&self, _request: Request<GetDepartmentTreeRequest>) -> Result<Response<GetDepartmentTreeResponse>, Status> {
        let tree_nodes = self.state.department_repository.get_tree().await.map_err(|e| Status::internal(e.to_string()))?;
        fn flatten(nodes: Vec<crate::repository::DepartmentTreeNode>) -> Vec<DepartmentInfo> {
            let mut r = Vec::new();
            for n in nodes { r.push(DepartmentInfo { id: n.id, name: n.name, parent_id: n.parent_id.unwrap_or(0), description: String::new(), sort_order: n.sort_order, status: 1, created_at: 0, updated_at: 0 }); r.extend(flatten(n.children)); }
            r
        }
        Ok(Response::new(GetDepartmentTreeResponse { tree: flatten(tree_nodes) }))
    }

    async fn get_department_users(&self, request: Request<GetDepartmentUsersRequest>) -> Result<Response<GetDepartmentUsersResponse>, Status> {
        let req = request.into_inner(); let page = req.page.max(1); let page_size = req.page_size.clamp(1, 100);
        let (ids, total) = self.state.department_repository.get_users(req.department_id, page, page_size).await.map_err(|e| Status::internal(e.to_string()))?;
        let mut users = Vec::new();
        for uid in ids { if let Ok(Some(u)) = self.state.user_repository.find_by_id(uid).await { users.push(GetUserResponse { id: u.id, username: u.username, nickname: u.nickname.unwrap_or_default(), avatar: u.avatar.unwrap_or_default(), phone: u.phone.unwrap_or_default(), email: u.email.unwrap_or_default(), gender: u.gender.unwrap_or(0), address: u.address.unwrap_or_default(), role: u.role, status: u.status, created_at: u.created_at.timestamp(), updated_at: u.updated_at.timestamp() }); } }
        Ok(Response::new(GetDepartmentUsersResponse { users, total }))
    }

    async fn list_dictionary_types(&self, request: Request<ListDictionaryTypesRequest>) -> Result<Response<ListDictionaryTypesResponse>, Status> {
        let req = request.into_inner(); let page = req.page.max(1); let page_size = req.page_size.clamp(1, 100);
        let kw = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
        let status = if req.status == 0 { None } else { Some(req.status) };
        let result = self.state.announcement_repository.list_dictionary_types(page, page_size, kw, status).await.map_err(|e| Status::internal(e.to_string()))?;
        let types: Vec<DictionaryTypeInfo> = result.types.into_iter().map(|t| DictionaryTypeInfo { id: t.id, code: t.code, name: t.name, description: t.description.unwrap_or_default(), sort: t.sort, status: t.status, created_at: t.created_at.timestamp(), updated_at: t.updated_at.timestamp() }).collect();
        Ok(Response::new(ListDictionaryTypesResponse { types, total: result.total }))
    }

    async fn create_dictionary_type(&self, request: Request<CreateDictionaryTypeRequest>) -> Result<Response<CreateDictionaryTypeResponse>, Status> {
        let req = request.into_inner(); if req.code.is_empty() || req.name.is_empty() { return Err(Status::invalid_argument("字典类型编码和名称不能为空")); }
        let desc = if req.description.is_empty() { None } else { Some(req.description.as_str()) };
        let sort = Some(req.sort);
        let id = self.state.announcement_repository.create_dictionary_type(&req.code, &req.name, desc, sort).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreateDictionaryTypeResponse { r#type: Some(DictionaryTypeInfo { id, code: req.code, name: req.name, description: req.description, sort: sort.unwrap_or(0), status: 1, created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp() }) }))
    }

    async fn update_dictionary_type(&self, request: Request<UpdateDictionaryTypeRequest>) -> Result<Response<UpdateDictionaryTypeResponse>, Status> {
        let req = request.into_inner();
        if self.state.announcement_repository.find_dictionary_type_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))?.is_none() { return Err(Status::not_found("字典类型不存在")); }
        self.state.announcement_repository.update_dictionary_type(req.id, Some(req.name).filter(|s| !s.is_empty()), Some(req.description).filter(|s| !s.is_empty()), Some(req.sort).filter(|&s| s != 0), Some(req.status)).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateDictionaryTypeResponse { success: true }))
    }

    async fn delete_dictionary_type(&self, request: Request<DeleteDictionaryTypeRequest>) -> Result<Response<DeleteDictionaryTypeResponse>, Status> {
        let req = request.into_inner();
        if !self.state.announcement_repository.delete_dictionary_type(req.id).await.map_err(|e| Status::internal(e.to_string()))? { return Err(Status::not_found("字典类型不存在")); }
        Ok(Response::new(DeleteDictionaryTypeResponse { success: true }))
    }

    async fn list_dictionary_items(&self, request: Request<ListDictionaryItemsRequest>) -> Result<Response<ListDictionaryItemsResponse>, Status> {
        let req = request.into_inner();
        let type_id = if req.type_id == 0 { None } else { Some(req.type_id) };
        let type_code = if req.type_code.is_empty() { None } else { Some(req.type_code.as_str()) };
        let kw = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
        let status = if req.status == 0 { None } else { Some(req.status) };
        let result = self.state.announcement_repository.list_dictionary_items(type_id, type_code, kw, status).await.map_err(|e| Status::internal(e.to_string()))?;
        let items: Vec<DictionaryItemInfo> = result.items.into_iter().map(|i| DictionaryItemInfo { id: i.id, type_id: i.type_id, label: i.label, value: i.value, sort: i.sort, status: i.status, is_default: i.is_default, remark: i.remark.unwrap_or_default(), created_at: i.created_at.timestamp(), updated_at: i.updated_at.timestamp() }).collect();
        Ok(Response::new(ListDictionaryItemsResponse { items, total: result.total }))
    }

    async fn create_dictionary_item(&self, request: Request<CreateDictionaryItemRequest>) -> Result<Response<CreateDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        let sort = Some(req.sort); let status = Some(req.status); let is_default = Some(req.is_default);
        let remark = if req.remark.is_empty() { None } else { Some(req.remark.as_str()) };
        let params = crate::repository::CreateDictionaryItemParams { type_id: req.type_id, label: &req.label, value: &req.value, sort, status, is_default, remark };
        let id = self.state.announcement_repository.create_dictionary_item(params).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreateDictionaryItemResponse { item: Some(DictionaryItemInfo { id, type_id: req.type_id, label: req.label, value: req.value, sort: sort.unwrap_or(0), status: status.unwrap_or(1), is_default: is_default.unwrap_or(false), remark: remark.unwrap_or("").to_string(), created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp() }) }))
    }

    async fn update_dictionary_item(&self, request: Request<UpdateDictionaryItemRequest>) -> Result<Response<UpdateDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        if self.state.announcement_repository.find_dictionary_item_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))?.is_none() { return Err(Status::not_found("字典项不存在")); }
        let label = if req.label.is_empty() { None } else { Some(req.label) };
        let value = if req.value.is_empty() { None } else { Some(req.value) };
        let remark = if req.remark.is_empty() { None } else { Some(req.remark) };
        let sort_val = if req.sort == 0 { None } else { Some(req.sort) };
        let status_val = if req.status == 0 { None } else { Some(req.status) };
        let is_default_val = if req.is_default { Some(true) } else { None };
        let update_params = crate::repository::UpdateDictionaryItemParams { id: req.id, label: label.as_deref().unwrap_or(""), value: value.as_deref().unwrap_or(""), sort: sort_val, status: status_val, is_default: is_default_val, remark: remark.as_deref() };
        self.state.announcement_repository.update_dictionary_item(update_params).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateDictionaryItemResponse { success: true }))
    }

    async fn delete_dictionary_item(&self, request: Request<DeleteDictionaryItemRequest>) -> Result<Response<DeleteDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        if !self.state.announcement_repository.delete_dictionary_item(req.id).await.map_err(|e| Status::internal(e.to_string()))? { return Err(Status::not_found("字典项不存在")); }
        Ok(Response::new(DeleteDictionaryItemResponse { success: true }))
    }

    async fn get_dictionary_type(&self, request: Request<GetDictionaryTypeRequest>) -> Result<Response<GetDictionaryTypeResponse>, Status> {
        let req = request.into_inner();
        match self.state.announcement_repository.find_dictionary_type_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))? {
            Some(t) => Ok(Response::new(GetDictionaryTypeResponse { r#type: Some(DictionaryTypeInfo { id: t.id, code: t.code, name: t.name, description: t.description.unwrap_or_default(), sort: t.sort, status: t.status, created_at: t.created_at.timestamp(), updated_at: t.updated_at.timestamp() }) })),
            None => Err(Status::not_found("字典类型不存在")),
        }
    }

    async fn get_dictionary_item(&self, request: Request<GetDictionaryItemRequest>) -> Result<Response<GetDictionaryItemResponse>, Status> {
        let req = request.into_inner();
        match self.state.announcement_repository.find_dictionary_item_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))? {
            Some(i) => Ok(Response::new(GetDictionaryItemResponse { item: Some(DictionaryItemInfo { id: i.id, type_id: i.type_id, label: i.label, value: i.value, sort: i.sort, status: i.status, is_default: i.is_default, remark: i.remark.unwrap_or_default(), created_at: i.created_at.timestamp(), updated_at: i.updated_at.timestamp() }) })),
            None => Err(Status::not_found("字典项不存在")),
        }
    }

    async fn list_announcements(&self, request: Request<ListAnnouncementsRequest>) -> Result<Response<ListAnnouncementsResponse>, Status> {
        let req = request.into_inner();
        let is_active = if req.is_active == 0 { None } else { Some(req.is_active == 1) };
        let result = self.state.announcement_repository.list(req.page.max(1), req.page_size.clamp(1, 100), is_active).await.map_err(|e| Status::internal(e.to_string()))?;
        let announcements: Vec<AnnouncementInfo> = result.announcements.into_iter().map(|a| AnnouncementInfo { id: a.id, title: a.title, content: String::new(), r#type: a.announcement_type, priority: a.priority, is_pinned: a.is_pinned, is_active: a.is_active, created_by: a.created_by_name.unwrap_or_default(), start_time: a.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(), end_time: a.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(), created_at: a.created_at.timestamp(), updated_at: 0 }).collect();
        Ok(Response::new(ListAnnouncementsResponse { announcements, total: result.total }))
    }

    async fn get_announcement(&self, request: Request<GetAnnouncementRequest>) -> Result<Response<GetAnnouncementResponse>, Status> {
        let req = request.into_inner();
        match self.state.announcement_repository.find_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))? {
            Some(a) => Ok(Response::new(GetAnnouncementResponse { announcement: Some(AnnouncementInfo { id: a.id, title: a.title, content: a.content, r#type: a.announcement_type, priority: a.priority, is_pinned: a.is_pinned, is_active: a.is_active, created_by: a.created_by_name.unwrap_or_default(), start_time: a.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(), end_time: a.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(), created_at: a.created_at.timestamp(), updated_at: a.updated_at.timestamp() }) })),
            None => Err(Status::not_found("公告不存在")),
        }
    }

    async fn create_announcement(&self, request: Request<CreateAnnouncementRequest>) -> Result<Response<CreateAnnouncementResponse>, Status> {
        let req = request.into_inner();
        let created_by: Option<i64> = req.created_by.parse().ok();
        let id = self.state.announcement_repository.create(&req.title, &req.content, &req.r#type, req.priority, req.is_pinned, req.is_active, None, None, created_by).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreateAnnouncementResponse { announcement: Some(AnnouncementInfo { id, title: req.title, content: req.content, r#type: req.r#type, priority: req.priority, is_pinned: req.is_pinned, is_active: req.is_active, created_by: req.created_by, start_time: String::new(), end_time: String::new(), created_at: Utc::now().timestamp(), updated_at: Utc::now().timestamp() }) }))
    }

    async fn update_announcement(&self, request: Request<UpdateAnnouncementRequest>) -> Result<Response<UpdateAnnouncementResponse>, Status> {
        let req = request.into_inner();
        if self.state.announcement_repository.find_by_id(req.id).await.map_err(|e| Status::internal(e.to_string()))?.is_none() { return Err(Status::not_found("公告不存在")); }
        let title = if req.title.is_empty() { None } else { Some(req.title) };
        let content = if req.content.is_empty() { None } else { Some(req.content) };
        let ann_type = if req.r#type.is_empty() { None } else { Some(req.r#type) };
        let priority_val = if req.priority == 0 { None } else { Some(req.priority) };
        self.state.announcement_repository.update(req.id, title, content, ann_type, priority_val, Some(req.is_pinned), Some(req.is_active), None, None).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateAnnouncementResponse { success: true }))
    }

    async fn delete_announcement(&self, request: Request<DeleteAnnouncementRequest>) -> Result<Response<DeleteAnnouncementResponse>, Status> {
        let req = request.into_inner();
        if !self.state.announcement_repository.delete(req.id).await.map_err(|e| Status::internal(e.to_string()))? { return Err(Status::not_found("公告不存在")); }
        Ok(Response::new(DeleteAnnouncementResponse { success: true }))
    }

    async fn list_system_configs(&self, request: Request<ListSystemConfigsRequest>) -> Result<Response<ListSystemConfigsResponse>, Status> {
        let req = request.into_inner();
        let category = if req.category.is_empty() { None } else { Some(req.category.as_str()) };
        let configs = self.state.announcement_repository.list_system_configs(category).await.map_err(|e| Status::internal(e.to_string()))?;
        let config_infos: Vec<SystemConfigInfo> = configs.into_iter().map(|c| SystemConfigInfo { id: c.id, category: c.category, key: c.config_key, value: c.config_value.unwrap_or_default(), r#type: c.value_type, label: c.label, description: c.description.unwrap_or_default(), sort: c.sort_order, status: c.status, created_at: c.created_at.timestamp(), updated_at: c.updated_at.timestamp() }).collect();
        Ok(Response::new(ListSystemConfigsResponse { configs: config_infos }))
    }

    async fn update_system_config(&self, request: Request<UpdateSystemConfigRequest>) -> Result<Response<UpdateSystemConfigResponse>, Status> {
        let req = request.into_inner();
        if !self.state.announcement_repository.update_config(&req.key, &req.value).await.map_err(|e| Status::internal(e.to_string()))? { return Err(Status::not_found("配置不存在")); }
        Ok(Response::new(UpdateSystemConfigResponse { success: true }))
    }

    async fn batch_update_system_configs(&self, request: Request<BatchUpdateSystemConfigsRequest>) -> Result<Response<BatchUpdateSystemConfigsResponse>, Status> {
        let req = request.into_inner();
        let pairs: Vec<(String, String)> = req.configs.into_iter().map(|c| (c.key, c.value)).collect();
        self.state.announcement_repository.batch_update_system_configs(&pairs).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchUpdateSystemConfigsResponse { success: true }))
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for UserServiceImpl {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;
        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}").into() })?;
        let server = UserServiceServer::new(UserServiceImpl::new(self.state.clone()));
        let handle = tokio::spawn(async move { if let Err(e) = Server::builder().add_service(server).serve(addr).await { tracing::error!("gRPC server error: {}", e); } });
        Ok(handle)
    }
}
