use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{error, info};
use serde_json::json;
use std::sync::Arc;

use crate::error::{ApiKeyError, Result};
use crate::models::{
    ApiKey, CreateKeyRequest, KeyQuery, KeyStats, PageResult, UpdateKeyRequest, ValidateKeyRequest,
};
use crate::repository::{ApiKeyRepository, PostgresApiKeyRepository};

/// HTTP 请求处理器
///
/// 使用数据库持久化存储 API Keys
/// 应用状态 - 使用数据库持久化
#[derive(Clone)]
pub struct AppState {
    /// 数据库仓库
    pub repository: Arc<PostgresApiKeyRepository>,
}

/// 从请求头获取用户ID的辅助函数
/// API Gateway 会将 JWT 验证后的用户信息通过 x-user-id header 传递
fn extract_user_id(headers: &axum::http::HeaderMap) -> i64 {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1) // 默认用户ID为1（匿名用户或开发模式）
}

/// 创建API密钥
/// POST /api/apikeys
pub async fn create_key(
    State(state): State<AppState>,
    headers: Extension<axum::http::HeaderMap>,
    Json(payload): Json<CreateKeyRequest>,
) -> Result<impl IntoResponse> {
    let user_id = extract_user_id(&headers);

    let (mut api_key, key_id, key_secret) = ApiKey::generate(
        payload.name,
        payload.description,
        payload.permission_level,
        user_id,
    );

    api_key.allowed_ips = payload.allowed_ips.unwrap_or_default();
    api_key.rate_limit = payload.rate_limit.unwrap_or(1000);
    api_key.expires_at = payload.expires_at;
    api_key.tenant_id = payload.tenant_id;

    if let Err(e) = state.repository.create(&api_key).await {
        error!("创建 API Key 失败: {}", e);
        return Err(ApiKeyError::DatabaseError(e));
    }

    info!("创建API密钥: {}", api_key.id);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "code": 201,
            "message": "密钥创建成功，请妥善保管密钥Secret，仅此次可见",
            "data": {
                "id": api_key.id,
                "key_id": key_id,
                "key_secret": key_secret,
                "name": api_key.name,
                "permission_level": api_key.permission_level.to_string(),
                "expires_at": api_key.expires_at
            }
        })),
    ))
}

/// 获取密钥列表
/// GET /api/apikeys
pub async fn list_keys(
    State(state): State<AppState>,
    headers: Extension<axum::http::HeaderMap>,
    Query(query): Query<KeyQuery>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers);
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match state
        .repository
        .list_by_user(user_id, page, page_size)
        .await
    {
        Ok((keys, total)) => {
            let page_keys: Vec<_> = keys
                .iter()
                .map(|k| {
                    serde_json::json!({
                        "id": k.id,
                        "name": k.name,
                        "key_prefix": k.key_id,
                        "description": k.description,
                        "permission_level": k.permission_level.to_string(),
                        "status": k.status.clone(),
                        "expires_at": k.expires_at,
                        "last_used_at": k.last_used_at,
                        "created_at": k.created_at
                    })
                })
                .collect();

            Json(json!({
                "code": 200,
                "message": "操作成功",
                "data": PageResult {
                    records: page_keys,
                    total,
                    page: query.page.unwrap_or(1),
                    page_size: query.page_size.unwrap_or(20)
                }
            }))
        }
        Err(e) => {
            error!("获取密钥列表失败: {}", e);
            Json(json!({
                "code": 500,
                "message": "获取密钥列表失败",
                "data": null
            }))
        }
    }
}

/// 获取密钥详情
/// GET /api/apikeys/:id
pub async fn get_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => Ok(Json(json!({
            "code": 200,
            "message": "操作成功",
            "data": key
        }))),
        Ok(None) => Err(ApiKeyError::KeyNotFound(id)),
        Err(e) => {
            error!("获取密钥详情失败: {}", e);
            Err(ApiKeyError::DatabaseError(e))
        }
    }
}

/// 更新密钥
/// PUT /api/apikeys/:id
pub async fn update_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateKeyRequest>,
) -> Result<impl IntoResponse> {
    let existing = match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => key,
        Ok(None) => return Err(ApiKeyError::KeyNotFound(id)),
        Err(e) => return Err(ApiKeyError::DatabaseError(e)),
    };

    let mut updated = existing;
    if let Some(name) = payload.name {
        updated.name = name;
    }
    if let Some(description) = payload.description {
        updated.description = Some(description);
    }
    if let Some(permission) = payload.permission_level {
        updated.permission_level = permission;
    }
    if let Some(allowed_ips) = payload.allowed_ips {
        updated.allowed_ips = allowed_ips;
    }
    if let Some(rate_limit) = payload.rate_limit {
        updated.rate_limit = rate_limit;
    }
    if let Some(status) = payload.status {
        updated.status = status;
    }
    if let Some(expires_at) = payload.expires_at {
        updated.expires_at = Some(expires_at);
    }
    updated.updated_at = Utc::now();

    if let Err(e) = state.repository.update(&updated).await {
        error!("更新 API Key 失败: {}", e);
        return Err(ApiKeyError::DatabaseError(e));
    }

    info!("更新API密钥: {}", id);

    Ok(Json(json!({
        "code": 200,
        "message": "更新成功",
        "data": updated
    })))
}

/// 删除密钥
/// DELETE /api/apikeys/:id
pub async fn delete_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    match state.repository.delete(&id).await {
        Ok(()) => {
            info!("删除API密钥: {}", id);
            Ok(Json(json!({
                "code": 200,
                "message": "删除成功"
            })))
        }
        Err(e) => {
            error!("删除 API Key 失败: {}", e);
            Err(ApiKeyError::DatabaseError(e))
        }
    }
}

/// 验证密钥
/// POST /api/apikeys/validate
pub async fn validate_key(
    State(state): State<AppState>,
    Json(payload): Json<ValidateKeyRequest>,
) -> impl IntoResponse {
    match state.repository.find_by_key_id(&payload.key).await {
        Ok(Some(key)) => {
            if key.status != "active" {
                return Json(json!({
                    "valid": false,
                    "error": "密钥已被禁用"
                }));
            }

            if key.expires_at.is_some_and(|e| e < Utc::now()) {
                return Json(json!({
                    "valid": false,
                    "error": "密钥已过期"
                }));
            }

            if !key.allowed_ips.is_empty()
                && let Some(ref ip) = payload.ip_address
                    && !key.allowed_ips.iter().any(|p| p == ip) {
                        return Json(json!({
                            "valid": false,
                            "error": "IP地址不被允许"
                        }));
                    }

            let _ = state.repository.update_last_used(&key.id).await;

            Json(json!({
                "valid": true,
                "key_id": key.key_id,
                "permission_level": key.permission_level.to_string()
            }))
        }
        Ok(None) => Json(json!({
            "valid": false,
            "error": "密钥无效"
        })),
        Err(e) => {
            error!("验证密钥失败: {}", e);
            Json(json!({
                "valid": false,
                "error": "验证失败"
            }))
        }
    }
}

/// 禁用密钥
/// POST /api/apikeys/:id/disable
pub async fn disable_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let existing = match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => key,
        Ok(None) => return Err(ApiKeyError::KeyNotFound(id)),
        Err(e) => return Err(ApiKeyError::DatabaseError(e)),
    };

    let mut updated = existing;
    updated.status = "inactive".to_string();
    updated.updated_at = Utc::now();

    if let Err(e) = state.repository.update(&updated).await {
        error!("禁用 API Key 失败: {}", e);
        return Err(ApiKeyError::DatabaseError(e));
    }

    info!("禁用API密钥: {}", id);

    Ok(Json(json!({
        "code": 200,
        "message": "密钥已禁用"
    })))
}

/// 启用密钥
/// POST /api/apikeys/:id/enable
pub async fn enable_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let existing = match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => key,
        Ok(None) => return Err(ApiKeyError::KeyNotFound(id)),
        Err(e) => return Err(ApiKeyError::DatabaseError(e)),
    };

    let mut updated = existing;
    updated.status = "active".to_string();
    updated.updated_at = Utc::now();

    if let Err(e) = state.repository.update(&updated).await {
        error!("启用 API Key 失败: {}", e);
        return Err(ApiKeyError::DatabaseError(e));
    }

    info!("启用API密钥: {}", id);

    Ok(Json(json!({
        "code": 200,
        "message": "密钥已启用"
    })))
}

/// 获取统计信息
/// GET /api/apikeys/stats
pub async fn get_stats(
    State(state): State<AppState>,
    headers: Extension<axum::http::HeaderMap>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers);
    match state.repository.list_by_user(user_id, 1, 1).await {
        Ok((_, total)) => Json(json!({
            "code": 200,
            "message": "操作成功",
            "data": KeyStats {
                total_keys: total,
                active_keys: total,
                expired_keys: 0,
                revoked_keys: 0,
                total_requests: 0,
                failed_requests: 0
            }
        })),
        Err(e) => {
            error!("获取统计信息失败: {}", e);
            Json(json!({
                "code": 500,
                "message": "获取统计信息失败",
                "data": null
            }))
        }
    }
}
