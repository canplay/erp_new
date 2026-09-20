//! 租户生命周期处理器
use super::*;

// ============== 租户生命周期接口 ==============

/// 激活租户
pub async fn activate_tenant(
    state: Arc<TenantAppState>,
    tenant_id: i64,
) -> Result<String, Status> {
    state
        .lifecycle
        .activate_tenant(tenant_core::TenantId::new(tenant_id))
        .await
        .map(|_| "active".to_string())
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 暂停租户
pub async fn suspend_tenant(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    reason: &str,
) -> Result<String, Status> {
    state
        .lifecycle
        .suspend_tenant(tenant_core::TenantId::new(tenant_id), reason)
        .await
        .map(|_| "suspended".to_string())
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 续期租户
pub async fn renew_tenant(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    days: i64,
) -> Result<DateTime<Utc>, Status> {
    state
        .lifecycle
        .renew_tenant(tenant_core::TenantId::new(tenant_id), days)
        .await
        .map(|_| Utc::now() + chrono::Duration::days(days))
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 获取租户状态
pub async fn get_tenant_state(
    state: Arc<TenantAppState>,
    tenant_id: i64,
) -> Result<TenantState, Status> {
    state
        .lifecycle
        .get_tenant_state(tenant_core::TenantId::new(tenant_id))
        .await
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

/// 获取即将过期的租户
pub async fn list_expiring_tenants(
    state: Arc<TenantAppState>,
    within_days: i64,
) -> Result<Vec<(i64, String, DateTime<Utc>, i32)>, Status> {
    state
        .lifecycle
        .get_expiring_tenants(within_days)
        .await
        .map(|tenants| {
            tenants
                .into_iter()
                .map(|t| (t.value(), "".to_string(), Utc::now(), 0))
                .collect()
        })
        .map_err(|e| Status::internal(format!("Lifecycle error: {e}" )))
}

