//! Route builder - centralizes router creation with middleware
//!
//! Assembles all route modules, applies middleware layers (CORS, logging,
//! rate limiting, auth, operation logging), and returns the fully configured `axum::Router`.
//!
//! # Middleware Execution Order (outermost → innermost)
//! 1. `security_headers_middleware` - security response headers (CSP, HSTS, etc.)
//! 2. `request_id_middleware` - generates trace/request IDs for every request
//! 3. `cors_layer` - handles CORS preflight and headers
//! 4. `logging_middleware` - logs all requests/responses with timing
//! 5. `rate_limit_middleware` - enforces per-IP rate limits (before auth to protect login)
//! 6. `auth_middleware` - JWT verification and user header injection
//! 7. `operation_log_middleware` - records write operations to audit logs (after auth for user info)
//! 8. `grpc_error_handler_middleware` - catches handler errors and returns consistent responses

use std::sync::Arc;
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;
use crate::{
    AppState, AuthState, RateLimitState,
    enhanced_logging::request_id_middleware,
    grpc_error_middleware::grpc_error_handler_middleware,
    middleware::{auth_middleware, cors_layer, logging_middleware, operation_log_middleware, rate_limit_middleware, security_headers_middleware},
};
use common::middleware::csrf_protection_middleware;

/// Create the fully configured HTTP router
///
/// # Architecture
/// - All frontend requests enter through HTTP on api-gateway
/// - api-gateway translates requests to gRPC calls
/// - gRPC responses are converted back to HTTP for the frontend
pub fn create_router(
    state: Arc<AppState>,
    rate_limit_state: RateLimitState,
    auth_state: AuthState,
) -> Router {
    // ================================================================
    // Middleware layers applied from innermost to outermost.
    // Execution order is outermost → innermost (request flows through
    // outermost first, then each inner layer).
    // ================================================================

    // Build base routes + innermost middleware first.
    let router = crate::routes::all_routes()
        .route("/ws/status", axum::routing::get(crate::ws_routes::ws_status_handler))
        .route("/ws/messages", axum::routing::get(crate::ws_routes::ws_messages_handler))
        // 8. Innermost: gRPC error handler (catches handler/propagation errors)
        .layer(axum::middleware::from_fn(grpc_error_handler_middleware));

    // 7. CSRF 保护中间件 —— 验证写操作的 CSRF Token（需在 auth 之后）
    let router = router
        .layer(axum::middleware::from_fn(csrf_protection_middleware));

    // 7. 操作日志中间件 —— 记录写操作到审计日志（需在 auth 之后以获取用户信息）
    let router = router
        .layer(axum::middleware::from_fn(operation_log_middleware));

    // 6. Auth middleware (always applied - JWT is required at startup)
    let router = {
        tracing::info!("应用 JWT 鉴权中间件");
        router
            .layer(axum::middleware::from_fn(auth_middleware))
            .layer(axum::extract::Extension(auth_state))
    };

    // Apply remaining middleware + state conversion.
    // .with_state() converts from Router<Arc<AppState>> to Router<()>.
    router
        // 5. Rate limiting (applied before auth to protect login endpoints)
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::extract::Extension(rate_limit_state))
        // 4. Logging (captures all requests including rejected ones)
        .layer(axum::middleware::from_fn(logging_middleware))
        // 3. CORS (must handle preflight OPTIONS before any processing)
        .layer(cors_layer(&state.cors_config))
        // 2. Request/Trace ID generation
        .layer(axum::middleware::from_fn(request_id_middleware))
        // 1. Request body size limit (10MB)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        // 0. Security response headers (absolute outermost, applied to all responses)
        .layer(axum::middleware::from_fn(security_headers_middleware))
        // Provide shared state to route handlers (converts to Router<()>)
        // 同时注入 Extension 供 operation_log_middleware 访问 AppState
        .layer(axum::extract::Extension(state.clone()))
        .with_state(state)
}
