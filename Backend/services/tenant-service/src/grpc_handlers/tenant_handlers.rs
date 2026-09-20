//! 租户管理处理器
use super::*;

// ============== 租户管理接口实现 ==============

/// 获取租户详情
pub async fn get_tenant(state: Arc<TenantAppState>, id: i64) -> Result<Option<TenantInfo>, Status> {
    state
        .repository
        .find_by_id(id)
        .await
        .map(|opt| opt.map(TenantInfo::from))
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取租户列表
pub async fn list_tenants(
    state: Arc<TenantAppState>,
    page: i32,
    page_size: i32,
    keyword: Option<String>,
) -> Result<PaginatedTenantsInfo, Status> {
    state
        .repository
        .list(page, page_size, keyword.as_deref())
        .await
        .map(|result| PaginatedTenantsInfo {
            tenants: result
                .tenants
                .into_iter()
                .map(TenantListItemInfo::from)
                .collect(),
            total: result.total,
        })
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 创建租户
pub async fn create_tenant(
    state: Arc<TenantAppState>,
    name: String,
    code: String,
    domain: Option<String>,
    description: Option<String>,
    max_users: i32,
    max_storage: i64,
) -> Result<i64, Status> {
    state
        .repository
        .create(
            &name,
            &code,
            domain.as_deref(),
            description.as_deref(),
            max_users,
            max_storage,
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 更新租户参数
#[derive(Debug, Clone)]
pub struct UpdateTenantParams {
    pub id: i64,
    pub name: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
    pub max_users: Option<i32>,
    pub max_storage: Option<i64>,
}

/// 更新租户
pub async fn update_tenant(
    state: Arc<TenantAppState>,
    params: UpdateTenantParams,
) -> Result<bool, Status> {
    state
        .repository
        .update(
            params.id,
            crate::repository::UpdateTenantParams {
                name: params.name,
                domain: params.domain,
                description: params.description,
                status: params.status,
                max_users: params.max_users,
                max_storage: params.max_storage,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 删除租户
pub async fn delete_tenant(state: Arc<TenantAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .delete(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取租户用户列表
pub async fn list_tenant_users(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    page: i32,
    page_size: i32,
) -> Result<Vec<TenantUserInfo>, Status> {
    state
        .repository
        .list_users(tenant_id, page, page_size, None, None)
        .await
        .map(|result| result.users.into_iter().map(TenantUserInfo::from).collect())
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 添加租户用户
pub async fn add_tenant_user(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    user_id: i64,
    role: String,
    department: Option<String>,
    position: Option<String>,
) -> Result<bool, Status> {
    state
        .repository
        .add_user(
            tenant_id,
            user_id,
            &role,
            department.as_deref(),
            position.as_deref(),
        )
        .await
        .map(|()| true)
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 更新租户用户
pub async fn update_tenant_user(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    user_id: i64,
    role: Option<String>,
    department: Option<String>,
    position: Option<String>,
) -> Result<bool, Status> {
    state
        .repository
        .update_user(
            tenant_id,
            user_id,
            role.as_deref(),
            department.as_deref(),
            position.as_deref(),
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 移除租户用户
pub async fn remove_tenant_user(
    state: Arc<TenantAppState>,
    tenant_id: i64,
    user_id: i64,
) -> Result<bool, Status> {
    state
        .repository
        .remove_user(tenant_id, user_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

/// 获取使用统计
pub async fn get_usage_stats(
    state: Arc<TenantAppState>,
    tenant_id: i64,
) -> Result<UsageStatsInfo, Status> {
    state
        .repository
        .get_usage_stats(tenant_id)
        .await
        .map(|s| UsageStatsInfo {
            total_users: s.total_users,
            active_users: s.active_users,
            used_storage: s.used_storage,
            max_storage: s.max_storage,
            monthly_api_calls: s.monthly_api_calls,
            api_call_limit: s.api_call_limit,
        })
        .map_err(|e| Status::internal(format!("Database error: {e}" )))
}

