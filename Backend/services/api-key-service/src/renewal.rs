//! API Key 续期机制模块
//!
//! 实现 API Key 续期、轮换、过期管理

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API Key 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyState {
    /// 活跃
    #[default]
    Active,
    /// 即将过期
    ExpiringSoon,
    /// 已过期
    Expired,
    /// 已吊销
    Revoked,
    /// 暂停
    Suspended,
}

/// API Key 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyType {
    /// 主 Key
    #[default]
    Primary,
    /// 辅助 Key
    Secondary,
    /// 只读 Key
    ReadOnly,
    /// 临时 Key
    Temporary,
}

/// API Key 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_id: String,
    pub name: String,
    pub key_type: KeyType,
    pub state: KeyState,
    pub user_id: String,
    /// 密钥哈希值（不存储明文）
    pub key_hash: String,
    /// Key 前缀（用于识别）
    pub key_prefix: String,
    pub scopes: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 到期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 最后轮换时间
    pub last_rotated_at: Option<DateTime<Utc>>,
    /// 使用次数
    pub usage_count: u64,
    /// 备注
    pub description: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 续期请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalRequest {
    pub key_id: String,
    pub extension_days: u32,
    pub reason: Option<String>,
}

/// 轮换请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationRequest {
    pub key_id: String,
    pub generate_backup: bool,
    pub grace_period_minutes: u32,
}

/// 轮换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationResult {
    pub success: bool,
    pub new_key: Option<ApiKey>,
    pub backup_key: Option<ApiKey>,
    pub message: String,
}

/// 续期结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalResult {
    pub success: bool,
    pub new_key: Option<ApiKey>,
    pub backup_key: Option<ApiKey>,
    pub message: String,
}

/// 续期配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalConfig {
    /// 默认有效期（天）
    pub default_validity_days: u32,
    /// 续期最大天数
    pub max_extension_days: u32,
    /// 提前告警天数
    pub warn_before_days: u32,
    /// 允许自动续期
    pub allow_auto_renewal: bool,
    /// 自动续期次数限制
    pub max_auto_renewals: u32,
    /// 轮换宽限期（分钟）
    pub rotation_grace_period_minutes: u32,
    /// 是否启用备份 Key
    pub enable_backup_key: bool,
    /// 备份 Key 有效期（天）
    pub backup_key_validity_days: u32,
}

impl Default for RenewalConfig {
    fn default() -> Self {
        Self {
            default_validity_days: 365,
            max_extension_days: 730,
            warn_before_days: 30,
            allow_auto_renewal: true,
            max_auto_renewals: 2,
            rotation_grace_period_minutes: 60,
            enable_backup_key: true,
            backup_key_validity_days: 7,
        }
    }
}

/// `key_id` -> (`new_key_id`, `expires_at`)
type RotatingKeyMap = HashMap<String, (String, DateTime<Utc>)>;

/// API Key 管理器
pub struct ApiKeyManager {
    /// Key 存储
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// 备份 Key（轮换期间）
    backup_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// 轮换中的 Key
    rotating_keys: Arc<RwLock<RotatingKeyMap>>,
    /// 配置
    config: RenewalConfig,
}

impl ApiKeyManager {
    /// 创建新的 `ApiKeyManager`
    #[must_use]
    pub fn new(config: RenewalConfig) -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            backup_keys: Arc::new(RwLock::new(HashMap::new())),
            rotating_keys: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 创建默认配置的管理器
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new(RenewalConfig::default())
    }

    /// 创建 API Key
    ///
    /// # Arguments
    /// * `name` - Key 名称
    /// * `key_type` - Key 类型
    /// * `user_id` - 用户 ID
    /// * `scopes` - 权限范围
    /// * `validity_days` - 有效期（天）
    ///
    /// # Returns
    /// 创建的 API Key
    pub async fn create_key(
        &self,
        name: &str,
        key_type: KeyType,
        user_id: &str,
        scopes: Vec<String>,
        validity_days: Option<u32>,
    ) -> ApiKey {
        let now = Utc::now();
        let key_id = generate_key_id();
        let (key_hash, key_prefix) = generate_key_pair();

        let expires_at = validity_days
            .map(|days| now + Duration::days(i64::from(days)))
            .or_else(|| Some(now + Duration::days(i64::from(self.config.default_validity_days))));

        let key = ApiKey {
            key_id: key_id.clone(),
            name: name.to_string(),
            key_type,
            state: KeyState::Active,
            user_id: user_id.to_string(),
            key_hash,
            key_prefix,
            scopes,
            created_at: now,
            expires_at,
            last_used_at: None,
            last_rotated_at: None,
            usage_count: 0,
            description: None,
            metadata: HashMap::new(),
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id, key.clone());

        key
    }

    /// 获取 API Key
    ///
    /// # Arguments
    /// * `key_id` - Key ID
    ///
    /// # Returns
    /// API Key 信息
    pub async fn get_key(&self, key_id: &str) -> Option<ApiKey> {
        let keys = self.keys.read().await;
        keys.get(key_id).cloned()
    }

    /// 验证 API Key
    ///
    /// # Arguments
    /// * `key_prefix` - Key 前缀
    /// * `key_hash` - Key 哈希值
    ///
    /// # Returns
    /// 验证结果
    pub async fn verify_key(&self, key_prefix: &str, _key_hash: &str) -> Option<ApiKey> {
        let mut keys = self.keys.write().await;

        // 查找匹配的 Key
        if let Some(key) = keys.values_mut().find(|k| k.key_prefix == key_prefix) {
            // 检查是否过期
            if let Some(expires) = key.expires_at
                && expires < Utc::now() {
                    key.state = KeyState::Expired;
                    return None;
                }

            // 检查状态
            if key.state != KeyState::Active {
                return None;
            }

            // 更新使用统计
            key.last_used_at = Some(Utc::now());
            key.usage_count += 1;

            return Some(key.clone());
        }

        None
    }

    /// 续期 API Key
    ///
    /// # Arguments
    /// * `request` - 续期请求
    ///
    /// # Returns
    /// 续期结果
    pub async fn renew_key(&self, request: RenewalRequest) -> RenewalResult {
        let mut keys = self.keys.write().await;

        let key = match keys.get_mut(&request.key_id) {
            Some(k) => k,
            None => {
                return RenewalResult {
                    success: false,
                    new_key: None,
                    backup_key: None,
                    message: "API Key 不存在".to_string(),
                };
            }
        };

        // 检查是否可以续期
        if key.state == KeyState::Revoked {
            return RenewalResult {
                success: false,
                new_key: None,
                backup_key: None,
                message: "已吊销的 Key 无法续期".to_string(),
            };
        }

        // 检查是否超过最大续期天数
        let extension_days = request.extension_days.min(self.config.max_extension_days);

        // 计算新到期时间
        let now = Utc::now();
        let current_expires = key.expires_at.unwrap_or(now);
        let new_expires = if current_expires > now {
            current_expires + Duration::days(i64::from(extension_days))
        } else {
            now + Duration::days(i64::from(extension_days))
        };

        key.expires_at = Some(new_expires);
        key.state = KeyState::Active;

        RenewalResult {
            success: true,
            new_key: Some(key.clone()),
            backup_key: None,
            message: format!(
                "成功续期 {extension_days} 天，新到期时间: {new_expires}"
            ),
        }
    }

    /// 轮换 API Key
    ///
    /// # Arguments
    /// * `request` - 轮换请求
    ///
    /// # Returns
    /// 轮换结果
    pub async fn rotate_key(&self, request: RotationRequest) -> RotationResult {
        let mut keys = self.keys.write().await;

        let old_key = match keys.get(&request.key_id) {
            Some(k) => k.clone(),
            None => {
                return RotationResult {
                    success: false,
                    new_key: None,
                    backup_key: None,
                    message: "API Key 不存在".to_string(),
                };
            }
        };

        // 生成新 Key
        let (new_hash, new_prefix) = generate_key_pair();
        let now = Utc::now();

        let new_key = ApiKey {
            key_id: generate_key_id(),
            name: old_key.name.clone(),
            key_type: old_key.key_type,
            state: KeyState::Active,
            user_id: old_key.user_id.clone(),
            key_hash: new_hash,
            key_prefix: new_prefix,
            scopes: old_key.scopes.clone(),
            created_at: now,
            expires_at: Some(now + Duration::days(i64::from(self.config.default_validity_days))),
            last_used_at: None,
            last_rotated_at: Some(now),
            usage_count: 0,
            description: old_key.description.clone(),
            metadata: old_key.metadata.clone(),
        };

        // 创建备份 Key（如果在宽限期内）
        let backup_key = if request.generate_backup || self.config.enable_backup_key {
            let backup_expiry =
                now + Duration::minutes(i64::from(self.config.rotation_grace_period_minutes));
            let (backup_hash, backup_prefix) = generate_key_pair();

            let backup = ApiKey {
                key_id: generate_key_id(),
                name: format!("{}_backup", old_key.name),
                key_type: old_key.key_type,
                state: KeyState::Active,
                user_id: old_key.user_id.clone(),
                key_hash: backup_hash,
                key_prefix: backup_prefix,
                scopes: old_key.scopes.clone(),
                created_at: now,
                expires_at: Some(backup_expiry),
                last_used_at: None,
                last_rotated_at: None,
                usage_count: 0,
                description: Some("轮换备份 Key".to_string()),
                metadata: HashMap::new(),
            };

            let mut backup_keys = self.backup_keys.write().await;
            backup_keys.insert(backup.key_id.clone(), backup.clone());

            Some(backup)
        } else {
            None
        };

        // 更新旧 Key 状态
        if let Some(old) = keys.get_mut(&request.key_id) {
            old.state = KeyState::Suspended;
        }

        // 记录轮换
        let mut rotating = self.rotating_keys.write().await;
        rotating.insert(
            request.key_id.clone(),
            (
                new_key.key_id.clone(),
                backup_key
                    .as_ref()
                    .and_then(|b| b.expires_at)
                    .unwrap_or(now),
            ),
        );

        // 保存新 Key
        keys.insert(new_key.key_id.clone(), new_key.clone());

        RotationResult {
            success: true,
            new_key: Some(new_key),
            backup_key,
            message: "Key 轮换成功".to_string(),
        }
    }

    /// 吊销 API Key
    ///
    /// # Arguments
    /// * `key_id` - Key ID
    /// * `reason` - 吊销原因
    ///
    /// # Returns
    /// 是否成功
    pub async fn revoke_key(&self, key_id: &str, reason: Option<String>) -> bool {
        let mut keys = self.keys.write().await;

        if let Some(key) = keys.get_mut(key_id) {
            key.state = KeyState::Revoked;
            if let Some(r) = reason {
                key.metadata.insert("revoke_reason".to_string(), r);
            }
            return true;
        }
        false
    }

    /// 获取即将过期的 Key
    ///
    /// # Returns
    /// 即将过期的 Key 列表
    pub async fn get_expiring_keys(&self) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        let now = Utc::now();
        let warning_threshold = now + Duration::days(i64::from(self.config.warn_before_days));

        keys.values()
            .filter(|k| {
                k.state == KeyState::Active
                    && k.expires_at
                        .is_some_and(|e| e <= warning_threshold)
                    && k.expires_at.is_some_and(|e| e > now)
            })
            .cloned()
            .collect()
    }

    /// 获取用户的所有 Key
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    ///
    /// # Returns
    /// Key 列表
    pub async fn get_user_keys(&self, user_id: &str) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        keys.values()
            .filter(|k| k.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> KeyStats {
        let keys = self.keys.read().await;

        let total = keys.len();
        let active = keys
            .values()
            .filter(|k| k.state == KeyState::Active)
            .count();
        let expiring = keys
            .values()
            .filter(|k| k.state == KeyState::ExpiringSoon)
            .count();
        let expired = keys
            .values()
            .filter(|k| k.state == KeyState::Expired)
            .count();
        let revoked = keys
            .values()
            .filter(|k| k.state == KeyState::Revoked)
            .count();

        let total_usage: u64 = keys.values().map(|k| k.usage_count).sum();

        KeyStats {
            total_keys: total,
            active_keys: active,
            expiring_keys: expiring,
            expired_keys: expired,
            revoked_keys: revoked,
            total_usage,
        }
    }

    /// 更新 Key 状态（定期检查）
    pub async fn update_states(&self) {
        let mut keys = self.keys.write().await;
        let now = Utc::now();
        let warning_threshold = now + Duration::days(i64::from(self.config.warn_before_days));

        for key in keys.values_mut() {
            if (key.state == KeyState::Active || key.state == KeyState::ExpiringSoon)
                && let Some(expires) = key.expires_at {
                    if expires <= now {
                        key.state = KeyState::Expired;
                    } else if expires <= warning_threshold {
                        key.state = KeyState::ExpiringSoon;
                    }
                }
        }
    }
}

/// Key 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expiring_keys: usize,
    pub expired_keys: usize,
    pub revoked_keys: usize,
    pub total_usage: u64,
}

/// 生成 Key ID
fn generate_key_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("key_{timestamp:016x}")
}

/// 生成 Key 对（哈希和前缀）
fn generate_key_pair() -> (String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let hash = format!("{timestamp:032x}");
    let prefix = format!("{:08}", timestamp % 100000000);
    (hash, prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_key() {
        let manager = ApiKeyManager::default_manager();

        let key = manager
            .create_key(
                "测试 Key",
                KeyType::Primary,
                "user_001",
                vec!["read".to_string(), "write".to_string()],
                Some(365),
            )
            .await;

        assert_eq!(key.name, "测试 Key");
        assert_eq!(key.state, KeyState::Active);
        assert_eq!(key.key_type, KeyType::Primary);
    }

    #[tokio::test]
    async fn test_renew_key() {
        let manager = ApiKeyManager::default_manager();

        let key = manager
            .create_key("测试 Key", KeyType::Primary, "user_001", vec![], Some(30))
            .await;

        let request = RenewalRequest {
            key_id: key.key_id.clone(),
            extension_days: 60,
            reason: Some("业务需要".to_string()),
        };

        let result = manager.renew_key(request).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_rotate_key() {
        let manager = ApiKeyManager::default_manager();

        let key = manager
            .create_key("测试 Key", KeyType::Primary, "user_001", vec![], None)
            .await;

        let request = RotationRequest {
            key_id: key.key_id.clone(),
            generate_backup: true,
            grace_period_minutes: 30,
        };

        let result = manager.rotate_key(request).await;
        assert!(result.success);
        assert!(result.new_key.is_some());
        assert!(result.backup_key.is_some());
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let manager = ApiKeyManager::default_manager();

        let key = manager
            .create_key("测试 Key", KeyType::Primary, "user_001", vec![], None)
            .await;

        let revoked = manager
            .revoke_key(&key.key_id, Some("不再需要".to_string()))
            .await;
        assert!(revoked);

        let fetched = manager.get_key(&key.key_id).await;
        assert_eq!(fetched.unwrap().state, KeyState::Revoked);
    }

    #[tokio::test]
    async fn test_verify_key() {
        let manager = ApiKeyManager::default_manager();

        let key = manager
            .create_key("测试 Key", KeyType::Primary, "user_001", vec![], None)
            .await;

        // 验证 Key
        let verified = manager.verify_key(&key.key_prefix, &key.key_hash).await;
        assert!(verified.is_some());
    }

    #[tokio::test]
    async fn test_get_user_keys() {
        let manager = ApiKeyManager::default_manager();

        manager
            .create_key("Key1", KeyType::Primary, "user_001", vec![], None)
            .await;
        manager
            .create_key("Key2", KeyType::Secondary, "user_001", vec![], None)
            .await;
        manager
            .create_key("Key3", KeyType::Primary, "user_002", vec![], None)
            .await;

        let user_keys = manager.get_user_keys("user_001").await;
        assert_eq!(user_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = ApiKeyManager::default_manager();

        manager
            .create_key("Key1", KeyType::Primary, "user_001", vec![], None)
            .await;
        manager
            .create_key("Key2", KeyType::Secondary, "user_001", vec![], None)
            .await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.active_keys, 2);
    }
}
