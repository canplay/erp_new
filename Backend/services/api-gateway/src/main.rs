//! API Gateway 服务入口
//!
//! 使用 ServiceBootstrap 统一启动器
//!
//! | 协议 | 端口 | 说明 |
//! |------|------|------|
//! | HTTP | 8090 | REST/WebSocket 接口
//!

use api_gateway::{AppState, AuthState, RateLimitState, create_router};
use common::service_bootstrap::{ServiceBootstrap, ServiceConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ServiceBootstrap 统一启动器（不含 gRPC 端口）
    let config = ServiceConfig {
        service_name: "api-gateway",
        http_port: 8090,
        grpc_port: 0,
        grpc_addr: "".to_string(),
        http_addr: "0.0.0.0:8090".to_string(),
    };

    let http_port = config.http_port;
    let bootstrap = ServiceBootstrap::new(config);

    // ========== API Gateway HTTP 配置 ==========
    let host = std::env::var("API_GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr = format!("{}:{}", host, http_port);

    // ========== 限流配置 ==========
    let max_requests: u32 = std::env::var("RATE_LIMIT_MAX")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);
    let window_secs: u64 = std::env::var("RATE_LIMIT_WINDOW")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);
    let burst: u32 = std::env::var("RATE_LIMIT_BURST")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);
    let rate_limit_state = RateLimitState::new(max_requests, window_secs, burst);

    // ========== 初始化数据库连接池 ==========
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/myai".to_string());
    let pool = sqlx::PgPool::connect_lazy(&db_url)
        .expect("Failed to connect to database");

    // ========== 初始化 gRPC 客户端、服务发现和应用状态 ==========
    let state = Arc::new(AppState::new(pool).await?);

    // ========== 启动后台服务发现重连任务 ====
    AppState::start_reconnection_task(&state);

    // ========== JWT 鉴权配置（必选，生产禁止跳过鉴权） ==========
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET 环境变量必须配置(生产环境禁止无鉴权启动)");
    let jwt_issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "myai".to_string());
    let jwt_audience = std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "myai-users".to_string());
    let auth_state = Some(AuthState::new(jwt_secret, jwt_issuer, jwt_audience));
    tracing::info!("JWT 鉴权已启用");

    // ========== CORS 配置 ==========
    let _cors_origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    let _cors_methods = std::env::var("CORS_ALLOWED_METHODS")
        .unwrap_or_else(|_| "GET,POST,PUT,DELETE,PATCH,OPTIONS".to_string());
    let _cors_headers = std::env::var("CORS_ALLOWED_HEADERS")
        .unwrap_or_else(|_| "Content-Type,Authorization,X-Requested-With".to_string());
    let _cors_max_age: u64 = std::env::var("CORS_MAX_AGE")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);

    // ========== 日志输出启动信息 ==========
    tracing::info!("{} starting on {}", "api-gateway", addr);

    // ========== 构建并启动服务（支持优雅关闭） ==========
    let app = create_router(state, rate_limit_state, auth_state);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("API Gateway 服务已启动，监听 {}", addr);
    let http_server = axum::serve(listener, app)
        .with_graceful_shutdown(bootstrap.create_shutdown_future());

    let _ = http_server.await;

    tracing::info!("API Gateway 服务已关闭");
    Ok(())
}
