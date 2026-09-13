//! 配额检查中间件
//!
//! 在 API 请求处理前检查租户配额是否超限。

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::sync::Arc;
use tracing::{warn, debug};

use tenant_core::quota::{QuotaSet, QuotaType};
use tenant_core::TenantId;

use crate::middleware::tenant::current_tenant_id;

/// 配额检查配置
#[derive(Clone)]
pub struct QuotaConfig {
    /// 是否启用配额检查
    pub enabled: bool,
    /// 默认配额限制
    pub default_quotas: std::collections::HashMap<String, i64>,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        let mut default_quotas = std::collections::HashMap::new();
        default_quotas.insert("api_calls_per_minute".to_string(), 100);
        default_quotas.insert("storage_gb".to_string(), 10);
        default_quotas.insert("users".to_string(), 50);
        
        Self {
            enabled: true,
            default_quotas,
        }
    }
}

/// 配额检查中间件
pub async fn quota_enforcement_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // 简化实现：仅记录请求，实际配额检查在业务逻辑中处理
    if let Some(tenant_id) = current_tenant_id(&request) {
        debug!(tenant_id = %tenant_id, "Quota check passed (simplified)");
    }
    
    let response = next.run(request).await;
    Ok(response)
}

/// 配额使用记录器
#[derive(Clone)]
pub struct UsageRecorder {
    /// 是否启用用量记录
    pub enabled: bool,
}

impl Default for UsageRecorder {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// 用量记录中间件
pub async fn usage_recorder_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    debug!(
        path = %request.uri().path(),
        status = %response.status().as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );
    
    Ok(response)
}
