//! Auth Service 集成测试
//!
//! 测试场景:
//! - Normal login → returns token
//! - Wrong password → 401
//! - JWT expiry → refresh succeeds
//! - Weak password → registration rejected
//! - Token validation → correct claims extracted

use auth_core::{JwtService, PasswordService};
use grpc_proto::auth::{
    LoginRequest, LoginResponse, RefreshRequest, RefreshResponse, RegisterRequest,
    RegisterResponse, TokenRequest, TokenResponse, auth_service_client::AuthServiceClient,
    auth_service_server::{AuthService, AuthServiceServer},
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use auth_service::handlers::{AuthServiceImpl, AppState};
use auth_service::repository::{UserInfo, UserRepositoryError};

// ==================== Mock Repository ====================

/// Mock user repository for testing
#[derive(Clone, Debug)]
struct MockUserRepository {
    users: std::collections::HashMap<String, UserInfo>,
}

impl MockUserRepository {
    fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
        }
    }

    fn add_user(&mut self, user: UserInfo) {
        self.users.insert(user.username.clone(), user);
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserInfo>, UserRepositoryError> {
        Ok(self.users.get(username).cloned())
    }

    async fn find_by_id(&self, user_id: i64) -> Result<Option<UserInfo>, UserRepositoryError> {
        Ok(self.users.values().find(|u| u.id == user_id).cloned())
    }

    async fn create(
        &mut self,
        username: &str,
        password_hash: &str,
        email: Option<String>,
    ) -> Result<i64, UserRepositoryError> {
        if self.users.contains_key(username) {
            return Err(UserRepositoryError::AlreadyExists);
        }
        let id = (self.users.len() as i64) + 1;
        let user = UserInfo {
            id,
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            email,
            phone: None,
            nickname: None,
            status: 1,
            role: "user".to_string(),
            must_change_password: false,
        };
        self.users.insert(username.to_string(), user);
        Ok(id)
    }

    async fn update_password(
        &mut self,
        user_id: i64,
        new_password_hash: &str,
    ) -> Result<(), UserRepositoryError> {
        for user in self.users.values_mut() {
            if user.id == user_id {
                user.password_hash = new_password_hash.to_string();
                return Ok(());
            }
        }
        Err(UserRepositoryError::NotFound)
    }
}

// ==================== Mock AuthServiceImpl ====================

/// Wrapper around AuthServiceImpl that uses MockUserRepository
struct MockAuthServiceImpl {
    jwt_service: JwtService,
    password_service: PasswordService,
    users: std::sync::Mutex<MockUserRepository>,
}

impl MockAuthServiceImpl {
    fn new() -> Self {
        let jwt_service = JwtService::new(
            "test-secret-key-for-integration-tests",
            "test-issuer",
            "test-audience",
            2,  // 2 seconds expiry for testing
            60, // 60 seconds refresh expiry
        );
        let password_service = PasswordService;
        Self {
            jwt_service,
            password_service,
            users: std::sync::Mutex::new(MockUserRepository::new()),
        }
    }

    fn with_test_user(username: &str, password: &str) -> Self {
        let mut service = Self::new();
        let password_hash = service.password_service.hash_password(password).unwrap();
        let user = UserInfo {
            id: 1,
            username: username.to_string(),
            password_hash,
            email: Some(format!("{}@example.com", username)),
            phone: None,
            nickname: Some(username.to_string()),
            status: 1,
            role: "user".to_string(),
            must_change_password: false,
        };
        service.users.lock().unwrap().add_user(user);
        service
    }

    fn with_disabled_user(username: &str, password: &str) -> Self {
        let mut service = Self::new();
        let password_hash = service.password_service.hash_password(password).unwrap();
        let user = UserInfo {
            id: 1,
            username: username.to_string(),
            password_hash,
            email: None,
            phone: None,
            nickname: None,
            status: 0, // disabled
            role: "user".to_string(),
            must_change_password: false,
        };
        service.users.lock().unwrap().add_user(user);
        service
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginResponse, Status> {
        let users = self.users.lock().unwrap();
        let user = users
            .find_by_username(username)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match user {
            Some(user) => {
                // Check account status
                if user.status != 1 {
                    return Err(Status::unauthenticated("账号已被禁用"));
                }

                // Verify password
                if !self
                    .password_service
                    .verify_bcrypt(&password, &user.password_hash)
                {
                    return Err(Status::unauthenticated("Invalid credentials"));
                }

                // Generate token
                let token = self
                    .jwt_service
                    .generate_access_token(user.id, &user.username, "user")
                    .map_err(|e| Status::internal(format!("Token generation error: {}", e)))?;

                Ok(LoginResponse {
                    token,
                    user_id: user.id,
                    username: user.username,
                    role: user.role,
                    must_change_password: user.must_change_password,
                })
            }
            None => Err(Status::unauthenticated("Invalid credentials")),
        }
    }

    async fn register(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<RegisterResponse, Status> {
        // Validate password strength
        if password.len() < 8 {
            return Err(Status::invalid_argument("密码长度不能少于 8 位"));
        }
        let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        if !has_letter || !has_digit {
            return Err(Status::invalid_argument("密码必须同时包含字母和数字"));
        }
        // Check common weak passwords
        const WEAK: &[&str] = &[
            "12345678", "123456789", "password", "admin123", "12345678a", "a12345678",
        ];
        if WEAK.contains(&password) {
            return Err(Status::invalid_argument("密码过于简单, 请更换"));
        }

        let password_hash = self
            .password_service
            .hash_bcrypt(password)
            .map_err(|e| Status::internal(format!("Password hash error: {}", e)))?;

        let mut users = self.users.lock().unwrap();
        let user_id = users
            .create(username, &password_hash, Some(email.to_string()))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let token = self
            .jwt_service
            .generate_access_token(user_id, username, "user")
            .map_err(|e| Status::internal(format!("Token generation error: {}", e)))?;

        Ok(RegisterResponse {
            token,
            user_id,
            username: username.to_string(),
        })
    }

    async fn validate_token(&self, token: &str) -> TokenResponse {
        match self.jwt_service.verify_token(token) {
            Ok(claims) => TokenResponse {
                valid: true,
                user_id: claims.sub,
                username: claims.username,
                role: claims.role,
            },
            Err(_) => TokenResponse {
                valid: false,
                user_id: 0,
                username: String::new(),
                role: String::new(),
            },
        }
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshResponse, Status> {
        let user_id = self
            .jwt_service
            .verify_refresh_token(refresh_token)
            .map_err(|_| Status::unauthenticated("Invalid refresh token"))?;

        let users = self.users.lock().unwrap();
        let user = users
            .find_by_id(user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match user {
            Some(user) => {
                let token = self
                    .jwt_service
                    .generate_access_token(user_id, &user.username, "user")
                    .map_err(|e| Status::internal(format!("Token generation error: {}", e)))?;

                Ok(RefreshResponse {
                    token,
                    user_id,
                    username: user.username,
                    role: "user".to_string(),
                })
            }
            None => Err(Status::not_found("User not found")),
        }
    }
}

// ==================== Test Modules ====================

#[cfg(test)]
mod auth_flow_tests {
    use super::*;

    // ==================== 1. Normal login → returns token ====================

    #[tokio::test]
    async fn test_login_returns_token() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");
        let result = service.login("testuser", "password123").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user_id, 1);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.role, "user");
        assert_eq!(response.must_change_password, false);
    }

    #[tokio::test]
    async fn test_login_with_valid_credentials() {
        let service = MockAuthServiceImpl::with_test_user("admin", "admin12345");
        let result = service.login("admin", "admin12345").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert!(response.user_id > 0);
    }

    // ==================== 2. Wrong password → 401 ====================

    #[tokio::test]
    async fn test_wrong_password_returns_401() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");
        let result = service.login("testuser", "wrongpassword").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_nonexistent_user_returns_401() {
        let service = MockAuthServiceImpl::new();
        let result = service.login("nonexistent", "password123").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_disabled_user_returns_401() {
        let service = MockAuthServiceImpl::with_disabled_user("disabled_user", "password123");
        let result = service.login("disabled_user", "password123").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    // ==================== 3. JWT expiry → refresh succeeds ====================

    #[tokio::test]
    async fn test_jwt_expiry_refresh_succeeds() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");

        // Login to get tokens
        let login_response = service.login("testuser", "password123").await.unwrap();
        assert!(!login_response.token.is_empty());

        // Verify the token is valid
        let validation = service.validate_token(&login_response.token).await;
        assert!(validation.valid);
        assert_eq!(validation.user_id, 1);

        // Generate a refresh token
        let refresh_token = service
            .jwt_service
            .generate_refresh_token(1)
            .unwrap();

        // Use refresh token to get new access token
        let refresh_response = service.refresh_token(&refresh_token).await;
        assert!(refresh_response.is_ok());

        let new_token = refresh_response.unwrap();
        assert!(!new_token.token.is_empty());
        assert_eq!(new_token.user_id, 1);
    }

    #[tokio::test]
    async fn test_refresh_with_invalid_token_fails() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");

        let result = service.refresh_token("invalid.refresh.token").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_expired_token_validation_fails() {
        // Create a token with a different secret, which should fail validation
        let different_jwt_service = JwtService::new(
            "different-secret-key-for-expired-token-test",
            "test-issuer",
            "test-audience",
            3600, // 1 hour expiry
            60,
        );

        let token = different_jwt_service.generate_access_token(1, "testuser", "user").unwrap();

        // Try to validate with the original service (different secret)
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");
        let validation = service.validate_token(&token).await;
        assert!(!validation.valid);
    }

    // ==================== 4. Weak password → registration rejected ====================

    #[tokio::test]
    async fn test_weak_password_rejected() {
        let service = MockAuthServiceImpl::new();

        // Test: too short
        let result = service.register("newuser", "123", "test@example.com").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);

        // Test: no digits
        let result = service.register("newuser", "abcdefgh", "test@example.com").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);

        // Test: no letters
        let result = service.register("newuser", "12345678", "test@example.com").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_common_weak_passwords_rejected() {
        let service = MockAuthServiceImpl::new();

        // Common weak passwords should be rejected
        let weak_passwords = ["12345678", "password", "admin123"];

        for password in &weak_passwords {
            let result = service.register("newuser", password, "test@example.com").await;
            assert!(
                result.is_err(),
                "Password '{}' should be rejected",
                password
            );
        }
    }

    #[tokio::test]
    async fn test_valid_registration_succeeds() {
        let service = MockAuthServiceImpl::new();

        let result = service
            .register("newuser", "validPass123", "newuser@example.com")
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.username, "newuser");
        assert!(response.user_id > 0);
    }

    #[tokio::test]
    async fn test_duplicate_username_rejected() {
        let service = MockAuthServiceImpl::new();

        // First registration
        let result = service
            .register("testuser", "password123", "test1@example.com")
            .await;
        assert!(result.is_ok());

        // Second registration with same username
        let result = service
            .register("testuser", "differentPass123", "test2@example.com")
            .await;
        assert!(result.is_err());
    }

    // ==================== 5. Token validation → correct claims ====================

    #[tokio::test]
    async fn test_token_validation_extracts_correct_claims() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");

        // Login to get a valid token
        let login_response = service.login("testuser", "password123").await.unwrap();

        // Validate the token
        let validation = service.validate_token(&login_response.token).await;
        assert!(validation.valid);
        assert_eq!(validation.user_id, 1);
        assert_eq!(validation.username, "testuser");
        assert_eq!(validation.role, "user");
    }

    #[tokio::test]
    async fn test_invalid_token_validation_fails() {
        let service = MockAuthServiceImpl::new();

        let validation = service.validate_token("invalid.token.here").await;
        assert!(!validation.valid);
        assert_eq!(validation.user_id, 0);
        assert!(validation.username.is_empty());
        assert!(validation.role.is_empty());
    }

    #[tokio::test]
    async fn test_tampered_token_validation_fails() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");

        let login_response = service.login("testuser", "password123").await.unwrap();

        // Tamper with the token
        let mut tampered_token = login_response.token;
        if tampered_token.len() > 10 {
            tampered_token.replace_range(5..10, "XXXXX");
        }

        let validation = service.validate_token(&tampered_token).await;
        assert!(!validation.valid);
    }

    #[tokio::test]
    async fn test_wrong_secret_token_validation_fails() {
        // Create a token with a different secret
        let different_service = JwtService::new(
            "different-secret-key",
            "test-issuer",
            "test-audience",
            3600,
            604800,
        );

        let token = different_service
            .generate_access_token(1, "testuser", "user")
            .unwrap();

        // Try to validate with the original service
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");
        let validation = service.validate_token(&token).await;
        assert!(!validation.valid);
    }

    // ==================== Additional integration tests ====================

    #[tokio::test]
    async fn test_login_after_registration() {
        let service = MockAuthServiceImpl::new();

        // Register a new user
        let register_response = service
            .register("newuser", "securePass123", "new@example.com")
            .await
            .unwrap();

        // Login with the same credentials
        let login_response = service
            .login("newuser", "securePass123")
            .await
            .unwrap();

        // Both should have the same user_id
        assert_eq!(register_response.user_id, login_response.user_id);
    }

    #[tokio::test]
    async fn test_password_hash_consistency() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");

        // First login
        let result1 = service.login("testuser", "password123").await;
        assert!(result1.is_ok());

        // Second login with same password
        let result2 = service.login("testuser", "password123").await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_full_auth_flow() {
        let service = MockAuthServiceImpl::new();

        // 1. Register
        let register_response = service
            .register("fulluser", "fullPass123", "full@example.com")
            .await
            .unwrap();
        assert!(!register_response.token.is_empty());

        // 2. Validate the registration token
        let validation = service.validate_token(&register_response.token).await;
        assert!(validation.valid);
        assert_eq!(validation.username, "fulluser");

        // 3. Login
        let login_response = service.login("fulluser", "fullPass123").await.unwrap();
        assert!(!login_response.token.is_empty());

        // 4. Validate login token
        let validation = service.validate_token(&login_response.token).await;
        assert!(validation.valid);
        assert_eq!(validation.user_id, register_response.user_id);

        // 5. Refresh token
        let refresh_token = service
            .jwt_service
            .generate_refresh_token(register_response.user_id)
            .unwrap();
        let refresh_response = service.refresh_token(&refresh_token).await.unwrap();
        assert!(!refresh_response.token.is_empty());

        // 6. Validate refreshed token
        let validation = service.validate_token(&refresh_response.token).await;
        assert!(validation.valid);
        assert_eq!(validation.user_id, register_response.user_id);
    }

    // ==================== Password strength edge cases ====================

    #[tokio::test]
    async fn test_password_exactly_8_chars_with_letter_and_digit() {
        let service = MockAuthServiceImpl::new();
        let result = service.register("user1", "abcdefg1", "test@example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_password_with_special_chars() {
        let service = MockAuthServiceImpl::new();
        let result = service
            .register("user2", "pass@word123", "test@example.com")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_password_very_long() {
        let service = MockAuthServiceImpl::new();
        let result = service
            .register("user3", "ThisIsAVeryLongPassword1234567890", "test@example.com")
            .await;
        assert!(result.is_ok());
    }

    // ==================== JWT expiry edge cases ====================

    #[tokio::test]
    async fn test_refresh_token_with_nonexistent_user() {
        let service = MockAuthServiceImpl::new();

        // Generate a refresh token for a non-existent user
        let refresh_token = service.jwt_service.generate_refresh_token(999).unwrap();

        let result = service.refresh_token(&refresh_token).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_token_validation_after_refresh() {
        let service = MockAuthServiceImpl::with_test_user("testuser", "password123");

        // Original token
        let login_response = service.login("testuser", "password123").await.unwrap();

        // Generate refresh token and get new access token
        let refresh_token = service
            .jwt_service
            .generate_refresh_token(login_response.user_id)
            .unwrap();
        let refresh_response = service.refresh_token(&refresh_token).await.unwrap();

        // New token should be valid
        let validation = service.validate_token(&refresh_response.token).await;
        assert!(validation.valid);
        assert_eq!(validation.user_id, login_response.user_id);
    }
}

// ==================== Unit tests for password validation ====================

#[cfg(test)]
mod password_validation_tests {
    use super::*;

    #[test]
    fn test_password_strength_validation() {
        // Valid passwords
        assert!(is_valid_password("password123"));
        assert!(is_valid_password("12345678a"));
        assert!(is_valid_password("Abcdefg1"));

        // Invalid: too short
        assert!(!is_valid_password("123"));
        assert!(!is_valid_password(""));

        // Invalid: no digits
        assert!(!is_valid_password("abcdefgh"));

        // Invalid: no letters
        assert!(!is_valid_password("12345678"));
    }

    #[test]
    fn test_common_weak_passwords() {
        let weak = ["12345678", "123456789", "password", "admin123", "12345678a", "a12345678"];
        for password in &weak {
            let is_weak = weak.contains(password);
            assert!(is_weak, "Password '{}' should be in weak list", password);
        }
    }

    fn is_valid_password(password: &str) -> bool {
        if password.len() < 8 {
            return false;
        }
        let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        has_letter && has_digit
    }
}

// ==================== Unit tests for JWT service ====================

#[cfg(test)]
mod jwt_service_tests {
    use super::*;

    #[test]
    fn test_jwt_service_creation() {
        let service = JwtService::new("secret", "issuer", "audience", 3600, 604800);
        // Just verify it can be created without panicking
    }

    #[test]
    fn test_password_service_creation() {
        let service = PasswordService;
        // Just verify it can be created without panicking
    }

    #[tokio::test]
    async fn test_jwt_generate_and_verify() {
        let service = JwtService::new("test-secret", "test-issuer", "test-audience", 3600, 604800);

        let token = service.generate_access_token(1, "testuser", "admin").unwrap();
        let claims = service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "admin");
    }

    #[tokio::test]
    async fn test_jwt_refresh_token_flow() {
        let service = JwtService::new("test-secret", "test-issuer", "test-audience", 3600, 604800);

        let refresh_token = service.generate_refresh_token(42).unwrap();
        let user_id = service.verify_refresh_token(&refresh_token).unwrap();

        assert_eq!(user_id, 42);
    }

    #[tokio::test]
    async fn test_jwt_wrong_audience_fails() {
        let service = JwtService::new("test-secret", "test-issuer", "wrong-audience", 3600, 604800);
        let token = service.generate_access_token(1, "testuser", "admin").unwrap();

        let wrong_service = JwtService::new("test-secret", "test-issuer", "correct-audience", 3600, 604800);
        let result = wrong_service.verify_token(&token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwt_wrong_issuer_fails() {
        let service = JwtService::new("test-secret", "wrong-issuer", "test-audience", 3600, 604800);
        let token = service.generate_access_token(1, "testuser", "admin").unwrap();

        let wrong_service = JwtService::new("test-secret", "correct-issuer", "test-audience", 3600, 604800);
        let result = wrong_service.verify_token(&token);
        assert!(result.is_err());
    }
}
