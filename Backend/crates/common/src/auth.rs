//! 认证相关工具
//!
//! 提供 JWT 编码/解码、请求头提取用户信息的辅助函数。
//! JWT 验证包含 issuer/audience 校验。

use axum::http::Request;
use serde::{Deserialize, Serialize};

/// JWT Claims（用于编解码）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// 用户 ID
    pub sub: i64,
    /// 用户名
    pub username: String,
    /// 角色
    pub role: String,
    /// 签发者
    pub iss: Option<String>,
    /// 受众
    pub aud: Option<String>,
    /// 过期时间 (Unix timestamp)
    pub exp: usize,
    /// 签发时间 (Unix timestamp)
    pub iat: usize,
}

/// JWT 验证器
pub struct JwtValidator {
    secret: String,
    issuer: Option<String>,
    audience: Option<String>,
}

impl JwtValidator {
    /// 创建新的 JWT 验证器
    #[must_use]
    pub const fn new(secret: String, issuer: Option<String>, audience: Option<String>) -> Self {
        Self {
            secret,
            issuer,
            audience,
        }
    }

    /// 验证 JWT 令牌，返回 Claims
    ///
    /// 验证过程包含：
    /// - 签名验证（HMAC-SHA256）
    /// - 过期时间验证（exp）
    /// - issuer 验证（如果配置）
    /// - audience 验证（如果配置）
    pub fn verify(&self, token: &str) -> Result<JwtClaims, String> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        if let Some(ref issuer) = self.issuer {
            validation.set_issuer(&[issuer.as_str()]);
        }

        if let Some(ref audience) = self.audience {
            validation.set_audience(&[audience.as_str()]);
        }

        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|t| t.claims)
        .map_err(|e| format!("JWT 验证失败: {e}" ))
    }

    /// 生成 JWT 令牌
    ///
    /// 生成的令牌包含 issuer/audience（如果配置），
    /// 接收方可以使用 `verify` 方法进行完整验证。
    #[must_use]
    pub fn generate(&self, user_id: i64, username: &str, role: &str) -> Result<String, String> {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("时间错误: {e}" ))?
            .as_secs() as usize;

        let claims = JwtClaims {
            sub: user_id,
            username: username.to_string(),
            role: role.to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            exp: now + 3600, // 默认 1 小时过期
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| format!("JWT 编码失败: {e}" ))
    }

    /// 从 Authorization header 提取 Bearer token
    #[must_use]
    pub fn extract_token(auth_header: &str) -> Option<&str> {
        auth_header.strip_prefix("Bearer " )
    }
}

/// 用户上下文信息（从请求头提取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

impl UserContext {
    /// 从请求中提取用户上下文
    pub fn from_request<B>(request: &Request<B>) -> Option<Self> {
        let headers = request.headers();

        let user_id = headers
            .get("x-user-id" )
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())?;

        let username = headers
            .get("x-user-name" )
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string)?;

        let role = headers
            .get("x-user-role" )
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string)?;

        Some(Self {
            user_id,
            username,
            role,
        })
    }

    /// 检查是否是管理员
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

/// 请求头常量
pub const HEADER_USER_ID: &str = "x-user-id";
pub const HEADER_USER_NAME: &str = "x-user-name";
pub const HEADER_USER_ROLE: &str = "x-user-role";
pub const HEADER_USER_TOKEN: &str = "x-user-token";

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    #[test]
    fn test_from_request() {
        let mut request = Request::builder();
        request = request.header("x-user-id" , "123" );
        request = request.header("x-user-name" , "testuser" );
        request = request.header("x-user-role" , "admin" );

        let request = request.body(Body::empty()).unwrap();
        let ctx = UserContext::from_request(&request).unwrap();

        assert_eq!(ctx.user_id, 123);
        assert_eq!(ctx.username, "testuser" );
        assert_eq!(ctx.role, "admin" );
        assert!(ctx.is_admin());
    }

    #[test]
    fn test_from_request_missing_headers() {
        let request = Request::builder().body(Body::empty()).unwrap();
        let ctx = UserContext::from_request(&request);

        assert!(ctx.is_none());
    }

    #[test]
    fn test_jwt_verify_valid_token() {
        let validator = JwtValidator::new(
            "test-secret-key-at-least-32-characters-long".to_string(),
            Some("my-issuer".to_string()),
            Some("my-audience".to_string()),
        );

        let token = validator
            .generate(123, "testuser" , "admin" )
            .unwrap();

        let claims = validator.verify(&token).unwrap();
        assert_eq!(claims.sub, 123);
        assert_eq!(claims.username, "testuser" );
        assert_eq!(claims.role, "admin" );
        assert_eq!(claims.iss, Some("my-issuer".to_string()));
        assert_eq!(claims.aud, Some("my-audience".to_string()));
    }

    #[test]
    fn test_jwt_verify_wrong_issuer() {
        let validator = JwtValidator::new(
            "test-secret-key-at-least-32-characters-long".to_string(),
            Some("my-issuer".to_string()),
            Some("my-audience".to_string()),
        );

        let token = validator
            .generate(123, "testuser" , "admin" )
            .unwrap();

        let wrong_validator = JwtValidator::new(
            "test-secret-key-at-least-32-characters-long".to_string(),
            Some("wrong-issuer".to_string()),
            Some("my-audience".to_string()),
        );

        assert!(wrong_validator.verify(&token).is_err());
    }

    #[test]
    fn test_jwt_verify_wrong_audience() {
        let validator = JwtValidator::new(
            "test-secret-key-at-least-32-characters-long".to_string(),
            Some("my-issuer".to_string()),
            Some("my-audience".to_string()),
        );

        let token = validator
            .generate(123, "testuser" , "admin" )
            .unwrap();

        let wrong_validator = JwtValidator::new(
            "test-secret-key-at-least-32-characters-long".to_string(),
            Some("my-issuer".to_string()),
            Some("wrong-audience".to_string()),
        );

        assert!(wrong_validator.verify(&token).is_err());
    }

    #[test]
    fn test_jwt_verify_no_issuer_audience() {
        let validator = JwtValidator::new(
            "test-secret-key-at-least-32-characters-long".to_string(),
            None,
            None,
        );

        let token = validator
            .generate(123, "testuser" , "admin" )
            .unwrap();

        let claims = validator.verify(&token).unwrap();
        assert_eq!(claims.sub, 123);
        assert_eq!(claims.username, "testuser" );
        assert_eq!(claims.iss, None);
        assert_eq!(claims.aud, None);
    }

    #[test]
    fn test_jwt_extract_token() {
        assert_eq!(
            JwtValidator::extract_token("Bearer abc123" ),
            Some("abc123" )
        );
        assert_eq!(
            JwtValidator::extract_token("Basic abc123" ),
            None
        );
    }
}
