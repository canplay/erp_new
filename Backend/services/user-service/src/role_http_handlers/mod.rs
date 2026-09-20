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

mod inherit_handlers;
mod perm_handlers;
mod role_crud;
mod router;

pub(crate) use inherit_handlers::*;
pub(crate) use perm_handlers::*;
pub(crate) use role_crud::*;
pub(crate) use router::*;
