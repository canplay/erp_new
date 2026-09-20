//! HTTP REST API Handlers
//!
//! 提供 HTTP REST API 接口供 API Gateway 调用

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use common::constants::{
    DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, MAX_USERNAME_LENGTH, MIN_PAGE,
    MIN_USERNAME_LENGTH, VALID_ROLES, VALID_STATUSES, default_password,
};
use crate::helpers::{
    json_success, json_ok, json_error, json_error_fmt, json_success_msg, json_ok_msg,
    json_error_msg, json_error_msg_fmt, json_health,
};

/// 密码哈希（使用 argon2）
fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("密码哈希失败: {e}"))
}


/// HTTP 应用状态
#[derive(Clone)]
pub(crate) struct HttpAppState {
    pub inner: crate::handlers::AppState,
}

impl std::ops::Deref for HttpAppState {
    type Target = crate::handlers::AppState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// ============ 请求/响应结构 ============

/// 用户查询参数
#[derive(Debug, Deserialize)]
pub(crate) struct UserQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub(crate) struct CreateUserRequest {
    pub username: String,
    pub password: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateUserRequest {
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<i32>,
    pub address: Option<String>,
    pub avatar: Option<String>,
    pub role: Option<String>,
    pub status: Option<i32>,
}

/// 更新状态请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateStatusRequest {
    pub status: i32,
    pub lock_hours: Option<i32>,
}

/// 更新角色请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateRoleRequest {
    pub role: String,
}

/// 用户响应
#[derive(Debug, Serialize)]
pub(crate) struct UserResponse {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub gender: i32,
    pub address: Option<String>,
    pub role: String,
    pub status: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::repository::UserDetail> for UserResponse {
    fn from(user: crate::repository::UserDetail) -> Self {
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            avatar: user.avatar,
            phone: user.phone,
            email: user.email,
            gender: user.gender.unwrap_or(0),
            address: user.address,
            role: user.role,
            status: user.status,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

impl From<crate::repository::UserListItem> for UserResponse {
    fn from(user: crate::repository::UserListItem) -> Self {
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            avatar: user.avatar,
            phone: user.phone,
            email: user.email,
            gender: user.gender.unwrap_or(0),
            address: user.address,
            role: user.role,
            status: user.status,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

// ============ 参数校验 ============

/// 验证用户名格式
fn validate_username(username: &str) -> Result<(), &'static str> {
    if username.is_empty() {
        return Err("用户名不能为空");
    }
    if username.len() < MIN_USERNAME_LENGTH {
        return Err("用户名长度不能少于3个字符");
    }
    if username.len() > MAX_USERNAME_LENGTH {
        return Err("用户名长度不能超过50个字符");
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("用户名只能包含字母、数字和下划线");
    }
    Ok(())
}

/// 验证角色值
fn validate_role(role: &str) -> Result<(), &'static str> {
    if !VALID_ROLES.contains(&role) {
        return Err("无效的角色值");
    }
    Ok(())
}

/// 验证状态值
fn validate_status(status: i32) -> Result<(), &'static str> {
    if !VALID_STATUSES.contains(&status) {
        return Err("无效的状态值");
    }
    Ok(())
}


mod batch_import_export;
mod router;
mod user_crud;

pub(crate) use batch_import_export::*;
pub(crate) use router::{create_http_router, health};
pub(crate) use user_crud::*;
