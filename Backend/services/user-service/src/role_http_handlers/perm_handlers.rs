//! 数据权限与字段权限处理器
use super::*;

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

