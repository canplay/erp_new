//! Auth Service - 认证服务入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | gRPC | 9091 | 内部服务通信
//! | HTTP | 8081 | 健康检查端点
//!

use auth_core::{JwtService, PasswordService};
use auth_service::{AppState, AuthServiceImpl};
use common::health::health_routes;
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
// 使用 ServiceBootstrap 统一启动器
    let config = ServiceConfig::from_env("auth-service" , 8081, 9091);

    let bootstrap = ServiceBootstrap::new(config);

    // 从环境变量获取 JWT 配置
    let jwt_secret = std::env::var("JWT_SECRET" )
        .expect("JWT_SECRET must be set - configure it in production environment" );
    // 安全加固: JWT 密钥长度必须 ≥ 32 字符（HS256 安全基线）
    assert!(
        jwt_secret.len() >= 32,
        "JWT_SECRET 长度必须至少 32 字符（当前不足，存在暴力破解风险）"
    );
    let jwt_issuer = std::env::var("JWT_ISSUER" ).unwrap_or_else(|_| "myai".to_string());
    let jwt_audience = std::env::var("JWT_AUDIENCE" ).unwrap_or_else(|_| "myai-users".to_string());
    let access_token_ttl = std::env::var("JWT_ACCESS_TOKEN_TTL" )
        .unwrap_or_else(|_| "3600".to_string())
        .parse::<u64>()
        .unwrap_or(3600);
    let refresh_token_ttl = std::env::var("JWT_REFRESH_TOKEN_TTL" )
        .unwrap_or_else(|_| "604800".to_string())
        .parse::<u64>()
        .unwrap_or(604800);

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL" )
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // 创建 gRPC 服务
    let jwt_service = JwtService::new(&jwt_secret, &jwt_issuer, &jwt_audience, access_token_ttl, refresh_token_ttl);
    let password_service = PasswordService::new(12);
    let grpc_state = Arc::new(AppState::new(pool.clone(), jwt_service, password_service));
    let auth_service = AuthServiceImpl::new(grpc_state);
    let http_router = health_routes();

    bootstrap.start_with_fns(
        || Ok(pool.clone()),
        |_p| Ok(Box::new(auth_service)),
        || Ok(http_router),
    ).await?;

    Ok(())
}
