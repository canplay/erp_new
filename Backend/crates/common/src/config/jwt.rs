//! JWT 配置

use config::ConfigError;
use serde::Deserialize;

/// JWT 配置
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-test-secret-do-not-use-in-production".to_string()),
            issuer: "backend-rust".to_string(),
            audience: "backend-rust".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        }
    }
}

impl JwtConfig {
    /// 验证 JWT 配置是否有效
    pub fn validate(&self) -> Result<(), ConfigError> {
        // JWT secret 长度必须 >= 32 字符，否则存在安全风险
        if self.secret.len() < 32 {
            return Err(ConfigError::Message(format!(
                "JWT secret 长度必须 >= 32 字符（当前: {}），请设置安全的密钥",
                self.secret.len()
            )));
        }
        // 检查是否为默认密钥（仅在环境变量 JWT_REQUIRE_STRONG_SECRET=true 时检查）
        if self.secret == "default-test-secret-do-not-use-in-production" {
            let require_strong = std::env::var("JWT_REQUIRE_STRONG_SECRET")
                .unwrap_or_else(|_| "false".to_string())
                .to_lowercase() == "true";
            if require_strong {
                return Err(ConfigError::Message(
                    "禁止使用默认 JWT secret，请在生产环境中设置安全的密钥".to_string(),
                ));
            }
        }
        if self.access_token_expiry_secs == 0 {
            return Err(ConfigError::Message("访问令牌过期时间不能为 0".to_string()));
        }
        if self.refresh_token_expiry_secs <= self.access_token_expiry_secs {
            return Err(ConfigError::Message(
                "刷新令牌过期时间必须大于访问令牌过期时间".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_config_validate() {
        let mut config = JwtConfig::default();
        // 默认配置使用默认 secret，不设置 JWT_REQUIRE_STRONG_SECRET 时验证通过
        assert!(config.validate().is_ok());

        // 使用安全的 secret
        config.secret = "test-secret-key-for-testing-only-not-for-production-use-32".to_string();
        assert!(config.validate().is_ok());

        // secret 太短
        config.secret = "short".to_string();
        assert!(config.validate().is_err());

        // access_token_expiry_secs 为 0
        config.secret = "test-secret-key-for-testing-only-not-for-production-use-32".to_string();
        config.access_token_expiry_secs = 0;
        assert!(config.validate().is_err());

        // refresh_token_expiry_secs <= access_token_expiry_secs
        config.access_token_expiry_secs = 3600;
        config.refresh_token_expiry_secs = 3600;
        assert!(config.validate().is_err());
    }
}
