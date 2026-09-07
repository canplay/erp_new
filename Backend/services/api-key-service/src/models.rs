//! API密钥服务数据模型
//!
//! 使用 repository.rs 中定义的 `ApiKey` 类型，保持数据一致性

pub use crate::repository::{ApiKey, ApiKeyUsageLog};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 保留 KeyStatus 枚举用于内部逻辑
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyStatus {
    #[default]
    Active,
    Inactive,
    Expired,
    Revoked,
}

impl std::fmt::Display for KeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyStatus::Active => write!(f, "active"),
            KeyStatus::Inactive => write!(f, "inactive"),
            KeyStatus::Expired => write!(f, "expired"),
            KeyStatus::Revoked => write!(f, "revoked"),
        }
    }
}

/// 权限级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PermissionLevel {
    #[default]
    ReadOnly,
    ReadWrite,
    Admin,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::ReadOnly => write!(f, "read_only"),
            PermissionLevel::ReadWrite => write!(f, "read_write"),
            PermissionLevel::Admin => write!(f, "admin"),
        }
    }
}

// 为 ApiKey 添加辅助方法
impl ApiKey {
    /// 生成新的API密钥
    /// 返回 (`ApiKey`, `key_id`, `key_secret`) 三元组
    #[must_use]
    pub fn generate(
        name: String,
        description: Option<String>,
        permission_level: i32,
        user_id: i64,
    ) -> (Self, String, String) {
        // 生成随机密钥
        let key_id = format!("ak_{}", &Uuid::new_v4().to_string().replace('-', "")[..24]);
        // 使用 rand::random 生成 32 字节的随机数据
        let key_secret: String = (0..32)
            .map(|_| {
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let idx = (rand::random::<u8>() as usize) % 62;
                chars.chars().nth(idx).unwrap()
            })
            .collect();

        let key = format!("{key_id}_{key_secret}");
        let key_hash = Self::hash_key(&key);
        let key_hint = key_secret[key_secret.len().saturating_sub(4)..].to_string();

        let now = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap();
        let api_key = Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            key_id: key_id.clone(),
            secret_key_hash: key_hash,
            key_hint,
            permission_level,
            allowed_ips: Vec::new(),
            rate_limit: 1000,
            tenant_id: None,
            user_id: Some(user_id),
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
            expires_at: None,
            last_used_at: None,
        };

        (api_key, key_id, key_secret)
    }

    /// 哈希密钥 (SHA-256)
    ///
    /// 审计修复 (2026-08-04): 原实现为明文存储 + std DefaultHasher(非密码学安全)。
    /// API key 本身为高熵 UUID, 无需加盐; SHA-256 单向哈希后 DB 泄露不还原密钥。
    pub(crate) fn hash_key(key: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// 验证密钥
    #[must_use]
    pub fn verify(&self, key: &str) -> bool {
        self.secret_key_hash == Self::hash_key(key)
    }

    /// 是否过期
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap()
        } else {
            false
        }
    }
}

/// 使用记录 - 数据库模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsageLog {
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
    pub created_at: DateTime<Utc>,
}

impl KeyUsageLog {
    #[must_use]
    pub fn new(key_id: String, api_path: String, method: String, status_code: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            key_id,
            api_path,
            method,
            status_code,
            request_size: 0,
            response_size: 0,
            response_time_ms: 0,
            ip_address: None,
            user_agent: None,
            error_message: None,
            created_at: DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap(),
        }
    }
}

/// 创建密钥请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission_level: i32, // 使用 i32 替代枚举
    pub allowed_ips: Option<Vec<String>>,
    pub rate_limit: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<i64>,
}

/// 更新密钥请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_level: Option<i32>, // 使用 i32 替代枚举
    pub allowed_ips: Option<Vec<String>>,
    pub rate_limit: Option<i64>,
    pub status: Option<String>, // 使用 String 替代枚举
    pub expires_at: Option<DateTime<Utc>>,
}

/// 密钥查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>, // 使用 String
    pub keyword: Option<String>,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
}

/// 验证密钥请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateKeyRequest {
    pub key: String,
    pub ip_address: Option<String>,
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResult<T> {
    pub records: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 密钥验证响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateKeyResponse {
    pub valid: bool,
    pub key_id: Option<String>,
    pub permission_level: Option<PermissionLevel>,
    pub error: Option<String>,
}

/// 密钥统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStats {
    pub total_keys: i64,
    pub active_keys: i64,
    pub expired_keys: i64,
    pub revoked_keys: i64,
    pub total_requests: i64,
    pub failed_requests: i64,
}
