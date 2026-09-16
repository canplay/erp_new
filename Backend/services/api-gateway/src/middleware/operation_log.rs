//! 操作日志中间件
//!
//! 将写操作 (POST/PUT/DELETE/PATCH) 记录到审计日志。
//!
//! # 功能
//! - 仅拦截 /api/ 下的写请求
//! - 跳过健康检查、登录、静态资源等路径
//! - 在响应发送后异步调用 audit-service 的 LogAction RPC
//! - 不阻塞响应

use crate::AppState;

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// 从请求路径和方法推断资源类型和操作类型
fn infer_resource_and_action(path: &str, method: &str) -> (String, i32) {
    // 审计操作枚举 (对应 audit.proto AuditAction)
    const ACTION_CREATE: i32 = 0;
    const ACTION_UPDATE: i32 = 1;
    const ACTION_DELETE: i32 = 2;
    const ACTION_OTHER: i32 = 99;

    let resource_type = path
        .trim_start_matches("/api/" )
        .trim_start_matches("admin/" )
        .trim_start_matches("v1/" )
        .split('/')
        .next()
        .unwrap_or("unknown" )
        .to_string();

    let action = match method {
        "POST" => ACTION_CREATE,
        "PUT" | "PATCH" => ACTION_UPDATE,
        "DELETE" => ACTION_DELETE,
        _ => ACTION_OTHER,
    };

    (resource_type, action)
}

/// 检查路径是否需要记录操作日志
fn should_log_operation(path: &str, method: &str) -> bool {
    let is_write = matches!(
        method,
        "POST" | "PUT" | "DELETE" | "PATCH"
    );

    let skip = !path.starts_with("/api/" )
        || path.starts_with("/api/user/login" )
        || path.starts_with("/api/user/register" )
        || path.starts_with("/api/auth/" )
        || path.starts_with("/api/health" )
        || path.starts_with("/api/audit/" )
        || path.starts_with("/api/export/" )
        || path.starts_with("/ws/" )
        || path.starts_with("/health" )
        || path.starts_with("/ready" )
        || path.starts_with("/metrics" )
        || path.starts_with("/rate-limit" )
        || path.starts_with("/trace" );

    is_write && !skip
}

/// 操作日志中间件
///
/// 在响应发送后异步调用 audit-service 的 LogAction RPC，不阻塞响应。
pub async fn operation_log_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start_time = std::time::Instant::now();

    if !should_log_operation(&path, method.as_str()) {
        return next.run(request).await;
    }

    // 提取用户信息（auth_middleware 已注入 x-user-* 头）
    let user_id: i64 = request
        .headers()
        .get("x-user-id" )
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let username = request
        .headers()
        .get("x-user-name" )
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous" )
        .to_string();

    // 提取客户端 IP
    let _client_ip = crate::enhanced_logging::get_client_ip(&request)
        .unwrap_or_else(|| "unknown".to_string());

    // 从路径推断资源类型和操作
    let (resource_type, action) = infer_resource_and_action(&path, method.as_str());

    // 执行请求
    // 在 next.run 之前提取 AppState
    let app_state = request.extensions().get::<Arc<AppState>>().cloned();

    let response = next.run(request).await;
    let status = response.status().as_u16();
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // 仅在响应成功 (2xx) 时记录操作日志
    if status >= 200 && status < 300 {
        let description = format!("{} {} - {}ms" , method, path, duration_ms);

        // 异步记录到审计日志（失败不阻塞响应）
        // 从请求扩展中获取 AppState（由 route_builder 注入 Extension 层）
        if let Some(state) = app_state {
            tokio::spawn(async move {
                if let Ok(mut client) = state.grpc_clients.read().await.audit_client().await {
                    let _ = client
                        .log_action(
                            user_id,
                            username,
                            action,
                            resource_type,
                            0,
                            description,
                            std::collections::HashMap::new(),
                            std::collections::HashMap::new(),
                        )
                        .await;
                }
            });
        }
    }

    response
}
