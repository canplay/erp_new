//! 用户 CRUD 处理器
use super::*;

// ============ 处理器实现 ============

/// 获取用户列表
pub(crate) async fn list_users(
    State(state): State<HttpAppState>,
    Query(params): Query<UserQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(DEFAULT_PAGE_SIZE).max(MIN_PAGE);
    let page_size = params
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    match state.user_repository.list(page, page_size).await {
        Ok(result) => {
            let users: Vec<UserResponse> = result.users.into_iter().map(std::convert::Into::into).collect();
            (
                StatusCode::OK,
                json_success(serde_json::json!({
                    "list": users,
                    "total": result.total,
                    "page": page,
                    "page_size": page_size
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("查询用户列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询用户列表失败"),
            )
                .into_response()
        }
    }
}

/// 获取单个用户
pub(crate) async fn get_user(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    match state.user_repository.find_by_id(user_id).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            json_success(serde_json::json!(UserResponse::from(user))),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询用户失败"),
            )
                .into_response()
        }
    }
}

/// 创建用户
pub(crate) async fn create_user(
    State(state): State<HttpAppState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // 参数校验
    if let Err(msg) = validate_username(&req.username) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 密码处理：使用 argon2 哈希
    let password_to_hash = req.password.as_deref().unwrap_or(&default_password()).to_string();
    let password_hash = match hash_password(&password_to_hash) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("密码哈希失败: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("密码处理失败"),
            )
                .into_response();
        }
    };

    match state
        .user_repository
        .create(
            &req.username,
            &password_hash,
            req.email,
            req.nickname,
            req.phone,
            None,
        )
        .await
    {
        Ok(user_id) => (
            StatusCode::CREATED,
            json_success(serde_json::json!({
                "id": user_id,
                "username": req.username
            })),
        )
            .into_response(),
        Err(crate::repository::UserRepositoryError::AlreadyExists) => (
            StatusCode::CONFLICT,
            json_error("用户名已存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("创建用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("创建用户失败"),
            )
                .into_response()
        }
    }
}

/// 更新用户
pub(crate) async fn update_user(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match state
        .user_repository
        .update(
            user_id,
            req.nickname.clone(),
            req.gender,
            req.address.clone(),
            req.avatar.clone(),
        )
        .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            json_success(serde_json::json!(UserResponse::from(user))),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新用户失败"),
            )
                .into_response()
        }
    }
}

/// 删除用户
pub(crate) async fn delete_user(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    match state.user_repository.delete(user_id).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("用户删除成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("删除用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("删除用户失败"),
            )
                .into_response()
        }
    }
}

/// 更新用户状态
pub(crate) async fn update_user_status(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    // 状态值校验
    if let Err(msg) = validate_status(req.status) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 更新用户状态
    match state
        .user_repository
        .update_status(user_id, req.status)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("状态更新成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新用户状态失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新用户状态失败"),
            )
                .into_response()
        }
    }
}

/// 更新用户角色
pub(crate) async fn update_user_role(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    // 角色值校验
    if let Err(msg) = validate_role(&req.role) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 更新用户角色
    match state.user_repository.update_role(user_id, &req.role).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("角色更新成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新用户角色失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新用户角色失败"),
            )
                .into_response()
        }
    }
}

