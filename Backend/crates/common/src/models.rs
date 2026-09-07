//! 统一数据模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 用户基本信息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
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
    pub created_at: i64,
    pub updated_at: i64,
}

/// 用户状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum UserStatus {
    Disabled = 0,
    Enabled = 1,
    Locked = 2,
}

impl From<i32> for UserStatus {
    fn from(v: i32) -> Self {
        match v {
            1 => Self::Enabled,
            2 => Self::Locked,
            _ => Self::Disabled,
        }
    }
}

impl From<UserStatus> for i32 {
    fn from(v: UserStatus) -> Self {
        v as Self
    }
}

/// 性别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum Gender {
    Unknown = 0,
    Male = 1,
    Female = 2,
}

impl From<i32> for Gender {
    fn from(v: i32) -> Self {
        match v {
            1 => Self::Male,
            2 => Self::Female,
            _ => Self::Unknown,
        }
    }
}

impl From<Gender> for i32 {
    fn from(v: Gender) -> Self {
        v as Self
    }
}

// ==================== 请求结构 ====================

/// 登录请求
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: Option<String>,
}

/// Token验证请求
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

/// 刷新令牌请求
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 获取用户请求
#[derive(Debug, Clone, Deserialize)]
pub struct GetUserRequest {
    pub user_id: i64,
}

/// 更新用户请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub gender: Option<i32>,
    pub address: Option<String>,
}

/// 分页查询请求
#[derive(Debug, Clone, Deserialize)]
pub struct ListUsersRequest {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
}

// ==================== 响应结构 ====================

/// 登录响应
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: Option<String>,
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub expires_in: i64,
}

/// 注册响应
#[derive(Debug, Clone, Serialize)]
pub struct RegisterResponse {
    pub token: String,
    pub user_id: i64,
    pub username: String,
}

/// Token验证响应
#[derive(Debug, Clone, Serialize)]
pub struct VerifyTokenResponse {
    pub valid: bool,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub role: Option<String>,
}

/// 用户信息响应
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
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
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            avatar: user.avatar,
            phone: user.phone,
            email: user.email,
            gender: user.gender,
            address: user.address,
            role: user.role,
            status: user.status,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// 用户列表响应
#[derive(Debug, Clone, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

impl Pagination {
    #[must_use]
    pub fn new(page: i64, page_size: i64, total: i64) -> Self {
        Self {
            page,
            page_size,
            total,
            total_pages: (total as f64 / page_size as f64).ceil() as i64,
        }
    }
}
