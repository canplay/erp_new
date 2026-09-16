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

use common::AppError;
use common::AppResult;
use crate::models::{
    ApiKey, CreateKeyRequest, KeyQuery, KeyStats, PageResult, UpdateKeyRequest, ValidateKeyRequest,
};
use crate::repository::{ApiKeyRepository, PostgresApiKeyRepository};
use crate::helpers::{
    json_api_key_created, json_api_key_list, json_api_key_error, json_api_key_detail,
    json_api_key_updated, json_api_key_deleted, json_api_key_disabled, json_api_key_expired,
    json_api_key_ip_not_allowed, json_api_key_invalid, json_api_key_valid, json_api_key_stats,
    json_api_key_simple, json_validation_error
};

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
        .get("x-user-id" )
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
) -> AppResult<impl IntoResponse> {
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
        error!("创建 API Key 失败: {}" , e);
        return Err(AppError::Database(e));
            }

    info!("创建API密钥: {}" , api_key.id);

    Ok(json_api_key_created(&key_id, &key_secret, &api_key.name, &api_key.permission_level.to_string(), api_key.expires_at.map(|d| d.to_rfc3339())))
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

            json_api_key_list(page_keys, total, query.page.unwrap_or(1), query.page_size.unwrap_or(20))
        }
        Err(e) => {
            error!("获取密钥列表失败: {}" , e);
            json_api_key_error(500, "获取密钥列表失败" )
        }
    }
}

/// 获取密钥详情
/// GET /api/apikeys/:id
pub async fn get_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => Ok(json_api_key_detail(&key)),
        Ok(None) => Err(AppError::ApiKeyNotFound(id)),
        Err(e) => {
            error!("获取密钥详情失败: {}" , e);
            Err(AppError::DatabaseError(e.to_string()))
        }
    }
}

/// 更新密钥
/// PUT /api/apikeys/:id
pub async fn update_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateKeyRequest>,
) -> AppResult<impl IntoResponse> {
    let existing = match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => key,
        Ok(None) => return Err(AppError::ApiKeyNotFound(id)),
        Err(e) => return Err(AppError::DatabaseError(e.to_string())),
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
        error!("更新 API Key 失败: {}" , e);
        return Err(AppError::Database(e));
            }

    info!("更新API密钥: {}" , id);

    Ok(json_api_key_updated(&updated))
}

/// 删除密钥
/// DELETE /api/apikeys/:id
pub async fn delete_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    match state.repository.delete(&id).await {
        Ok(()) => {
            info!("删除API密钥: {}" , id);
            Ok(json_api_key_deleted())
        }
        Err(e) => {
            error!("删除 API Key 失败: {}" , e);
            Err(AppError::DatabaseError(e.to_string()))
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
                return json_api_key_disabled();
            }

            if key.expires_at.is_some_and(|e| e < Utc::now()) {
                return json_api_key_expired();
            }

            if !key.allowed_ips.is_empty()
                && let Some(ref ip) = payload.ip_address
                    && !key.allowed_ips.iter().any(|p| p == ip) {
                        return json_api_key_ip_not_allowed();
                    }

            let _ = state.repository.update_last_used(&key.id).await;

            json_api_key_valid(&key.key_id, &key.permission_level.to_string())
        }
        Ok(None) => json_api_key_invalid(),
        Err(e) => {
            error!("验证密钥失败: {}" , e);
            json_validation_error("验证失败" )
        }
    }
}

/// 禁用密钥
/// POST /api/apikeys/:id/disable
pub async fn disable_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let existing = match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => key,
        Ok(None) => return Err(AppError::ApiKeyNotFound(id)),
        Err(e) => return Err(AppError::DatabaseError(e.to_string())),
    };

    let mut updated = existing;
    updated.status = "inactive".to_string();
    updated.updated_at = Utc::now();

    if let Err(e) = state.repository.update(&updated).await {
        error!("禁用 API Key 失败: {}" , e);
        return Err(AppError::Database(e));
            }

    info!("禁用API密钥: {}" , id);

    Ok(json_api_key_simple("密钥已禁用" ))
}

/// 启用密钥
/// POST /api/apikeys/:id/enable
pub async fn enable_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let existing = match state.repository.find_by_id(&id).await {
        Ok(Some(key)) => key,
        Ok(None) => return Err(AppError::ApiKeyNotFound(id)),
        Err(e) => return Err(AppError::DatabaseError(e.to_string())),
    };

    let mut updated = existing;
    updated.status = "active".to_string();
    updated.updated_at = Utc::now();

    if let Err(e) = state.repository.update(&updated).await {
        error!("启用 API Key 失败: {}" , e);
        return Err(AppError::Database(e));
            }

    info!("启用API密钥: {}" , id);

    Ok(json_api_key_simple("密钥已启用" ))
}

/// 获取统计信息
/// GET /api/apikeys/stats
pub async fn get_stats(
    State(state): State<AppState>,
    headers: Extension<axum::http::HeaderMap>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers);
    match state.repository.list_by_user(user_id, 1, 1).await {
        Ok((_, total)) => json_api_key_stats(total),
        Err(e) => {
            error!("获取统计信息失败: {}" , e);
json_api_key_error(500, "获取统计信息失败" )
        }
    }
}
