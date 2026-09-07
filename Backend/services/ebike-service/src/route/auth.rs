//! 用户认证路由
//!
//! 提供登录、用户信息查询和登出端点。

use auth_core::{AuthenticatedUser, PasswordService};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use serde_json::{json, Value};

use crate::AppState;

/// 用户登录
///
/// 验证用户名密码，返回 JWT token 及用户基本信息。
pub async fn login(
    State(state): State<AppState>,
    AxumJson(req): AxumJson<Value>,
) -> Response {
    let username = req["username"].as_str().unwrap_or("");
    let password = req["password"].as_str().unwrap_or("");

    match state.user_repo.find_by_username(username).await {
        Ok(Some(user)) => {
            let password_ok = {
                let pwd = PasswordService;
                pwd.verify_bcrypt(password, &user.password_hash)
            };

            if !password_ok {
                return common::unauthorized_response("username or password error").into_response();
            }

            match state.jwt_service.generate_access_token(
                user.id,
                &user.username,
                &user.role,
            ) {
                Ok(token) => common::ok_response(json!({
                    "token": token,
                    "user": {
                        "id": user.id,
                        "username": user.username,
                        "nickname": user.nickname,
                        "avatar": user.avatar,
                        "role": user.role,
                    }
                })).into_response(),
                Err(e) => common::internal_error_response(
                    &format!("token generation failed: {e:?}")
                ).into_response(),
            }
        }
        Ok(None) => common::unauthorized_response("username or password error").into_response(),
        Err(e) => common::internal_error_response(&format!("{e:?}")).into_response(),
    }
}

pub async fn info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Response {
    let user_id = user.0.user_id;

    match state.user_repo.user_info(user_id).await {
        Ok(Some(user)) => common::ok_response(json!({
            "id": user.id,
            "username": user.username,
            "nickname": user.nickname,
            "avatar": user.avatar,
            "phone": user.phone,
            "email": user.email,
            "gender": user.gender,
            "address": user.address,
            "role": user.role,
            "status": user.status,
        })).into_response(),
        Ok(None) => common::not_found_response("user not found").into_response(),
        Err(e) => common::internal_error_response(&format!("{e:?}")).into_response(),
    }
}

pub async fn loginout() -> Response {
    common::success_with_message_response("ok").into_response()
}
