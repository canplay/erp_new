//! JWT 服务

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// JWT Claims 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // 用户ID
    pub username: String,
    pub role: String,
    pub tenant_id: Option<i64>, // 租户ID（可选，用于多租户场景）
    pub iss: String,            // 发行者
    pub aud: String,            // 受众
    pub exp: i64,               // 过期时间
    pub iat: i64,               // 签发时间
}

/// JWT 服务
#[derive(Clone)]
pub struct JwtService {
    secret: String,
    issuer: String,
    audience: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl JwtService {
    #[must_use]
    pub fn new(
        secret: &str,
        issuer: &str,
        audience: &str,
        access_token_expiry_secs: u64,
        refresh_token_expiry_secs: u64,
    ) -> Self {
        Self {
            secret: secret.to_string(),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            access_token_expiry: Duration::seconds(access_token_expiry_secs as i64),
            refresh_token_expiry: Duration::seconds(refresh_token_expiry_secs as i64),
        }
    }

    /// 生成访问令牌（支持多租户）
    pub fn generate_access_token_with_tenant(
        &self,
        user_id: i64,
        username: &str,
        role: &str,
        tenant_id: Option<i64>,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            role: role.to_string(),
            tenant_id,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            exp: (now + self.access_token_expiry).timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    /// 生成访问令牌（兼容旧版本）
    pub fn generate_access_token(
        &self,
        user_id: i64,
        username: &str,
        role: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        self.generate_access_token_with_tenant(user_id, username, role, None)
    }

    /// 生成刷新令牌
    pub fn generate_refresh_token(
        &self,
        user_id: i64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            username: "refresh".to_string(),
            role: "refresh".to_string(),
            tenant_id: None,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            exp: (now + self.refresh_token_expiry).timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    /// 验证令牌
    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);
        let key = DecodingKey::from_secret(self.secret.as_bytes());

        let token_data = decode::<Claims>(token, &key, &validation)?;
        Ok(token_data.claims)
    }

    /// 获取租户 ID
    #[must_use]
    pub fn get_tenant_id(&self, token: &str) -> Option<i64> {
        self.verify_token(token)
            .ok()
            .and_then(|claims| claims.tenant_id)
    }

    /// 验证刷新令牌
    pub fn verify_refresh_token(&self, token: &str) -> Result<i64, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);
        let key = DecodingKey::from_secret(self.secret.as_bytes());

        let token_data = decode::<Claims>(token, &key, &validation)?;

        // 检查是否是刷新令牌
        if token_data.claims.role != "refresh" {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        Ok(token_data.claims.sub)
    }

    /// 从 token 获取用户信息
    #[must_use]
    pub fn get_user_from_token(&self, token: &str) -> Option<(i64, String, String)> {
        self.verify_token(token)
            .ok()
            .map(|claims| (claims.sub, claims.username, claims.role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_round_trip() {
        let service = JwtService::new("test-secret", "test-issuer", "test-audience", 3600, 604800);

        let token = service
            .generate_access_token(123, "testuser", "admin")
            .unwrap();
        let claims = service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, 123);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "admin");
    }
}
