//! 角色 CRUD 与功能权限处理器
use super::*;

pub(crate) async fn list_roles(
    State(state): State<HttpAppState>,
    Query(params): Query<RoleQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state
        .role_repository
        .list(page, page_size, params.keyword.as_deref())
        .await
    {
        Ok(result) => {
            let roles: Vec<RoleResponse> = result
                .roles
                .into_iter()
                .map(|r| RoleResponse {
                    id: r.id,
                    name: r.name,
                    code: r.code,
                    description: r.description,
                    role_type: r.role_type,
                    parent_id: None,
                    level: r.level,
                    sort_order: 0,
                    status: r.status,
                    is_default: r.is_default,
                    user_count: r.user_count,
                    created_at: r.created_at.to_rfc3339(),
                })
                .collect();

            (
                StatusCode::OK,
                json_success(serde_json::json!({
                    "list": roles,
                    "total": result.total,
                    "page": page,
                    "page_size": page_size
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("查询角色列表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询角色列表失败"),
            )
                .into_response()
        }
    }
}

/// 获取角色详情
pub(crate) async fn get_role(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => (
            StatusCode::OK,
            json_success(serde_json::json!({
                "id": role.id,
                "name": role.name,
                "code": role.code,
                "description": role.description,
                "role_type": role.role_type,
                "parent_id": role.parent_id,
                "level": role.level,
                "status": role.status,
                "is_default": role.is_default,
                "created_at": role.created_at.to_rfc3339(),
                "updated_at": role.updated_at.to_rfc3339(),
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("角色不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询角色失败"),
            )
                .into_response()
        }
    }
}

/// 创建角色
pub(crate) async fn create_role(
    State(state): State<HttpAppState>,
    Json(req): Json<CreateRoleRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() || req.code.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json_error("角色名称和代码不能为空"),
        )
            .into_response();
    }

    match state
        .role_repository
        .create(
            &req.name,
            &req.code,
            req.description,
            req.role_type.as_deref().unwrap_or("custom"),
            req.parent_id,
        )
        .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            json_success(serde_json::json!({
                "id": id,
                "code": req.code
            })),
        )
            .into_response(),
        Err(RoleRepositoryError::AlreadyExists) => (
            StatusCode::CONFLICT,
            json_error("角色代码已存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("创建角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("创建角色失败"),
            )
                .into_response()
        }
    }
}

/// 更新角色
pub(crate) async fn update_role(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    match state
        .role_repository
        .update(&code, req.name, req.description, req.status)
        .await
    {
        Ok(Some(role)) => (
            StatusCode::OK,
            json_success(serde_json::json!({
                "id": role.id,
                "name": role.name,
                "code": role.code,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("角色不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新角色失败"),
            )
                .into_response()
        }
    }
}

/// 删除角色
pub(crate) async fn delete_role(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    match state.role_repository.delete(&code).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("角色删除成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("角色不存在"),
        )
            .into_response(),
        Err(RoleRepositoryError::HasAssociatedUsers) => (
            StatusCode::CONFLICT,
            json_error("角色有用户关联，无法删除"),
        )
            .into_response(),
        Err(RoleRepositoryError::HasChildRoles) => (
            StatusCode::CONFLICT,
            json_error("角色有子角色，无法删除"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("删除角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("删除角色失败"),
            )
                .into_response()
        }
    }
}

/// 获取角色权限
pub(crate) async fn get_role_permissions(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => match state.role_repository.get_permissions(role.id).await {
            Ok(perms) => {
                let perm_list: Vec<PermissionResponse> = perms
                    .into_iter()
                    .map(|p| PermissionResponse {
                        id: p.id,
                        name: p.name,
                        code: p.code,
                        permission_type: p.permission_type,
                        parent_id: p.parent_id,
                        path: p.path,
                        method: p.method,
                        icon: p.icon,
                    })
                    .collect();

                (
                    StatusCode::OK,
                    json_success(serde_json::json!(perm_list)),
                )
                    .into_response()
            }
            Err(e) => {
                tracing::error!("获取角色权限失败: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("获取角色权限失败"),
                )
                    .into_response()
            }
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("角色不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询角色失败"),
            )
                .into_response()
        }
    }
}

/// 更新角色权限
pub(crate) async fn update_role_permissions(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
    Json(req): Json<UpdateRolePermissionsRequest>,
) -> impl IntoResponse {
    match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => {
            match state
                .role_repository
                .set_permissions(role.id, &req.permissions)
                .await
            {
                Ok(()) => (
                    StatusCode::OK,
                    json_ok_msg("权限更新成功"),
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("更新角色权限失败: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json_error("更新角色权限失败"),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("角色不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询角色失败"),
            )
                .into_response()
        }
    }
}

/// 获取角色下的用户
pub(crate) async fn get_role_users(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
    Query(params): Query<RoleQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => {
            match state
                .role_repository
                .get_users(role.id, page, page_size)
                .await
            {
                Ok((user_ids, total)) => (
                    StatusCode::OK,
                    json_success(serde_json::json!({
                        "user_ids": user_ids,
                        "total": total,
                        "page": page,
                        "page_size": page_size
                    })),
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("获取角色用户失败: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json_error("获取角色用户失败"),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("角色不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询角色失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询角色失败"),
            )
                .into_response()
        }
    }
}

/// 复制角色权限
pub(crate) async fn copy_role_permissions(
    State(state): State<HttpAppState>,
    Json(req): Json<CopyPermissionsRequest>,
) -> impl IntoResponse {
    // 获取源角色
    let source_role = match state.role_repository.find_by_code(&req.source_role).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("源角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("查询源角色失败: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询源角色失败"),
            )
                .into_response();
        }
    };

    // 获取目标角色 ID 列表
    let mut target_ids = Vec::new();
    for target_code in &req.target_roles {
        match state.role_repository.find_by_code(target_code).await {
            Ok(Some(role)) => target_ids.push(role.id),
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    json_error(&format!("目标角色 '{}' 不存在", target_code)),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("查询目标角色失败: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json_error("查询目标角色失败"),
                )
                    .into_response();
            }
        }
    }

    // 执行复制
    match state
        .role_repository
        .copy_permissions(source_role.id, &target_ids)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            json_ok_msg("权限复制成功"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("复制角色权限失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("复制角色权限失败"),
            )
                .into_response()
        }
    }
}

/// 获取所有权限列表
pub(crate) async fn list_permissions(State(state): State<HttpAppState>) -> impl IntoResponse {
    match state.role_repository.list_permissions().await {
        Ok(perms) => {
            let perm_list: Vec<PermissionResponse> = perms
                .into_iter()
                .map(|p| PermissionResponse {
                    id: p.id,
                    name: p.name,
                    code: p.code,
                    permission_type: p.permission_type,
                    parent_id: p.parent_id,
                    path: p.path,
                    method: p.method,
                    icon: p.icon,
                })
                .collect();

            (
                StatusCode::OK,
                json_success(serde_json::json!(perm_list)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("获取权限列表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("获取权限列表失败"),
            )
                .into_response()
        }
    }
}

