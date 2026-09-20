//! 健康检查与路由构建
use axum::Router;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use crate::helpers::json_health;
use super::HttpAppState;
use super::user_crud::{
    create_user, delete_user, get_user, list_users, update_user, update_user_role,
    update_user_status,
};
use super::batch_import_export::{
    batch_delete_users, batch_update_user_role, batch_update_user_status,
    download_import_template, export_users, get_import_template, import_users,
    reset_user_password,
};

pub(crate) async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        json_health("user-service"),
    )
        .into_response()
}


// ============ 路由构建 ============

/// 创建 HTTP 路由器
pub(crate) fn create_http_router(state: HttpAppState) -> Router {
    Router::new()
        // 用户 CRUD
        .route("/", axum::routing::get(list_users))
        .route("/", axum::routing::post(create_user))
        .route("/{user_id}", axum::routing::get(get_user))
        .route("/{user_id}", axum::routing::put(update_user))
        .route("/{user_id}", axum::routing::delete(delete_user))
        .route("/{user_id}/status", axum::routing::put(update_user_status))
        .route("/{user_id}/role", axum::routing::put(update_user_role))
        // 重置密码
        .route(
            "/{user_id}/reset-password",
            axum::routing::post(reset_user_password),
        )
        // 批量操作
        .route("/batch-role", axum::routing::put(batch_update_user_role))
        .route(
            "/batch-status",
            axum::routing::put(batch_update_user_status),
        )
        .route("/batch-delete", axum::routing::post(batch_delete_users))
        // 导入导出
        .route("/import-template", axum::routing::get(get_import_template))
        .route(
            "/import-template/download",
            axum::routing::get(download_import_template),
        )
        .route("/import", axum::routing::post(import_users))
        .route("/export", axum::routing::get(export_users))
        // 健康检查
        .route("/health", axum::routing::get(health))
        .with_state(state)
}
