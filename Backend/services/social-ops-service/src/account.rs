//! 社交账号与凭证数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 社交账号
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SocialAccount {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub platform: String,
    pub account_name: String,
    pub account_id: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub config_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 平台凭证（加密存储）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlatformCredential {
    pub id: Uuid,
    pub account_id: Uuid,
    pub credential_type: String,
    pub encrypted_data: String,  // AES-256-GCM 加密
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 创建账号请求
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub user_id: Option<Uuid>,
    pub platform: String,
    pub account_name: String,
    pub credentials: Option<String>,  // 明文凭证，入库前加密
    pub config: Option<serde_json::Value>,
}

/// 更新账号请求
#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub account_name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}
