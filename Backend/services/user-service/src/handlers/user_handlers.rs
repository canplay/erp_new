use grpc_proto::user::*;
use tonic::{Request, Response, Status};
use super::UserServiceImpl;

pub(crate) async fn get_user(s: &UserServiceImpl, request: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
    let req = request.into_inner();
    let user = s.state.user_repository.find_by_id(req.user_id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    match user {
        Some(user) => Ok(Response::new(GetUserResponse {
            id: user.id, username: user.username,
            nickname: user.nickname.unwrap_or_default(),
            avatar: user.avatar.unwrap_or_default(),
            phone: user.phone.unwrap_or_default(),
            email: user.email.unwrap_or_default(),
            gender: user.gender.unwrap_or(0),
            address: user.address.unwrap_or_default(),
            role: user.role, status: user.status,
            created_at: user.created_at.timestamp(),
            updated_at: user.updated_at.timestamp(),
        })),
        None => Err(Status::not_found("User not found")),
    }
}

pub(crate) async fn create_user(s: &UserServiceImpl, request: Request<CreateUserRequest>) -> Result<Response<CreateUserResponse>, Status> {
    let req = request.into_inner();
    if req.username.is_empty() { return Err(Status::invalid_argument("用户名不能为空")); }
    if req.username.len() > 50 { return Err(Status::invalid_argument("用户名长度不能超过50个字符")); }
    if req.password.len() < 8 { return Err(Status::invalid_argument("密码长度至少8位")); }

    let username = req.username.clone();
    let nickname = if req.nickname.is_empty() { None } else { Some(req.nickname.clone()) };
    let email = if req.email.is_empty() { None } else { Some(req.email) };
    let phone = if req.phone.is_empty() { None } else { Some(req.phone) };
    let gender = if req.gender == 0 { None } else { Some(req.gender) };

    let user_id = s.state.user_repository.create(&req.username, &req.password, email, nickname, phone, gender).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(CreateUserResponse {
        id: user_id, username,
        nickname: if req.nickname.is_empty() { String::new() } else { req.nickname },
    }))
}

pub(crate) async fn update_user(s: &UserServiceImpl, request: Request<UpdateUserRequest>) -> Result<Response<UpdateUserResponse>, Status> {
    let req = request.into_inner();
    let user = s.state.user_repository.update(
        req.user_id,
        Some(req.nickname).filter(|x| !x.is_empty()),
        Some(req.gender).filter(|&g| g != 0),
        Some(req.address).filter(|x| !x.is_empty()),
        Some(req.avatar).filter(|x| !x.is_empty()),
    ).await.map_err(|e| Status::internal(e.to_string()))?;

    match user {
        Some(user) => Ok(Response::new(UpdateUserResponse {
            id: user.id, username: user.username,
            nickname: user.nickname.unwrap_or_default(),
        })),
        None => Err(Status::not_found("User not found")),
    }
}

pub(crate) async fn delete_user(s: &UserServiceImpl, request: Request<DeleteUserRequest>) -> Result<Response<DeleteUserResponse>, Status> {
    let req = request.into_inner();
    let success = s.state.user_repository.delete(req.user_id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(DeleteUserResponse { success }))
}

pub(crate) async fn list_users(s: &UserServiceImpl, request: Request<ListUsersRequest>) -> Result<Response<ListUsersResponse>, Status> {
    let req = request.into_inner();
    let page = req.page.max(1);
    let page_size = req.page_size.clamp(1, 100);
    let result = s.state.user_repository.list(page, page_size).await
        .map_err(|e| Status::internal(e.to_string()))?;

    let users: Vec<GetUserResponse> = result.users.into_iter().map(|user| GetUserResponse {
        id: user.id, username: user.username,
        nickname: user.nickname.unwrap_or_default(),
        avatar: user.avatar.unwrap_or_default(),
        phone: user.phone.unwrap_or_default(),
        email: user.email.unwrap_or_default(),
        gender: user.gender.unwrap_or(0),
        address: user.address.unwrap_or_default(),
        role: user.role, status: user.status,
        created_at: user.created_at.timestamp(),
        updated_at: user.updated_at.timestamp(),
    }).collect();

    Ok(Response::new(ListUsersResponse { users, total: result.total }))
}

pub(crate) async fn update_user_status(s: &UserServiceImpl, request: Request<UpdateUserStatusRequest>) -> Result<Response<UpdateUserStatusResponse>, Status> {
    let req = request.into_inner();
    let found = s.state.user_repository.update_status(req.user_id, req.status).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !found { return Err(Status::not_found("用户不存在")); }
    Ok(Response::new(UpdateUserStatusResponse { success: true }))
}

pub(crate) async fn update_user_role(s: &UserServiceImpl, request: Request<UpdateUserRoleRequest>) -> Result<Response<UpdateUserRoleResponse>, Status> {
    let req = request.into_inner();
    if req.role.is_empty() { return Err(Status::invalid_argument("角色不能为空")); }
    let found = s.state.user_repository.update_role(req.user_id, &req.role).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !found { return Err(Status::not_found("用户不存在")); }
    Ok(Response::new(UpdateUserRoleResponse { success: true }))
}

pub(crate) async fn reset_password(s: &UserServiceImpl, request: Request<ResetPasswordRequest>) -> Result<Response<ResetPasswordResponse>, Status> {
    let req = request.into_inner();
    if req.new_password.is_empty() { return Err(Status::invalid_argument("new_password is required")); }
    if req.new_password.len() < 8 { return Err(Status::invalid_argument("new_password must be at least 8 characters")); }

    use argon2::password_hash::{SaltString, rand_core::OsRng};
    use argon2::PasswordHasher;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();
    let password_hash = argon2.hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|e| Status::internal(format!("密码哈希失败: {e}")))?.to_string();

    let found = s.state.user_repository.update_password(req.user_id, &password_hash).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !found { return Err(Status::not_found("用户不存在")); }
    Ok(Response::new(ResetPasswordResponse { success: true }))
}

pub(crate) async fn batch_update_user_role(s: &UserServiceImpl, request: Request<BatchUpdateUserRoleRequest>) -> Result<Response<BatchUpdateUserRoleResponse>, Status> {
    let req = request.into_inner();
    if req.user_ids.is_empty() { return Ok(Response::new(BatchUpdateUserRoleResponse { success: true, affected: 0 })); }
    let result = s.state.user_repository.batch_update_role(&req.user_ids, &req.role).await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(BatchUpdateUserRoleResponse {
        success: result.fail_count == 0, affected: result.success_count as i32,
    }))
}

pub(crate) async fn batch_update_user_status(s: &UserServiceImpl, request: Request<BatchUpdateUserStatusRequest>) -> Result<Response<BatchUpdateUserStatusResponse>, Status> {
    let req = request.into_inner();
    if req.user_ids.is_empty() { return Ok(Response::new(BatchUpdateUserStatusResponse { success: true, affected: 0 })); }
    let result = s.state.user_repository.batch_update_status(&req.user_ids, req.status).await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(BatchUpdateUserStatusResponse {
        success: result.fail_count == 0, affected: result.success_count as i32,
    }))
}

pub(crate) async fn batch_delete_users(s: &UserServiceImpl, request: Request<BatchDeleteUsersRequest>) -> Result<Response<BatchDeleteUsersResponse>, Status> {
    let req = request.into_inner();
    if req.user_ids.is_empty() { return Ok(Response::new(BatchDeleteUsersResponse { success: true, deleted: 0 })); }
    let result = s.state.user_repository.batch_delete(&req.user_ids).await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(BatchDeleteUsersResponse {
        success: result.fail_count == 0, deleted: result.success_count as i32,
    }))
}

pub(crate) async fn import_users(s: &UserServiceImpl, request: Request<ImportUsersRequest>) -> Result<Response<ImportUsersResponse>, Status> {
    let req = request.into_inner();
    let result = s.state.user_repository.batch_create_from_csv(
        &req.data_base64, &req.format, &req.update_mode,
    ).await.map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(ImportUsersResponse {
        total: result.total as i32, success_count: result.success_count as i32,
        fail_count: result.fail_count as i32, errors: result.errors,
    }))
}

pub(crate) async fn export_users(s: &UserServiceImpl, request: Request<ExportUsersRequest>) -> Result<Response<ExportUsersResponse>, Status> {
    let req = request.into_inner();
    let data = s.state.user_repository.export_to_csv(
        &req.keyword, req.status, &req.role, &req.format,
    ).await.map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(ExportUsersResponse { data: data.0, filename: data.1, format: req.format }))
}
