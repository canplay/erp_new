//! JWT 认证中间件
//!
//! 提供 Axum 路由层的 JWT 认证支持
//! 用于从 HTTP 请求中提取和验证 JWT token

use axum::{
    Json,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// JWT Claims 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: i64,
    /// 用户名
    pub username: String,
    /// 角色
    pub role: String,
    /// 过期时间
    pub exp: usize,
    /// 签发时间
    pub iat: usize,
}

/// 认证用户上下文
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

impl AuthUser {
    /// 从 Claims 创建 `AuthUser`
    #[must_use]
    pub fn from_claims(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            username: claims.username,
            role: claims.role,
        }
    }

    /// 检查是否是管理员
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

/// JWT 中间件配置
#[derive(Clone, Default)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: Option<String>,
}

impl JwtConfig {
    /// 从环境变量创建配置
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET 环境变量未设置，请在 .env 或环境中配置"),
            issuer: Some("myai".to_string()), // 审计修复 B7: 统一默认值
        }
    }
}

/// JWT 验证器
#[derive(Clone)]
pub struct JwtValidator {
    secret: String,
}

impl JwtValidator {
    /// 创建新的验证器
    #[must_use]
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            secret: config.secret.clone(),
        }
    }

    /// 验证 token 并返回 Claims
    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    /// 从 Authorization header 提取 token
    #[must_use]
    pub fn extract_token(auth_header: &str) -> Option<&str> {
        auth_header.strip_prefix("Bearer ")
    }
}

/// 从请求中提取认证用户的提取器
#[derive(Debug)]
pub struct AuthenticatedUser(pub AuthUser);

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidFormat,
    InvalidToken(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::MissingHeader => (
                StatusCode::UNAUTHORIZED,
                "缺少 Authorization header".to_string(),
            ),
            Self::InvalidFormat => (
                StatusCode::UNAUTHORIZED,
                "Authorization 格式错误，期望: Bearer <token>".to_string(),
            ),
            Self::InvalidToken(msg) => {
                (StatusCode::UNAUTHORIZED, format!("Token 无效: {msg}"))
            }
        };

        (
            status,
            Json(json!({
                "success": false,
                "error": message
            })),
        )
            .into_response()
    }
}

// 实现 FromRequestParts 用于 axum 0.8+
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + Clone + 'static,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从 Authorization header 获取 token
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingHeader)?;

        // 提取 Bearer token
        let token = JwtValidator::extract_token(auth_header).ok_or(AuthError::InvalidFormat)?;

        // 创建验证器
        let config = JwtConfig::from_env();
        let validator = JwtValidator::new(&config);

        // 验证 token
        let claims = validator
            .verify(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(Self(AuthUser::from_claims(claims)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use jsonwebtoken::{EncodingKey, Header, encode};

    fn create_test_token(user_id: i64, username: &str, role: &str) -> String {
        let config = JwtConfig {
            secret: "your-secret-key-change-in-production".to_string(),
            issuer: Some("myai".to_string()), // 审计修复 B7: 统一默认值
        };

        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .unwrap()
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            role: role.to_string(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_verify_valid_token() {
        let token = create_test_token(1, "testuser", "admin");
        // 使用与 create_test_token 相同的 secret
        let config = JwtConfig {
            secret: "your-secret-key-change-in-production".to_string(),
            issuer: Some("myai".to_string()), // 审计修复 B7: 统一默认值
        };
        let validator = JwtValidator::new(&config);

        let claims = validator.verify(&token).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_extract_token() {
        let header = "Bearer abc123";
        let token = JwtValidator::extract_token(header);
        assert_eq!(token, Some("abc123"));

        let invalid_header = "Basic abc123";
        let token = JwtValidator::extract_token(invalid_header);
        assert_eq!(token, None);
    }

    #[test]
    fn test_auth_user_from_claims() {
        let claims = Claims {
            sub: 42,
            username: "testuser".to_string(),
            role: "admin".to_string(),
            exp: 0,
            iat: 0,
        };

        let user = AuthUser::from_claims(claims);
        assert_eq!(user.user_id, 42);
        assert_eq!(user.username, "testuser");
        assert!(user.is_admin());
    }
}
