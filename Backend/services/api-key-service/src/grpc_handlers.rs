//! gRPC Service Handlers for API Key Service
//!
//! 提供 API Key 管理的 gRPC 接口
//!
//! 使用数据库持久化存储 API Keys

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::Status;
use uuid::Uuid;

use crate::models::{ApiKey, KeyUsageLog};
use crate::repository::{ApiKeyRepository, PostgresApiKeyRepository};

/// API Key 应用状态（使用数据库持久化）
#[derive(Clone)]
pub struct ApiKeyAppState {
    /// 数据库仓库
    pub repository: Arc<PostgresApiKeyRepository>,
    /// 内存缓存（用于 gRPC 服务，减少数据库查询）
    cache: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl ApiKeyAppState {
    /// 创建新的应用状态
    #[must_use]
    pub fn new(repository: Arc<PostgresApiKeyRepository>) -> Self {
        Self {
            repository,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从缓存获取或从数据库加载
    pub async fn get_key(&self, id: &str) -> Option<ApiKey> {
        // 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(key) = cache.get(id) {
                return Some(key.clone());
            }
        }

        // 从数据库加载
        if let Ok(Some(key)) = self.repository.find_by_id(id).await {
            let mut cache = self.cache.write().await;
            cache.insert(id.to_string(), key.clone());
            return Some(key);
        }

        None
    }

    /// 从缓存获取或从数据库加载（按 `key_id`）
    pub async fn get_key_by_key_id(&self, key_id: &str) -> Option<ApiKey> {
        // 先检查缓存
        {
            let cache = self.cache.read().await;
            for key in cache.values() {
                if key.key_id == key_id {
                    return Some(key.clone());
                }
            }
        }

        // 从数据库加载
        if let Ok(Some(key)) = self.repository.find_by_key_id(key_id).await {
            let mut cache = self.cache.write().await;
            cache.insert(key.id.clone(), key.clone());
            return Some(key);
        }

        None
    }

    /// 更新缓存
    pub async fn update_cache(&self, key: ApiKey) {
        let mut cache = self.cache.write().await;
        cache.insert(key.id.clone(), key);
    }

    /// 从缓存移除
    pub async fn remove_from_cache(&self, id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(id);
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 生成新的 `key_id`
    #[must_use]
    pub fn generate_key_id() -> String {
        format!("ak_{}" , &Uuid::new_v4().to_string().replace('-', "" )[..16])
    }
}

/// API Key 信息
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub description: Option<String>,
    pub permission_level: String,
    pub allowed_ips: Option<Vec<String>>,
    pub rate_limit: Option<i64>,
    pub status: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub user_id: String,
    pub tenant_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ApiKey> for ApiKeyInfo {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id.clone(),
            name: key.name,
            key_prefix: key.key_id.clone(),
            description: key.description,
            permission_level: key.permission_level.to_string(),
            allowed_ips: Some(key.allowed_ips),
            rate_limit: Some(key.rate_limit),
            status: key.status.clone(),
            expires_at: key.expires_at.map(|t| t.to_rfc3339()),
            last_used_at: key.last_used_at.map(|t| t.to_rfc3339()),
            user_id: key.user_id.map(|id| id.to_string()).unwrap_or_default(),
            tenant_id: key.tenant_id.map(|id| id.to_string()),
            created_at: key.created_at.to_rfc3339(),
            updated_at: key.updated_at.to_rfc3339(),
        }
    }
}

/// 使用记录信息
#[derive(Debug, Clone)]
pub struct KeyUsageLogInfo {
    pub id: String,
    pub key_id: String,
    pub api_path: String,
    pub method: String,
    pub status_code: i32,
    pub request_size: i64,
    pub response_size: i64,
    pub response_time_ms: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

impl From<KeyUsageLog> for KeyUsageLogInfo {
    fn from(log: KeyUsageLog) -> Self {
        Self {
            id: log.id.clone(),
            key_id: log.key_id,
            api_path: log.api_path,
            method: log.method,
            status_code: log.status_code,
            request_size: log.request_size,
            response_size: log.response_size,
            response_time_ms: log.response_time_ms,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            error_message: log.error_message,
            created_at: log.created_at.to_rfc3339(),
        }
    }
}

/// 密钥统计信息
#[derive(Debug, Clone, Default)]
pub struct KeyStatsInfo {
    pub total_keys: i64,
    pub active_keys: i64,
    pub expired_keys: i64,
    pub revoked_keys: i64,
    pub total_requests: i64,
    pub failed_requests: i64,
}

/// 分页 Key 响应
#[derive(Debug, Clone, Default)]
pub struct PaginatedApiKeysInfo {
    pub keys: Vec<ApiKeyInfo>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 分页使用记录响应
#[derive(Debug, Clone, Default)]
pub struct PaginatedUsageLogsInfo {
    pub logs: Vec<KeyUsageLogInfo>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 验证响应
#[derive(Debug, Clone)]
pub struct ValidateKeyResponse {
    pub valid: bool,
    pub key_id: Option<String>,
    pub permission_level: Option<String>,
    pub error: Option<String>,
}

/// 创建密钥响应
#[derive(Debug, Clone)]
pub struct CreateKeyResponse {
    pub id: String,
    pub key: String,
    pub key_prefix: String,
}

/// 创建 API Key 参数
#[derive(Debug, Clone)]
pub struct CreateApiKeyParams {
    pub name: String,
    pub description: Option<String>,
    pub permission_level: i32,
    pub allowed_ips: Option<Vec<String>>,
    pub rate_limit: Option<i64>,
    pub expires_at: Option<String>,
    pub user_id: String,
    pub tenant_id: Option<String>,
}

/// 更新 API Key 参数
#[derive(Debug, Clone)]
pub struct UpdateApiKeyParams {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_level: Option<i32>,
    pub allowed_ips: Option<Vec<String>>,
    pub rate_limit: Option<i64>,
    pub status: Option<i32>,
    pub expires_at: Option<String>,
}

// ============== Key 管理接口 ==============

/// 创建 API Key
pub async fn create_api_key(
    state: Arc<ApiKeyAppState>,
    params: CreateApiKeyParams,
) -> Result<CreateKeyResponse, Status> {
    tracing::info!("创建 API Key: name={}, user_id={}" , params.name, params.user_id);

    // 生成密钥
    let key_id = ApiKeyAppState::generate_key_id();
    let secret = Uuid::new_v4().to_string().replace('-', "" );
    let hint = format!("****{}" , &secret[secret.len().saturating_sub(4)..]);

    // 解析过期时间
    let expires_at = params.expires_at.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    // 构建 API Key
    let id = Uuid::new_v4().to_string();
    let api_key = ApiKey {
        id: id.clone(),
        name: params.name,
        description: params.description,
        key_id: key_id.clone(),
        secret_key_hash: ApiKey::hash_key(&secret), // 审计修复: 存储 SHA-256 哈希, 不再存明文
        key_hint: hint,
        permission_level: params.permission_level,
        allowed_ips: params.allowed_ips.unwrap_or_default(),
        rate_limit: params.rate_limit.unwrap_or(1000),
        tenant_id: params.tenant_id.and_then(|s| s.parse().ok()),
        user_id: params.user_id.parse().ok(),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expires_at,
        last_used_at: None,
    };

    // 保存到数据库
    if let Err(e) = state.repository.create(&api_key).await {
        tracing::error!("创建 API Key 失败: {e}" );
        return Err(Status::internal("创建 API Key 失败" ));
    }

    // 更新缓存
    state.update_cache(api_key).await;

    Ok(CreateKeyResponse {
        id,
        key: format!("{key_id}{secret}" ),
        key_prefix: key_id,
    })
}

/// 获取 Key 列表
pub async fn list_api_keys(
    state: Arc<ApiKeyAppState>,
    page: i32,
    page_size: i32,
    _status: Option<i32>,
    keyword: Option<String>,
    user_id: Option<String>,
    tenant_id: Option<String>,
) -> Result<PaginatedApiKeysInfo, Status> {
    tracing::info!("获取 Key 列表: page={page}, keyword={keyword:?}" );

    // 解析用户 ID
    let user_id = user_id.and_then(|s| s.parse().ok());
    let tenant_id: i64 = tenant_id.and_then(|s| s.parse().ok()).unwrap_or(0);
    let page = i64::from(page.max(1));
    let page_size = i64::from(page_size.clamp(1, 100));

    // 从数据库查询
    if let Some(uid) = user_id {
        match state.repository.list_by_user(uid, tenant_id, page, page_size).await {
            Ok((keys, total)) => {
                let key_infos: Vec<ApiKeyInfo> = keys.into_iter().map(ApiKeyInfo::from).collect();
                return Ok(PaginatedApiKeysInfo {
                    keys: key_infos,
                    total,
                    page: page as i32,
                    page_size: page_size as i32,
                });
            }
            Err(e) => {
                tracing::error!("获取密钥列表失败: {e}" );
                return Err(Status::internal("获取密钥列表失败" ));
            }
        }
    }

    // 返回空列表（需要更完整的实现）
    Ok(PaginatedApiKeysInfo::default())
}

/// 获取 Key 详情
pub async fn get_api_key(
    state: Arc<ApiKeyAppState>,
    id: String,
) -> Result<Option<ApiKeyInfo>, Status> {
    tracing::info!("获取 Key 详情: {id}" );

    if let Some(key) = state.get_key(&id).await {
        Ok(Some(ApiKeyInfo::from(key)))
    } else {
        Ok(None)
    }
}

/// 更新 Key
pub async fn update_api_key(
    state: Arc<ApiKeyAppState>,
    params: UpdateApiKeyParams,
) -> Result<bool, Status> {
    tracing::info!("更新 API Key: {}" , params.id);

    // 获取现有密钥
    let Some(mut key) = state.get_key(&params.id).await else {
        return Err(Status::not_found("API Key not found" ));
    };

    // 更新字段
    if let Some(n) = &params.name {
        key.name = n.clone();
    }
    if let Some(d) = &params.description {
        key.description = Some(d.clone());
    }
    if let Some(p) = params.permission_level {
        key.permission_level = p;
    }
    if let Some(ips) = &params.allowed_ips {
        key.allowed_ips = ips.clone();
    }
    if let Some(rl) = params.rate_limit {
        key.rate_limit = rl;
    }
    if let Some(s) = params.status {
        key.status = match s {
            0 | 1 => "active".to_string(),
            _ => "inactive".to_string(),
        };
    }
    if let Some(e) = &params.expires_at {
        key.expires_at = chrono::DateTime::parse_from_rfc3339(e)
            .ok()
            .map(|dt| dt.with_timezone(&Utc));
    }
    key.updated_at = Utc::now();

    // 保存到数据库
    if let Err(e) = state.repository.update(&key).await {
        tracing::error!("更新 API Key 失败: {e}" );
        return Err(Status::internal("更新 API Key 失败" ));
    }

    // 更新缓存
    state.update_cache(key).await;

    Ok(true)
}

/// 删除 Key
pub async fn delete_api_key(state: Arc<ApiKeyAppState>, id: String) -> Result<bool, Status> {
    tracing::info!("删除 API Key: {id}" );

    // 从数据库删除
    if let Err(e) = state.repository.delete(&id).await {
        tracing::error!("删除 API Key 失败: {e}" );
        return Err(Status::internal("删除 API Key 失败" ));
    }

    // 从缓存移除
    state.remove_from_cache(&id).await;

    Ok(true)
}

/// 批量删除 Key
pub async fn batch_delete_api_keys(
    state: Arc<ApiKeyAppState>,
    ids: Vec<String>,
) -> Result<i64, Status> {
    tracing::info!("批量删除 API Keys: count={}" , ids.len());

    // N+1 FIX: Use single batch DELETE instead of per-row loop
    let count = state.repository.batch_delete(&ids).await
        .map_err(|e| Status::internal(format!("批量删除 API Key 失败: {e}")))?;

    // Clear cache for each key (batched operation, but cache is in-memory so it's fast)
    for id in &ids {
        state.remove_from_cache(id).await;
    }

    Ok(count as i64)
}

/// 获取密钥统计
pub async fn get_key_statistics(
    state: Arc<ApiKeyAppState>,
    user_id: Option<String>,
    tenant_id: Option<String>,
) -> Result<KeyStatsInfo, Status> {
    tracing::info!("获取密钥统计: user_id={user_id:?}" );

    // 解析用户 ID
    let user_id = match user_id.and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => return Ok(KeyStatsInfo::default()),
    };
    let tenant_id: i64 = tenant_id.and_then(|s| s.parse().ok()).unwrap_or(0);

    // 从数据库获取列表来统计
    match state.repository.list_by_user(user_id, tenant_id, 1, 1000).await {
        Ok((keys, total)) => {
            let now = Utc::now();
            let mut active = 0i64;
            let mut expired = 0i64;

            for key in &keys {
                if key.status == "active" {
                    if let Some(exp) = key.expires_at {
                        if exp < now {
                            expired += 1;
                        } else {
                            active += 1;
                        }
                    } else {
                        active += 1;
                    }
                }
            }

            Ok(KeyStatsInfo {
                total_keys: total,
                active_keys: active,
                expired_keys: expired,
                revoked_keys: total - active - expired,
                total_requests: 0, // 需要使用日志表统计
                failed_requests: 0,
            })
        }
        Err(e) => {
            tracing::error!("获取密钥统计失败: {e}" );
            Err(Status::internal("获取密钥统计失败" ))
        }
    }
}

// ============== Key 验证接口 ==============

/// 验证 API Key
pub async fn validate_api_key(
    state: Arc<ApiKeyAppState>,
    key: String,
    ip_address: Option<String>,
    _api_path: Option<String>,
    _method: Option<String>,
) -> Result<ValidateKeyResponse, Status> {
    let prefix = if key.len() > 8 { &key[..8] } else { &key };
    tracing::info!("验证 API Key: prefix={prefix}" );

    // 提取 key_id（假设格式为 ak_xxx + secret）
    let key_id = if key.starts_with("ak_" ) && key.len() > 24 {
        key[..24].to_string()
    } else {
        key.clone()
    };

    // 查找密钥
    let Some(mut api_key) = state.get_key_by_key_id(&key_id).await else {
        return Ok(ValidateKeyResponse {
            valid: false,
            key_id: None,
            permission_level: None,
            error: Some("Invalid API Key".to_string()),
        });
    };

    // 检查状态
    if api_key.status != "active" {
        return Ok(ValidateKeyResponse {
            valid: false,
            key_id: Some(api_key.key_id),
            permission_level: Some(api_key.permission_level.to_string()),
            error: Some("API Key is disabled".to_string()),
        });
    }

    // 检查过期
    if let Some(expires) = api_key.expires_at
        && expires < Utc::now() {
            return Ok(ValidateKeyResponse {
                valid: false,
                key_id: Some(api_key.key_id),
                permission_level: Some(api_key.permission_level.to_string()),
                error: Some("API Key has expired".to_string()),
            });
        }

    // 检查 IP 白名单
    if !api_key.allowed_ips.is_empty()
        && let Some(ref ip) = ip_address
            && !api_key.allowed_ips.contains(ip) {
                return Ok(ValidateKeyResponse {
                    valid: false,
                    key_id: Some(api_key.key_id),
                    permission_level: Some(api_key.permission_level.to_string()),
                    error: Some("IP address not allowed".to_string()),
                });
            }

    // 更新最后使用时间
    api_key.last_used_at = Some(Utc::now());
    if let Err(e) = state.repository.update_last_used(&api_key.id).await {
        tracing::warn!("更新最后使用时间失败: {e}" );
    }
    state.update_cache(api_key.clone()).await;

    Ok(ValidateKeyResponse {
        valid: true,
        key_id: Some(api_key.key_id),
        permission_level: Some(api_key.permission_level.to_string()),
        error: None,
    })
}

/// 禁用 Key
pub async fn disable_api_key(state: Arc<ApiKeyAppState>, id: String) -> Result<bool, Status> {
    tracing::info!("禁用 API Key: {id}" );

    update_key_status(state, id, "inactive" ).await
}

/// 启用 Key
pub async fn enable_api_key(state: Arc<ApiKeyAppState>, id: String) -> Result<bool, Status> {
    tracing::info!("启用 API Key: {id}" );

    update_key_status(state, id, "active" ).await
}

/// 撤销 Key
pub async fn revoke_api_key(
    state: Arc<ApiKeyAppState>,
    id: String,
    _reason: Option<String>,
) -> Result<bool, Status> {
    tracing::info!("撤销 API Key: {id}" );

    // 撤销等同于禁用
    update_key_status(state, id, "revoked" ).await
}

/// 更新密钥状态（辅助函数）
async fn update_key_status(
    state: Arc<ApiKeyAppState>,
    id: String,
    new_status: &'static str,
) -> Result<bool, Status> {
    // 获取现有密钥
    let Some(mut key) = state.get_key(&id).await else {
        return Err(Status::not_found("API Key not found" ));
    };

    // 更新状态
    key.status = new_status.to_string();
    key.updated_at = Utc::now();

    // 保存到数据库
    if let Err(e) = state.repository.update(&key).await {
        tracing::error!("更新密钥状态失败: {e}" );
        return Err(Status::internal("更新密钥状态失败" ));
    }

    // 更新缓存
    state.update_cache(key).await;

    Ok(true)
}

// ============== 使用记录接口 ==============

/// 获取使用记录列表
pub async fn list_usage_logs(
    state: Arc<ApiKeyAppState>,
    key_id: Option<String>,
    _api_path: Option<String>,
    _status_code: Option<i32>,
    _start_date: Option<String>,
    _end_date: Option<String>,
    page: i32,
    page_size: i32,
) -> Result<PaginatedUsageLogsInfo, Status> {
    tracing::info!("获取使用记录: key_id={key_id:?}" );

    let page = i64::from(page.max(1));
    let page_size = i64::from(page_size.clamp(1, 100));

    match state
        .repository
        .list_usage_logs(key_id.as_deref(), page, page_size)
        .await
    {
        Ok((logs, total)) => {
            let log_infos: Vec<KeyUsageLogInfo> = logs
                .into_iter()
                .map(|log| KeyUsageLogInfo {
                    id: log.id,
                    key_id: log.key_id.clone(),
                    api_path: log.endpoint,
                    method: log.method,
                    status_code: log.status_code.unwrap_or(0),
                    request_size: log.request_size.unwrap_or(0),
                    response_size: log.response_size.unwrap_or(0),
                    response_time_ms: i64::from(log.latency_ms.unwrap_or(0)),
                    ip_address: log.ip_address,
                    user_agent: log.user_agent,
                    error_message: None,
                    created_at: log.created_at.to_rfc3339(),
                })
                .collect();

            Ok(PaginatedUsageLogsInfo {
                logs: log_infos,
                total,
                page: page as i32,
                page_size: page_size as i32,
            })
        }
        Err(e) => {
            tracing::error!("获取使用记录失败: {e}" );
            Err(Status::internal("获取使用记录失败" ))
        }
    }
}

/// 获取 Key 的使用记录
pub async fn get_key_usage_logs(
    state: Arc<ApiKeyAppState>,
    key_id: String,
    page: i32,
    page_size: i32,
) -> Result<PaginatedUsageLogsInfo, Status> {
    tracing::info!("获取 Key 使用记录: key_id={key_id}" );

    list_usage_logs(state, Some(key_id), None, None, None, None, page, page_size).await
}

/// 清除使用记录
pub async fn clear_usage_logs(
    state: Arc<ApiKeyAppState>,
    key_id: Option<String>,
    before_date: Option<String>,
) -> Result<i64, Status> {
    tracing::info!("清除使用记录: key_id={key_id:?}" );

    let before = before_date.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    match state
        .repository
        .delete_usage_logs(key_id.as_deref(), before)
        .await
    {
        Ok(count) => {
            tracing::info!("已清除 {count} 条使用记录" );
            Ok(count)
        }
        Err(e) => {
            tracing::error!("清除使用记录失败: {e}" );
            Err(Status::internal("清除使用记录失败" ))
        }
    }
}
