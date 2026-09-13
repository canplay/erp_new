//! Auth 服务路由 — HTTP → gRPC 转发
//!
//! @date 2026-07-17

use std::sync::Arc;
use axum::{
    Router,
    extract::{State, Extension},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{AppState, middleware::JwtClaims};
use crate::routes::helpers::{json_error, json_error_response_fmt, json_success};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// ============ Handler: 登录 ============

/// 记录登录日志（成功 status=1，失败 status=2）
async fn record_login_log(
    state: &Arc<AppState>,
    user_id: i64,
    username: &str,
    headers: &axum::http::HeaderMap,
    login_status: i32,
    fail_reason: &str,
) {
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let mut client = match state.grpc_clients.read().await.audit_client().await {
        Ok(c) => c,
        Err(_) => return, // 审计服务不可用时静默失败，不影响登录主流程
    };
    let _ = client
        .create_login_log(
            user_id,
            username.to_string(),
            ip_address,
            user_agent,
            String::new(),
            login_status,
            fail_reason.to_string(),
            "password".to_string(),
        )
        .await;
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut client = match state.grpc_clients.read().await.auth_client().await {
        Ok(c) => c,
        Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "认证服务不可用: {e}", &format!("认证服务不可用: {e}") )
    };

    match client.login(username.clone(), password).await {
        Ok(resp) => {
            // 登录成功，记录日志
            record_login_log(&state, resp.user_id, &resp.username, &headers, 1, "").await;
            
            // 生成 refresh_token（使用独立的 JWT，role="refresh"）
            let refresh_token = match state.jwt_service.generate_refresh_token(resp.user_id) {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("Failed to generate refresh token: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, json_error(&format!("生成刷新令牌失败: {}", e)));
                }
            };
            
            (StatusCode::OK, json_success(json!({
                "token": resp.token, "access_token": resp.token, "refresh_token": refresh_token,
                "user_id": resp.user_id, "username": resp.username, "role": resp.role,
                "must_change_password": resp.must_change_password,
            })))
        }
        Err(e) => {
            // 登录失败，记录日志（用户 ID 未知传 0）
            record_login_log(&state, 0, &username, &headers, 2, &format!("{e}")).await;
            (StatusCode::UNAUTHORIZED, json_error(&format!("登录失败: {e}")))
        }
    }
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let mut client = match state.grpc_clients.read().await.auth_client().await {
        Ok(c) => c,
        Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "认证服务不可用: {e}", &format!("认证服务不可用: {e}") )
    };

    match client.register(
        body.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        body.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        body.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        body.get("phone").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        body.get("nickname").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    ).await {
        Ok(resp) => (StatusCode::OK, json_success(json!({
            "token": resp.token, "user_id": resp.user_id, "username": resp.username
        }))),
        Err(e) => return json_error_response_fmt(StatusCode::BAD_REQUEST, "注册失败: {e}", &format!("注册失败: {e}") )
    }
}

async fn get_user_info_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
) -> impl IntoResponse {
    let mut client = match state.grpc_clients.read().await.user_client().await {
        Ok(c) => c,
        Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "用户服务不可用: {e}", &format!("用户服务不可用: {e}") )
    };

    match client.get_user(claims.sub).await {
        Ok(resp) => (StatusCode::OK, json_success(json!({
            "id": resp.id, "username": resp.username, "nickname": resp.nickname,
            "avatar": resp.avatar, "phone": resp.phone, "email": resp.email,
            "gender": resp.gender, "address": resp.address, "role": resp.role,
            "status": resp.status, "created_at": resp.created_at, "updated_at": resp.updated_at,
        }))),
        Err(e) => return json_error_response_fmt(StatusCode::INTERNAL_SERVER_ERROR, "获取用户信息失败: {e}", &format!("获取用户信息失败: {e}") )
    }
}

async fn refresh_token_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let refresh_token = body.get("refresh_token").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut client = match state.grpc_clients.read().await.auth_client().await {
        Ok(c) => c,
        Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "认证服务不可用: {e}", &format!("认证服务不可用: {e}") )
    };

    match client.refresh_token(refresh_token).await {
        Ok(resp) => (StatusCode::OK, json_success(json!({
            "access_token": resp.token, "user_id": resp.user_id, "username": resp.username, "role": resp.role
        }))),
        Err(e) => return json_error_response_fmt(StatusCode::UNAUTHORIZED, "Token刷新失败: {e}", &format!("Token刷新失败: {e}") )
    }
}

async fn update_profile_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let mut client = match state.grpc_clients.read().await.user_client().await {
        Ok(c) => c,
        Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "用户服务不可用: {e}", &format!("用户服务不可用: {e}") )
    };

    match client.update_user(
        claims.sub,
        body.get("nickname").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        body.get("avatar").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        body.get("gender").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        body.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    ).await {
        Ok(resp) => (StatusCode::OK, json_success(json!({
            "id": resp.id, "username": resp.username, "nickname": resp.nickname
        }))),
        Err(e) => return json_error_response_fmt(StatusCode::INTERNAL_SERVER_ERROR, "更新资料失败: {e}", &format!("更新资料失败: {e}") )
    }
}

async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let old_password = body.get("oldPassword")
        .or_else(|| body.get("old_password"))
        .and_then(|v| v.as_str()).unwrap_or("").to_string();
    let new_password = body.get("newPassword")
        .or_else(|| body.get("new_password"))
        .and_then(|v| v.as_str()).unwrap_or("").to_string();

    // 审计修复 (B4): 新密码强度校验(与注册策略一致: ≥8位, 含字母+数字)
    if new_password.len() < 8
        || !new_password.chars().any(|c| c.is_ascii_alphabetic())
        || !new_password.chars().any(|c| c.is_ascii_digit())
    {
        return (StatusCode::BAD_REQUEST, json_error("新密码必须不少于 8 位且同时包含字母和数字"));
    }

    // 先用旧密码验证
    let auth_ok = {
        let mut auth_client = match state.grpc_clients.read().await.auth_client().await {
            Ok(c) => c,
            Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "认证服务不可用: {e}", &format!("认证服务不可用: {e}") )
        };
        auth_client.login(claims.username.clone(), old_password).await.is_ok()
    };

    if !auth_ok {
        return (StatusCode::UNAUTHORIZED, json_error("原密码验证失败"));
    }

    // 通过 user-service gRPC 更新密码（reset_password 同时清除 must_change_password 标记）
    let update_result = {
        let mut user_client = match state.grpc_clients.read().await.user_client().await {
            Ok(c) => c,
            Err(e) => return json_error_response_fmt(StatusCode::SERVICE_UNAVAILABLE, "用户服务不可用: {e}", &format!("用户服务不可用: {e}") )
        };
        user_client.reset_password(claims.sub, new_password).await
    };
    match update_result {
        Ok(_) => (StatusCode::OK, json_success(json!({"message": "密码修改成功"}))),
        Err(e) => return json_error_response_fmt(StatusCode::INTERNAL_SERVER_ERROR, "密码修改失败: {e}", &format!("密码修改失败: {e}") )
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/user/login", post(login_handler))
        .route("/api/user/register", post(register_handler))
        .route("/api/auth/refresh", post(refresh_token_handler))
        .route("/api/user/info", get(get_user_info_handler).put(update_profile_handler))
        .route("/api/user/avatar", put(update_profile_handler))
        .route("/api/user/password", put(change_password_handler))
}
