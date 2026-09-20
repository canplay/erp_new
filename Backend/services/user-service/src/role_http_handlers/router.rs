//! 角色管理路由构建
use super::*;
use super::role_crud::*;
use super::perm_handlers::*;
use super::inherit_handlers::*;

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
