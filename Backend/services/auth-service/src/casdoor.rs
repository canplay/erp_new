//! Casdoor 集成模块
//!
//! 实现与 Casdoor 的 `OAuth2` 集成，提供 SSO 和用户管理功能
//!
//! @date 2026-05-17

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Casdoor 配置
#[derive(Debug, Clone)]
pub struct CasdoorConfig {
    /// Casdoor 服务器地址
    pub endpoint: String,
    /// 客户端 ID
    pub client_id: String,
    /// 客户端密钥
    pub client_secret: String,
    /// 组织名称
    pub organization: String,
    /// 应用程序名称
    pub application: String,
    /// 重定向 URI
    pub redirect_uri: String,
}

impl Default for CasdoorConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("CASDOOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            client_id: std::env::var("CASDOOR_CLIENT_ID")
                .unwrap_or_else(|_| "default-client-id".to_string()),
            client_secret: std::env::var("CASDOOR_CLIENT_SECRET")
                .expect("CASDOOR_CLIENT_SECRET must be set"),
            organization: std::env::var("CASDOOR_ORGANIZATION")
                .unwrap_or_else(|_| "built-in".to_string()),
            application: std::env::var("CASDOOR_APPLICATION")
                .unwrap_or_else(|_| "app-auth".to_string()),
            redirect_uri: std::env::var("CASDOOR_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8081/casdoor/callback".to_string()),
        }
    }
}

/// Casdoor 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasdoorUser {
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Casdoor Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasdoorTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
}

/// Casdoor 错误
#[derive(Debug, Clone)]
pub enum CasdoorError {
    /// 网络错误
    NetworkError(String),
    /// 认证失败
    AuthenticationFailed(String),
    /// 用户不存在
    UserNotFound,
    /// API 错误
    ApiError(String),
}

impl std::fmt::Display for CasdoorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "网络错误: {msg}"),
            Self::AuthenticationFailed(msg) => write!(f, "认证失败: {msg}"),
            Self::UserNotFound => write!(f, "用户不存在"),
            Self::ApiError(msg) => write!(f, "API 错误: {msg}"),
        }
    }
}

impl std::error::Error for CasdoorError {}

/// Casdoor 客户端
pub struct CasdoorClient {
    #[allow(dead_code)]
    config: CasdoorConfig,
    http_client: reqwest::Client,
}

impl CasdoorClient {
    /// 创建新的 Casdoor 客户端
    #[must_use]
    pub fn new(config: CasdoorConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    /// 获取授权 URL
    #[must_use]
    pub fn get_authorize_url(&self, state: &str) -> String {
        let params = [
            ("response_type", "code"),
            ("client_id", &self.config.client_id),
            ("redirect_uri", &self.config.redirect_uri),
            ("scope", "read"),
            ("state", state),
        ];

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!(
            "{}/login/oauth/authorize?{}",
            self.config.endpoint.trim_end_matches('/'),
            query_string
        )
    }

    /// 交换授权码获取 Token
    pub async fn exchange_code(&self, code: &str) -> Result<CasdoorTokenResponse, CasdoorError> {
        let url = format!(
            "{}/api/login/oauth/access_token",
            self.config.endpoint.trim_end_matches('/')
        );

        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        let response = self
            .http_client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| CasdoorError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(CasdoorError::AuthenticationFailed(text));
        }

        let token_response: CasdoorTokenResponse = response
            .json()
            .await
            .map_err(|e| CasdoorError::ApiError(e.to_string()))?;

        Ok(token_response)
    }

    /// 使用 Token 获取用户信息
    pub async fn get_user(&self, access_token: &str) -> Result<CasdoorUser, CasdoorError> {
        let url = format!(
            "{}/api/get-current-user",
            self.config.endpoint.trim_end_matches('/')
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|e| CasdoorError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(CasdoorError::ApiError(text));
        }

        let user: CasdoorUser = response
            .json()
            .await
            .map_err(|e| CasdoorError::ApiError(e.to_string()))?;

        Ok(user)
    }

    /// 刷新 Token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<CasdoorTokenResponse, CasdoorError> {
        let url = format!(
            "{}/api/login/oauth/refresh_token",
            self.config.endpoint.trim_end_matches('/')
        );

        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .http_client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| CasdoorError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(CasdoorError::AuthenticationFailed(text));
        }

        let token_response: CasdoorTokenResponse = response
            .json()
            .await
            .map_err(|e| CasdoorError::ApiError(e.to_string()))?;

        Ok(token_response)
    }

    /// 登出用户
    pub async fn logout(&self, access_token: &str) -> Result<(), CasdoorError> {
        let url = format!("{}/api/logout", self.config.endpoint.trim_end_matches('/'));

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|e| CasdoorError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(CasdoorError::ApiError(text));
        }

        Ok(())
    }

    /// 获取用户列表
    pub async fn get_users(&self, access_token: &str) -> Result<Vec<CasdoorUser>, CasdoorError> {
        let url = format!(
            "{}/api/get-users?p=1&pageSize=100",
            self.config.endpoint.trim_end_matches('/')
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|e| CasdoorError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(CasdoorError::ApiError(text));
        }

        #[derive(Deserialize)]
        struct UsersResponse {
            #[serde(rename = "data2")]
            data: Vec<CasdoorUser>,
        }

        let users_response: UsersResponse = response
            .json()
            .await
            .map_err(|e| CasdoorError::ApiError(e.to_string()))?;

        Ok(users_response.data)
    }
}

/// Casdoor 会话管理器
pub struct CasdoorSessionManager {
    /// 存储活跃会话
    sessions: Arc<RwLock<std::collections::HashMap<String, CasdoorSession>>>,
    /// 配置 (保留用于调试和扩展)
    #[allow(dead_code)]
    config: CasdoorConfig,
    /// 客户端
    client: CasdoorClient,
}

/// Casdoor 会话
#[derive(Debug, Clone)]
pub struct CasdoorSession {
    pub session_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user: CasdoorUser,
    pub created_at: std::time::Instant,
    pub expires_in: i64,
}

impl CasdoorSessionManager {
    /// 创建新的会话管理器
    #[must_use]
    pub fn new(config: CasdoorConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config: config.clone(),
            client: CasdoorClient::new(config),
        }
    }

    /// 创建授权 URL 并返回 state
    #[must_use]
    pub fn create_authorize_url(&self) -> (String, String) {
        let state = uuid::Uuid::new_v4().to_string();
        let url = self.client.get_authorize_url(&state);
        (url, state)
    }

    /// 处理授权回调
    pub async fn handle_callback(
        &self,
        code: &str,
        _state: &str,
    ) -> Result<CasdoorSession, CasdoorError> {
        // 交换授权码获取 Token
        let token_response = self.client.exchange_code(code).await?;

        // 获取用户信息
        let user = self.client.get_user(&token_response.access_token).await?;

        // 创建会话
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = CasdoorSession {
            session_id: session_id.clone(),
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            user,
            created_at: std::time::Instant::now(),
            expires_in: token_response.expires_in,
        };

        // 存储会话
        {
            let mut sessions = self.sessions.write();
            sessions.insert(session_id.clone(), session.clone());
        }

        tracing::info!(
            "创建 Casdoor 会话: session_id={}, user={}",
            session_id,
            session.user.name
        );

        Ok(session)
    }

    /// 获取会话
    #[must_use]
    pub fn get_session(&self, session_id: &str) -> Option<CasdoorSession> {
        let sessions = self.sessions.read();
        sessions.get(session_id).cloned()
    }

    /// 删除会话
    #[must_use]
    pub fn remove_session(&self, session_id: &str) -> Option<CasdoorSession> {
        let mut sessions = self.sessions.write();
        sessions.remove(session_id)
    }

    /// 清理过期会话
    pub fn cleanup_expired(&self) {
        let now = std::time::Instant::now();
        let mut sessions = self.sessions.write();
        sessions.retain(|_, session| {
            let elapsed = now.duration_since(session.created_at).as_secs() as i64;
            elapsed < session.expires_in
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casdoor_config_default() {
        unsafe { std::env::set_var("CASDOOR_CLIENT_SECRET", "test-secret-for-unit-test"); }
        let config = CasdoorConfig::default();
        assert_eq!(config.endpoint, "http://localhost:8000");
        assert_eq!(config.organization, "built-in");
        assert_eq!(config.client_secret, "test-secret-for-unit-test");
    }

    #[test]
    fn test_create_authorize_url() {
        unsafe { std::env::set_var("CASDOOR_CLIENT_SECRET", "test-secret-for-unit-test"); }
        let client = CasdoorClient::new(CasdoorConfig::default());
        let url = client.get_authorize_url("test_state");
        assert!(url.contains("authorize"));
        assert!(url.contains("client_id"));
    }
}
