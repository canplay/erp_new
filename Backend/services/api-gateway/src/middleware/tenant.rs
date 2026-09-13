//! 租户解析中间件
//!
//! 从 JWT 或请求头中提取租户 ID，注入到请求上下文中。

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::sync::Arc;
use tracing::{warn, debug};

use tenant_core::context::{set_tenant_context, TenantContext};
use tenant_core::TenantId;

use crate::middleware::jwt::JwtClaims;

/// 从请求中提取租户 ID 并注入到上下文中
pub async fn tenant_extraction_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // 从 JWT Claims 中提取租户 ID
    let tenant_id = request
        .extensions()
        .get::<JwtClaims>()
        .and_then(|claims| claims.tenant_id.as_deref())
        .and_then(|s| s.parse::<i64>().ok())
        .map(TenantId::new);

    // 如果没有从 JWT 获取到，尝试从 Header 获取
    let tenant_id = if let Some(id) = tenant_id {
        Some(id)
    } else {
        request
            .headers()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .map(TenantId::new)
    };

    if let Some(tenant_id) = tenant_id {
        debug!(tenant_id = %tenant_id, "Tenant context set");
        
        // 创建租户上下文
        let user_id = request
            .extensions()
            .get::<JwtClaims>()
            .and_then(|claims| Some(claims.sub));

        let roles = request
            .extensions()
            .get::<JwtClaims>()
            .map(|claims| claims.roles.clone())
            .unwrap_or_default();

        let permissions = request
            .extensions()
            .get::<JwtClaims>()
            .map(|claims| claims.permissions.clone())
            .unwrap_or_default();

        let ctx = TenantContext::new(tenant_id)
            .with_user_id(user_id.unwrap_or(0))
            .with_roles(roles)
            .with_permissions(permissions);

        set_tenant_context(ctx);
    } else {
        warn!("No tenant ID found in request");
    }

    let response = next.run(request).await;
    
    // 清理租户上下文
    tenant_core::context::clear_tenant_context();
    
    Ok(response)
}

/// 获取当前请求的租户 ID
pub fn current_tenant_id(request: &Request<Body>) -> Option<TenantId> {
    // 优先从上下文获取
    tenant_core::context::current_tenant_id().or_else(|| {
        // 从 JWT Claims 获取
        request
            .extensions()
            .get::<JwtClaims>()
            .and_then(|claims| claims.tenant_id.as_deref())
            .and_then(|s| s.parse::<i64>().ok())
            .map(TenantId::new)
    })
}
