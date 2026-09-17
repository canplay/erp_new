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

use common::{
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
pub struct HttpAppState {
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
pub struct UserQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
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
pub struct UpdateStatusRequest {
    pub status: i32,
    pub lock_hours: Option<i32>,
}

/// 更新角色请求
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

/// 用户响应
#[derive(Debug, Serialize)]
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

// ============ 处理器实现 ============

/// 获取用户列表
pub async fn list_users(
    State(state): State<HttpAppState>,
    Query(params): Query<UserQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(DEFAULT_PAGE_SIZE).max(MIN_PAGE);
    let page_size = params
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    match state.user_repository.list(page, page_size).await {
        Ok(result) => {
            let users: Vec<UserResponse> = result.users.into_iter().map(std::convert::Into::into).collect();
            (
                StatusCode::OK,
                json_success(serde_json::json!({
                    "list": users,
                    "total": result.total,
                    "page": page,
                    "page_size": page_size
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("查询用户列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询用户列表失败"),
            )
                .into_response()
        }
    }
}

/// 获取单个用户
pub async fn get_user(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    match state.user_repository.find_by_id(user_id).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            json_success(serde_json::json!(UserResponse::from(user))),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询用户失败"),
            )
                .into_response()
        }
    }
}

/// 创建用户
pub async fn create_user(
    State(state): State<HttpAppState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // 参数校验
    if let Err(msg) = validate_username(&req.username) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 密码处理：使用 argon2 哈希
    let password_to_hash = req.password.as_deref().unwrap_or(&default_password()).to_string();
    let password_hash = match hash_password(&password_to_hash) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("密码哈希失败: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("密码处理失败"),
            )
                .into_response();
        }
    };

    match state
        .user_repository
        .create(
            &req.username,
            &password_hash,
            req.email,
            req.nickname,
            req.phone,
            None,
        )
        .await
    {
        Ok(user_id) => (
            StatusCode::CREATED,
            json_success(serde_json::json!({
                "id": user_id,
                "username": req.username
            })),
        )
            .into_response(),
        Err(crate::repository::UserRepositoryError::AlreadyExists) => (
            StatusCode::CONFLICT,
            json_error("用户名已存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("创建用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("创建用户失败"),
            )
                .into_response()
        }
    }
}

/// 更新用户
pub async fn update_user(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match state
        .user_repository
        .update(
            user_id,
            req.nickname.clone(),
            req.gender,
            req.address.clone(),
            req.avatar.clone(),
        )
        .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            json_success(serde_json::json!(UserResponse::from(user))),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新用户失败"),
            )
                .into_response()
        }
    }
}

/// 删除用户
pub async fn delete_user(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    match state.user_repository.delete(user_id).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("用户删除成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("删除用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("删除用户失败"),
            )
                .into_response()
        }
    }
}

/// 更新用户状态
pub async fn update_user_status(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    // 状态值校验
    if let Err(msg) = validate_status(req.status) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 更新用户状态
    match state
        .user_repository
        .update_status(user_id, req.status)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("状态更新成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新用户状态失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新用户状态失败"),
            )
                .into_response()
        }
    }
}

/// 更新用户角色
pub async fn update_user_role(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    // 角色值校验
    if let Err(msg) = validate_role(&req.role) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 更新用户角色
    match state.user_repository.update_role(user_id, &req.role).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("角色更新成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新用户角色失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新用户角色失败"),
            )
                .into_response()
        }
    }
}

// ============ 批量操作请求/响应 ============

/// 批量更新角色请求
#[derive(Debug, Deserialize)]
pub struct BatchUpdateRoleRequest {
    pub user_ids: Vec<i64>,
    pub role: String,
}

/// 重置密码请求
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: Option<String>,
}

/// 重置用户密码
pub async fn reset_user_password(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    // 生成或使用提供的密码
    let password_to_hash = req.new_password.as_deref().unwrap_or(&default_password()).to_string();
    let password_hash = match hash_password(&password_to_hash) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("密码哈希失败: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("密码处理失败"),
            )
                .into_response();
        }
    };

    match state
        .user_repository
        .update_password(user_id, &password_hash)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "new_password": password_to_hash
            }), "密码重置成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("重置密码失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("重置密码失败"),
            )
                .into_response()
        }
    }
}

/// 获取导入模板（返回 CSV 模板结构）
pub async fn get_import_template() -> impl IntoResponse {
    let template = "username,password,email,nickname,phone,role\nuser1,,user1@example.com,用户1,13800000001,user\nuser2,,user2@example.com,用户2,13800000002,user\n";

    (
        StatusCode::OK,
        json_success(serde_json::json!({
            "template": template,
            "columns": ["username", "password", "email", "nickname", "phone", "role"],
            "description": "用户导入模板，请按此格式填写 CSV 文件"
        })),
    )
        .into_response()
}

/// 下载导入模板（返回 CSV 格式）
pub async fn download_import_template() -> impl IntoResponse {
    let csv_content = "username,password,email,nickname,phone,role\nuser1,,user1@example.com,用户1,13800000001,user\nuser2,,user2@example.com,用户2,13800000002,user\n";

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/csv; charset=utf-8")
        .header(
            "Content-Disposition",
            "attachment; filename=user_import_template.csv",
        )
        .body(axum::body::Body::from(csv_content))
        .unwrap_or_else(|e| {
            tracing::error!("构建下载模板响应失败: {e}");
            axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from("内部服务器错误"))
                .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
        })
        .into_response()
}

/// 导入用户（解析上传的 CSV 文件）
pub async fn import_users(
    State(state): State<HttpAppState>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let body = req.into_body();
    let bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("读取上传文件失败: {e}");
            return (
                StatusCode::BAD_REQUEST,
                json_error("无法读取上传文件"),
            )
                .into_response();
        }
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(bytes.as_ref());

    let mut total = 0;
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        total += 1;

        match result {
            Ok(record) => {
                if record.len() < 6 {
                    fail_count += 1;
                    errors.push(format!("行 {}: 列数不足", idx + 2));
                    continue;
                }

                let username = record.get(0).unwrap_or("").trim();
                let password = record.get(1).unwrap_or("").trim();
                let email = record.get(2).unwrap_or("").trim();
                let nickname = record.get(3).unwrap_or("").trim();
                let phone = record.get(4).unwrap_or("").trim();
                let _role = record.get(5).unwrap_or("user").trim();

                // 参数校验
                if let Err(msg) = validate_username(username) {
                    fail_count += 1;
                    errors.push(format!("行 {}: {}", idx + 2, msg));
                    continue;
                }

                // 密码哈希
                let password_to_hash = if password.is_empty() {
                    default_password()
                } else {
                    password.to_string()
                };
                let password_hash = match hash_password(&password_to_hash) {
                    Ok(h) => h,
                    Err(e) => {
                        fail_count += 1;
                        errors.push(format!("行 {}: 密码处理失败 - {}", idx + 2, e));
                        continue;
                    }
                };

                // 创建用户
                match state
                    .user_repository
                    .create(
                        username,
                        &password_hash,
                        if email.is_empty() {
                            None
                        } else {
                            Some(email.to_string())
                        },
                        if nickname.is_empty() {
                            None
                        } else {
                            Some(nickname.to_string())
                        },
                        if phone.is_empty() {
                            None
                        } else {
                            Some(phone.to_string())
                        },
                        Some(1), // gender default
                    )
                    .await
                {
                    Ok(_) => {
                        success_count += 1;
                        tracing::info!("成功导入用户: {username}");
                    }
                    Err(crate::repository::UserRepositoryError::AlreadyExists) => {
                        fail_count += 1;
                        errors.push(format!("行 {}: 用户名 '{}' 已存在", idx + 2, username));
                    }
                    Err(e) => {
                        fail_count += 1;
                        errors.push(format!("行 {}: 创建用户失败 - {:?}", idx + 2, e));
                    }
                }
            }
            Err(e) => {
                fail_count += 1;
                errors.push(format!("行 {}: 解析错误 - {}", idx + 2, e));
            }
        }
    }

    tracing::info!(
        "用户导入完成: 总数={total}, 成功={success_count}, 失败={fail_count}"
    );

    (
        StatusCode::OK,
        json_success_msg(serde_json::json!({
            "total": total,
            "success": success_count,
            "failed": fail_count,
            "errors": if errors.len() > 10 {
                errors[..10].to_vec()
            } else {
                errors
            }
        }), &format!("导入完成: 成功 {}", success_count)),
    )
        .into_response()
}

/// 导出用户（生成 CSV 文件）
pub async fn export_users(
    State(state): State<HttpAppState>,
    Query(_params): Query<UserQueryParams>,
) -> impl IntoResponse {
    let page_size = 10000; // 一次导出最大数量

    match state.user_repository.list(1, page_size).await {
        Ok(result) => {
            // 预分配 String 容量以提高性能
            let mut csv_content = String::with_capacity(result.users.len() * 200);
            csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");
            csv_content.push('\n');

            for user in result.users {
                use std::fmt::Write as FmtWrite;
                let _ = writeln!(
                    csv_content,
                    "{},{},{},{},{},{},{},{}",
                    user.id,
                    user.username,
                    user.nickname.unwrap_or_default(),
                    user.email.unwrap_or_default(),
                    user.phone.unwrap_or_default(),
                    user.role,
                    user.status,
                    user.created_at.to_rfc3339()
                );
            }

            axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/csv; charset=utf-8")
                .header(
                    "Content-Disposition",
                    "attachment; filename=users_export.csv",
                )
                .body(axum::body::Body::from(csv_content))
                .unwrap_or_else(|e| {
                    tracing::error!("构建导出用户响应失败: {e}");
                    axum::response::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from("内部服务器错误"))
                        .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
                })
                .into_response()
        }
        Err(e) => {
            tracing::error!("导出用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("导出用户失败"),
            )
                .into_response()
        }
    }
}

/// 批量更新状态请求
#[derive(Debug, Deserialize)]
pub struct BatchUpdateStatusRequest {
    pub user_ids: Vec<i64>,
    pub status: i32,
}

/// 批量删除请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub user_ids: Vec<i64>,
}

/// 批量更新用户角色（使用事务优化）
pub async fn batch_update_user_role(
    State(state): State<HttpAppState>,
    Json(req): Json<BatchUpdateRoleRequest>,
) -> impl IntoResponse {
    // 角色值校验
    if let Err(msg) = validate_role(&req.role) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 使用事务批量更新
    match state.user_repository.batch_update_role(&req.user_ids, &req.role).await {
        Ok(result) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            }), &format!("成功更新 {} 个用户的角色", result.success_count)),
        ).into_response(),
        Err(e) => {
            tracing::error!("批量更新用户角色失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("批量更新用户角色失败"),
            ).into_response()
        }
    }
}

/// 批量更新用户状态（使用事务优化）
pub async fn batch_update_user_status(
    State(state): State<HttpAppState>,
    Json(req): Json<BatchUpdateStatusRequest>,
) -> impl IntoResponse {
    // 状态值校验
    if let Err(msg) = validate_status(req.status) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 使用事务批量更新
    match state.user_repository.batch_update_status(&req.user_ids, req.status).await {
        Ok(result) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            }), &format!("成功更新 {} 个用户的状态", result.success_count)),
        ).into_response(),
        Err(e) => {
            tracing::error!("批量更新用户状态失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("批量更新用户状态失败"),
            ).into_response()
        }
    }
}

/// 批量删除用户（使用事务优化）
pub async fn batch_delete_users(
    State(state): State<HttpAppState>,
    Json(req): Json<BatchDeleteRequest>,
) -> impl IntoResponse {
    // 使用事务批量删除
    match state.user_repository.batch_delete(&req.user_ids).await {
        Ok(result) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            }), &format!("成功删除 {} 个用户", result.success_count)),
        ).into_response(),
        Err(e) => {
            tracing::error!("批量删除用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("批量删除用户失败"),
            ).into_response()
        }
    }
}

/// 健康检查
pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        json_health("user-service"),
    )
        .into_response()
}

// ============ 路由构建 ============

/// 创建 HTTP 路由器
pub fn create_http_router(state: HttpAppState) -> Router {
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
