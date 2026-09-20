//! 权限继承处理器
use super::*;

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

