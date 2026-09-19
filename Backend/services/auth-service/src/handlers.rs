//! Auth Service gRPC Handlers
//!
//! 使用 Repository 模式进行数据库操作

use auth_core::{JwtService, PasswordService};
use grpc_proto::auth::{
    LoginRequest, LoginResponse, RefreshRequest, RefreshResponse, RegisterRequest,
    RegisterResponse, TokenRequest, TokenResponse, auth_service_server::{AuthService, AuthServiceServer},
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::repository::UserRepository;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub user_repository: UserRepository,
    pub jwt_service: JwtService,
    pub password_service: PasswordService,
}

impl AppState {
    /// 创建新的应用状态
    #[must_use]
    pub fn new(
        pool: sqlx::PgPool,
        jwt_service: JwtService,
        password_service: PasswordService,
    ) -> Self {
        Self {
            user_repository: UserRepository::new(pool),
            jwt_service,
            password_service,
        }
    }
}

/// `AuthService` 实现
#[derive(Clone)]
pub struct AuthServiceImpl {
    state: Arc<AppState>,
}

impl AuthServiceImpl {
    /// 创建新的 `AuthService` 实例
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn login(
        &self,
        request: tonic::Request<LoginRequest>,
    ) -> Result<tonic::Response<LoginResponse>, tonic::Status> {
        let req = request.into_inner();

        // 使用 Repository 查询用户
        let user = self
            .state
            .user_repository
            .find_by_username(&req.username)
            .await
            .map_err(common::grpc_error::IntoTonicStatus::into_grpc_status)?;

        match user {
            Some(user) => {
                // 审计修复 (B4): 校验账号状态, 禁用(0)/锁定账号不允许登录
                if user.status != 1 {
                    tracing::warn!("登录被拒绝: 账号状态异常 username={}" , user.username);
                    return Err(tonic::Status::unauthenticated("账号已被禁用" ));
                }

                // 密码校验: 严格 bcrypt 校验。must_change_password 仅控制
                // "登录后强制跳转改密" , 不豁免密码验证(修复: 原实现任意密码可登录)
                if !self
                    .state
                    .password_service
                    .verify_bcrypt(&req.password, &user.password_hash)
                {
                    return Err(tonic::Status::unauthenticated("Invalid credentials" ));
                }

                // 生成 Token
                let token = self
                    .state
                    .jwt_service
                    .generate_access_token(user.id, &user.username, "user" )
                    .map_err(|e| {
                        tonic::Status::internal(format!("Token generation error: {e}" ))
                    })?;

                Ok(tonic::Response::new(LoginResponse {
                    token,
                    user_id: user.id,
                    username: user.username,
                    role: user.role,
                    must_change_password: user.must_change_password,
                }))
            }
            None => Err(tonic::Status::unauthenticated("Invalid credentials" )),
        }
    }

    async fn register(
        &self,
        request: tonic::Request<RegisterRequest>,
    ) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
        let req = request.into_inner();

        // 审计修复 (B4): 密码策略校验——长度 + 复杂度, 拒绝空密码/弱口令
        validate_password_strength(&req.password)?;

        // 密码哈希
        let password_hash = self
            .state
            .password_service
            .hash_bcrypt(&req.password)
            .map_err(|e| tonic::Status::internal(format!("Password hash error: {e}" )))?;

        // 使用 Repository 创建用户
        let user_id = self
            .state
            .user_repository
            .create(&req.username, &password_hash, Some(req.email.clone()))
            .await
            .map_err(common::grpc_error::IntoTonicStatus::into_grpc_status)?;

        // 生成 Token
        let token = self
            .state
            .jwt_service
            .generate_access_token(user_id, &req.username, "user" )
            .map_err(|e| tonic::Status::internal(format!("Token generation error: {e}" )))?;

        Ok(tonic::Response::new(RegisterResponse {
            token,
            user_id,
            username: req.username,
        }))
    }

    async fn validate_token(
        &self,
        request: tonic::Request<TokenRequest>,
    ) -> Result<tonic::Response<TokenResponse>, tonic::Status> {
        let req = request.into_inner();

        match self.state.jwt_service.verify_token(&req.token) {
            Ok(claims) => Ok(tonic::Response::new(TokenResponse {
                valid: true,
                user_id: claims.sub,
                username: claims.username,
                role: claims.role,
            })),
            Err(_) => Ok(tonic::Response::new(TokenResponse {
                valid: false,
                user_id: 0,
                username: String::new(),
                role: String::new(),
            })),
        }
    }

    async fn refresh_token(
        &self,
        request: tonic::Request<RefreshRequest>,
    ) -> Result<tonic::Response<RefreshResponse>, tonic::Status> {
        let req = request.into_inner();

        // 验证刷新 Token
        let user_id = self
            .state
            .jwt_service
            .verify_refresh_token(&req.refresh_token)
            .map_err(|_| tonic::Status::unauthenticated("Invalid refresh token" ))?;

        // 使用 Repository 查询用户
        let user = self
            .state
            .user_repository
            .find_by_id(user_id)
            .await
            .map_err(common::grpc_error::IntoTonicStatus::into_grpc_status)?;

        match user {
            Some(user) => {
                let token = self
                    .state
                    .jwt_service
                    .generate_access_token(user_id, &user.username, "user" )
                    .map_err(|e| {
                        tonic::Status::internal(format!("Token generation error: {e}" ))
                    })?;

                Ok(tonic::Response::new(RefreshResponse {
                    token,
                    user_id,
                    username: user.username,
                    role: "user".to_string(),
                }))
            }
            None => Err(tonic::Status::not_found("User not found" )),
        }
    }
}

/// 密码策略校验 (审计修复 B4)
///
/// 规则: 长度 ≥ 8; 必须同时包含字母与数字; 拒绝常见弱口令
fn validate_password_strength(password: &str) -> Result<(), tonic::Status> {
    if password.len() < 8 {
        return Err(tonic::Status::invalid_argument(
            "密码长度不能少于 8 位" ,
        ));
    }
    let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_letter || !has_digit {
        return Err(tonic::Status::invalid_argument(
            "密码必须同时包含字母和数字" ,
        ));
    }
    const WEAK: &[&str] = &[
        "12345678" , "123456789" , "password" , "admin123" , "12345678a" , "a12345678" ,
    ];
    let lower = password.to_lowercase();
    if WEAK.contains(&lower.as_str()) {
        return Err(tonic::Status::invalid_argument(
            "密码过于简单, 请更换" ,
        ));
    }
    Ok(())
}

impl common::service_bootstrap::GrpcServiceBuilder for AuthServiceImpl {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}" ).into() })?;
        let server = AuthServiceServer::new(AuthServiceImpl::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}" , e);
            }
        });
        Ok(handle)
    }
}
