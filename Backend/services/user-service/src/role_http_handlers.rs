//! 角色管理 HTTP Handlers

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_handlers::HttpAppState;
use crate::repository::RoleRepositoryError;
use crate::helpers::{json_success, json_ok, json_error, json_error_fmt, json_success_msg, json_ok_msg, json_error_msg, json_error_msg_fmt};

// ============ 请求/响应结构 ============

/// 角色查询参数
#[derive(Debug, Deserialize)]
pub(crate) struct RoleQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
}

/// 创建角色请求
#[derive(Debug, Deserialize)]
pub(crate) struct CreateRoleRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub role_type: Option<String>,
    pub parent_id: Option<i64>,
}

/// 更新角色请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
}

/// 更新角色权限请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateRolePermissionsRequest {
    pub permissions: Vec<i64>,
}

/// 复制角色权限请求
#[derive(Debug, Deserialize)]
pub(crate) struct CopyPermissionsRequest {
    pub source_role: String,
    pub target_roles: Vec<String>,
}

/// 角色响应
#[derive(Debug, Serialize)]
pub(crate) struct RoleResponse {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub role_type: String,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub status: i32,
    pub is_default: bool,
    pub user_count: i64,
    pub created_at: String,
}

/// 角色列表响应
#[derive(Debug, Serialize)]
pub(crate) struct RoleListResponse {
    pub list: Vec<RoleResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 权限响应
#[derive(Debug, Serialize)]
pub(crate) struct PermissionResponse {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub permission_type: String,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub icon: Option<String>,
}

// ============ 处理器实现 ============

/// 获取角色列表
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

// ============ 数据权限 ============

/// 数据权限请求
#[derive(Debug, Deserialize)]
pub(crate) struct DataPermissionsRequest {
    pub data_permissions: Vec<serde_json::Value>,
}

/// 获取角色数据权限
pub(crate) async fn get_role_data_permissions(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state.role_repository.get_data_permissions(role.id).await {
        Ok(perms) => (
            StatusCode::OK,
            json_success(perms),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

/// 更新角色数据权限
pub(crate) async fn update_role_data_permissions(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
    Json(req): Json<DataPermissionsRequest>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state
        .role_repository
        .set_data_permissions(role.id, &req.data_permissions)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            json_ok_msg("数据权限已更新"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

// ============ 字段权限 ============

/// 字段权限请求
#[derive(Debug, Deserialize)]
pub(crate) struct FieldPermissionsRequest {
    pub field_permissions: Vec<serde_json::Value>,
}

/// 获取角色字段权限
pub(crate) async fn get_role_field_permissions(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state.role_repository.get_field_permissions(role.id).await {
        Ok(perms) => (
            StatusCode::OK,
            json_success(perms),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

/// 更新角色字段权限
pub(crate) async fn update_role_field_permissions(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
    Json(req): Json<FieldPermissionsRequest>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state
        .role_repository
        .set_field_permissions(role.id, &req.field_permissions)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            json_ok_msg("字段权限已更新"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

// ============ 权限继承 ============

/// 继承请求
#[derive(Debug, Deserialize)]
pub(crate) struct InheritRequest {
    pub inherit_from: Vec<String>,
}

/// 获取角色继承链
pub(crate) async fn get_role_inherit(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state.role_repository.get_inherit_chain(role.id).await {
        Ok(chain) => (
            StatusCode::OK,
            json_success(chain),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

/// 设置角色继承
pub(crate) async fn set_role_inherit(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
    Json(req): Json<InheritRequest>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state
        .role_repository
        .set_inherit(role.id, &req.inherit_from)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            json_ok_msg("继承关系已设置"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

/// 移除角色继承
pub(crate) async fn remove_role_inherit(
    State(state): State<HttpAppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let role = match state.role_repository.find_by_code(&code).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                json_error("角色不存在"),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error(&format!("{}", e)),
            )
                .into_response();
        }
    };

    match state.role_repository.remove_inherit(role.id).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("继承关系已移除"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::OK,
            json_ok_msg("无继承关系"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_error(&format!("{}", e)),
        )
            .into_response(),
    }
}

// ============ 路由构建 ============

/// 创建角色管理路由
pub(crate) fn create_role_router(state: HttpAppState) -> Router {
    Router::new()
        .route("/", axum::routing::get(list_roles))
        .route("/", axum::routing::post(create_role))
        .route("/permissions", axum::routing::get(list_permissions))
        .route(
            "/copy-permissions",
            axum::routing::post(copy_role_permissions),
        )
        .route("/{code}", axum::routing::get(get_role))
        .route("/{code}", axum::routing::put(update_role))
        .route("/{code}", axum::routing::delete(delete_role))
        .route(
            "/{code}/permissions",
            axum::routing::get(get_role_permissions),
        )
        .route(
            "/{code}/permissions",
            axum::routing::put(update_role_permissions),
        )
        .route("/{code}/users", axum::routing::get(get_role_users))
        // 数据权限
        .route(
            "/{code}/data-permissions",
            axum::routing::get(get_role_data_permissions),
        )
        .route(
            "/{code}/data-permissions",
            axum::routing::put(update_role_data_permissions),
        )
        // 字段权限
        .route(
            "/{code}/field-permissions",
            axum::routing::get(get_role_field_permissions),
        )
        .route(
            "/{code}/field-permissions",
            axum::routing::put(update_role_field_permissions),
        )
        // 权限继承
        .route("/{code}/inherit", axum::routing::get(get_role_inherit))
        .route("/{code}/inherit", axum::routing::post(set_role_inherit))
        .route("/{code}/inherit", axum::routing::delete(remove_role_inherit))
        .with_state(state)
}
