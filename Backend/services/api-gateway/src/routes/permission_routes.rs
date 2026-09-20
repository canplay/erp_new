//! 权限管理路由 — 权限 CRUD、角色权限配置、数据/字段权限

use std::sync::Arc;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;

async fn list_permissions() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn create_permission() -> Json<Value> {
    json_ok()
}

async fn get_permission() -> Json<Value> {
    json_success(json!(null))
}

async fn update_permission() -> Json<Value> {
    json_ok()
}

async fn delete_permission() -> Json<Value> {
    json_ok()
}

async fn batch_create_permissions() -> Json<Value> {
    json_ok()
}

async fn get_role_perm_config() -> Json<Value> {
    json_success(json!(null))
}

async fn update_role_perm_config() -> Json<Value> {
    json_ok()
}

async fn get_role_data_perms() -> Json<Value> {
    json_success(json!([]))
}

async fn set_role_data_perms() -> Json<Value> {
    json_ok()
}

async fn get_role_field_perms() -> Json<Value> {
    json_success(json!([]))
}

async fn set_role_field_perms() -> Json<Value> {
    json_ok()
}

async fn get_role_inherit() -> Json<Value> {
    json_success(json!(null))
}

async fn set_role_inherit() -> Json<Value> {
    json_ok()
}

async fn remove_role_inherit() -> Json<Value> {
    json_ok()
}

async fn get_accessible_depts() -> Json<Value> {
    json_success(json!([]))
}

async fn get_accessible_tenants() -> Json<Value> {
    json_success(json!([]))
}

async fn validate_data_perm() -> Json<Value> {
    json_success(json!({"valid": true}))
}

async fn check_sensitive_perm() -> Json<Value> {
    json_success(json!({"sensitive": false}))
}

async fn refresh_perm_cache() -> Json<Value> {
    json_ok()
}

async fn batch_assign_perms() -> Json<Value> {
    json_ok()
}

async fn copy_role_perms() -> Json<Value> {
    json_ok()
}

async fn list_perm_change_logs() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn export_perm_change_logs() -> Json<Value> {
    json_success(json!(null))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/permissions", get(list_permissions).post(create_permission))
        .route("/api/admin/permissions/{key}", get(get_permission).put(update_permission).delete(delete_permission))
        .route("/api/admin/permissions/batch", post(batch_create_permissions))
        .route("/api/admin/roles/{role_name}/permission-config", get(get_role_perm_config).put(update_role_perm_config))
        .route("/api/admin/roles/{role_name}/data-permissions", get(get_role_data_perms).put(set_role_data_perms))
        .route("/api/admin/roles/{role_name}/field-permissions", get(get_role_field_perms).put(set_role_field_perms))
        .route("/api/admin/roles/{role_name}/inherit", get(get_role_inherit).post(set_role_inherit).delete(remove_role_inherit))
        .route("/api/admin/roles/{role_name}/accessible-departments", get(get_accessible_depts))
        .route("/api/admin/roles/{role_name}/accessible-tenants", get(get_accessible_tenants))
        .route("/api/permissions/validate-data", post(validate_data_perm))
        .route("/api/permissions/check-sensitive", post(check_sensitive_perm))
        .route("/api/permissions/refresh-cache", post(refresh_perm_cache))
        .route("/api/permissions/batch-assign", post(batch_assign_perms))
        .route("/api/permissions/copy", post(copy_role_perms))
        .route("/api/permission-change-logs", get(list_perm_change_logs))
        .route("/api/permission-change-logs/export", get(export_perm_change_logs))
}
