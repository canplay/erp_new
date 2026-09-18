//! 多租户数据隔离模块
//!
//! 实现租户数据隔离、租户配置管理、跨租户数据查询控制

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use common::{AppError, AppResult};

/// 租户隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IsolationLevel {
    /// 完全隔离 - 每个租户独立数据库
    Full,
    /// 模式隔离 - 同一数据库不同 schema
    Schema,
    /// 行级隔离 - 同一表通过 `tenant_id` 隔离
    #[default]
    RowLevel,
}

/// 租户状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TenantState {
    /// 活跃
    #[default]
    Active,
    /// 暂停
    Suspended,
    /// 试用
    Trial,
    /// 已删除
    Deleted,
}

/// 租户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub code: String,
    pub isolation_level: IsolationLevel,
    pub state: TenantState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_users: Option<u32>,
    pub max_storage_gb: Option<u32>,
    pub settings: HashMap<String, String>,
}

/// 租户数据隔离上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl TenantContext {
    /// 创建新的租户上下文
    #[must_use]
    pub fn new(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            user_id: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// 设置用户信息
    #[must_use]
    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// 添加角色
    #[must_use]
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// 添加权限
    #[must_use]
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
}

/// 数据隔离策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationPolicy {
    pub tenant_id: String,
    pub isolation_level: IsolationLevel,
    /// 是否启用数据加密
    pub encryption_enabled: bool,
    /// 是否启用审计日志
    pub audit_logging: bool,
    /// 是否允许跨租户查询
    pub cross_tenant_query: bool,
    /// 数据保留天数
    pub retention_days: u64,
}

impl Default for IsolationPolicy {
    fn default() -> Self {
        Self {
            tenant_id: String::new(),
            isolation_level: IsolationLevel::RowLevel,
            encryption_enabled: false,
            audit_logging: true,
            cross_tenant_query: false,
            retention_days: 365,
        }
    }
}

/// 数据查询过滤器（自动注入租户条件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFilter {
    pub tenant_id: String,
    /// 表名 -> 条件映射
    pub table_filters: HashMap<String, String>,
    /// 允许访问的字段
    pub allowed_fields: HashMap<String, Vec<String>>,
    /// 禁止访问的字段
    pub forbidden_fields: HashMap<String, Vec<String>>,
}

/// 租户数据隔离管理器
pub struct TenantIsolationManager {
    /// 租户配置缓存
    tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    /// 隔离策略缓存
    policies: Arc<RwLock<HashMap<String, IsolationPolicy>>>,
    /// 当前租户上下文
    context: Arc<RwLock<Option<TenantContext>>>,
}

impl TenantIsolationManager {
    /// 创建新的管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            context: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建默认隔离级别管理器
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new()
    }

    /// 注册租户
    ///
    /// # Arguments
    /// * `tenant` - 租户信息
    ///
    /// # Returns
    /// 是否成功
    pub async fn register_tenant(&self, tenant: Tenant) -> bool {
        let mut tenants = self.tenants.write().await;
        tenants.insert(tenant.id.clone(), tenant);
        true
    }

    /// 获取租户信息
    ///
    /// # Arguments
    /// * `tenant_id` - 租户 ID
    ///
    /// # Returns
    /// 租户信息（如果存在）
    pub async fn get_tenant(&self, tenant_id: &str) -> Option<Tenant> {
        let tenants = self.tenants.read().await;
        tenants.get(tenant_id).cloned()
    }

    /// 更新租户状态
    ///
    /// # Arguments
    /// * `tenant_id` - 租户 ID
    /// * `state` - 新状态
    ///
    /// # Returns
    /// 是否成功
    pub async fn update_tenant_state(&self, tenant_id: &str, state: TenantState) -> bool {
        let mut tenants = self.tenants.write().await;
        if let Some(tenant) = tenants.get_mut(tenant_id) {
            tenant.state = state;
            tenant.updated_at = Utc::now();
            return true;
        }
        false
    }

    /// 设置隔离策略
    ///
    /// # Arguments
    /// * `policy` - 隔离策略
    ///
    /// # Returns
    /// 是否成功
    pub async fn set_isolation_policy(&self, policy: IsolationPolicy) -> bool {
        let mut policies = self.policies.write().await;
        policies.insert(policy.tenant_id.clone(), policy);
        true
    }

    /// 获取隔离策略
    ///
    /// # Arguments
    /// * `tenant_id` - 租户 ID
    ///
    /// # Returns
    /// 隔离策略（如果存在）
    pub async fn get_isolation_policy(&self, tenant_id: &str) -> Option<IsolationPolicy> {
        let policies = self.policies.read().await;
        policies.get(tenant_id).cloned()
    }

    /// 设置当前租户上下文
    ///
    /// # Arguments
    /// * `context` - 租户上下文
    ///
    pub async fn set_context(&self, context: TenantContext) {
        let mut ctx = self.context.write().await;
        *ctx = Some(context);
    }

    /// 获取当前租户上下文
    ///
    /// # Returns
    /// 租户上下文（如果存在）
    pub async fn get_context(&self) -> Option<TenantContext> {
        let ctx = self.context.read().await;
        ctx.clone()
    }

    /// 清除当前上下文
    pub async fn clear_context(&self) {
        let mut ctx = self.context.write().await;
        *ctx = None;
    }

    /// 生成数据过滤条件
    ///
    /// # Arguments
    /// * `tenant_id` - 租户 ID
    /// * `table_name` - 表名
    ///
    /// # Returns
    /// SQL 过滤条件
    ///
    /// # Security
    /// tenant_id 通过 `common::sanitize_identifier` 白名单校验，
    /// 只允许 `[a-zA-Z_][a-zA-Z0-9_]*` 格式，防止 SQL 注入。
    pub async fn generate_filter(&self, tenant_id: &str, _table_name: &str) -> AppResult<String> {
        // FIX [SQL-INJ-006]: 校验 tenant_id 格式，防止 SQL 注入
        if let Err(e) = common::sanitize_identifier(tenant_id) {
            return Err(AppError::InvalidParam(format!("Invalid tenant_id: {e}" )));
        }
        let policies = self.policies.read().await;
        if let Some(policy) = policies.get(tenant_id) {
            let filter = match policy.isolation_level {
                IsolationLevel::Full => {
                    format!("database = '{tenant_id}'" )
                }
                IsolationLevel::Schema => {
                    format!("schema = '{tenant_id}'" )
                }
                IsolationLevel::RowLevel => {
                    format!("tenant_id = '{tenant_id}'" )
                }
            };
            Ok(filter)
        } else {
            Ok(format!("tenant_id = '{tenant_id}'" ))
        }
    }

    /// 允许构建动态查询的表白名单
    const ALLOWED_TABLES: &[&str] = &["tenants", "tenant_users", "audit_logs", "plans", "subscriptions", "invoices", "usage_records", "tenant_settings"];

    /// 构建数据过滤查询
    ///
    /// # Arguments
    /// * `table` - 表名
    /// * `base_query` - 基础查询条件
    ///
    /// # Returns
    /// 带租户过滤的完整查询
    ///
    /// # Security
    /// 表名通过白名单校验 + `common::sanitize_identifier` 双重防护，防止 SQL 注入。
    /// 使用显式列名替代 `SELECT *`，避免泄露敏感字段。
    pub async fn build_filtered_query(&self, table: &str, base_query: Option<&str>) -> AppResult<String> {
        // FIX [SQL-INJ-008]: 白名单校验表名（纵深防御第一层）
        if !Self::ALLOWED_TABLES.contains(&table) {
            return Err(AppError::InvalidParam(format!("Table not in whitelist: {table}")));
        }
        // FIX [SQL-INJ-007]: 校验表名格式（纵深防御第二层）
        if let Err(e) = common::sanitize_identifier(table) {
            return Err(AppError::InvalidParam(format!("Invalid table name: {e}")));
        }
        let columns = Self::table_columns(table);
        let ctx = self.context.read().await;
        if let Some(context) = ctx.as_ref() {
            let tenant_filter = self.generate_filter(&context.tenant_id, table).await?;
            let query = match base_query {
                Some(query) => format!("SELECT {columns} FROM {table} WHERE {query} AND {tenant_filter}"),
                None => format!("SELECT {columns} FROM {table} WHERE {tenant_filter}"),
            };
            Ok(query)
        } else {
            Ok(base_query.map_or_else(|| format!("SELECT {columns} FROM {table}"), |q| format!("SELECT {columns} FROM {table} WHERE {q}")))
        }
    }

    /// 返回指定表的显式列名列表
    fn table_columns(table: &str) -> &'static str {
        match table {
            "tenants" => "id, name, code, domain, description, max_users, max_storage, status, expires_at, created_at, updated_at",
            "tenant_users" => "id, tenant_id, user_id, username, email, role, department, position, status, joined_at",
            "audit_logs" => "id, tenant_id, user_id, action, resource_type, resource_id, details, created_at",
            "plans" => "id, name, description, plan_type, status, price_monthly, price_yearly, currency, features, quotas, sort_order, is_public, created_at, updated_at",
            "subscriptions" => "id, tenant_id, plan_id, status, current_period_start, current_period_end, trial_end, unit_price, created_at, updated_at",
            "invoices" => "id, subscription_id, tenant_id, amount, currency, status, issued_at, due_at, paid_at, created_at",
            "usage_records" => "id, tenant_id, resource_type, quantity, recorded_at, created_at",
            "tenant_settings" => "id, tenant_id, key, value, created_at, updated_at",
            _ => "id, tenant_id, created_at",
        }
    }

    /// 验证跨租户访问权限
    ///
    /// # Arguments
    /// * `target_tenant_id` - 目标租户 ID
    ///
    /// # Returns
    /// 是否允许访问
    pub async fn validate_cross_tenant_access(&self, target_tenant_id: &str) -> bool {
        let ctx = self.context.read().await;
        let policies = self.policies.read().await;

        // 如果没有上下文，不允许跨租户
        let Some(context) = ctx.as_ref() else {
            return false;
        };

        // 如果是同一租户，允许
        if context.tenant_id == target_tenant_id {
            return true;
        }

        // 检查源租户策略
        if let Some(policy) = policies.get(&context.tenant_id) {
            return policy.cross_tenant_query;
        }

        false
    }

    /// 获取租户的资源使用统计
    ///
    /// # Arguments
    /// * `tenant_id` - 租户 ID
    ///
    /// # Returns
    /// 资源使用统计
    pub async fn get_resource_usage(&self, tenant_id: &str) -> Option<ResourceUsage> {
        let tenants = self.tenants.read().await;
        let tenant = tenants.get(tenant_id)?;

        Some(ResourceUsage {
            tenant_id: tenant_id.to_string(),
            user_count: 0, // 实际实现中从数据库获取
            storage_used_gb: 0.0,
            max_users: tenant.max_users,
            max_storage_gb: tenant.max_storage_gb,
            is_within_limits: true, // 实际实现中计算
        })
    }

    /// 获取所有租户列表
    ///
    /// # Returns
    /// 租户列表
    pub async fn list_tenants(&self) -> Vec<Tenant> {
        let tenants = self.tenants.read().await;
        tenants.values().cloned().collect()
    }

    /// 获取租户统计
    pub async fn get_stats(&self) -> TenantStats {
        let tenants = self.tenants.read().await;
        let policies = self.policies.read().await;

        let total = tenants.len();
        let active = tenants
            .values()
            .filter(|t| t.state == TenantState::Active)
            .count();
        let suspended = tenants
            .values()
            .filter(|t| t.state == TenantState::Suspended)
            .count();
        let trial = tenants
            .values()
            .filter(|t| t.state == TenantState::Trial)
            .count();

        TenantStats {
            total_tenants: total,
            active_tenants: active,
            suspended_tenants: suspended,
            trial_tenants: trial,
            total_policies: policies.len(),
        }
    }

    /// 清理过期租户数据
    ///
    /// # Arguments
    /// * `retention_days` - 保留天数
    ///
    /// # Returns
    /// 清理的租户数量
    pub async fn cleanup_expired(&self, retention_days: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut tenants = self.tenants.write().await;

        let mut removed = 0;
        let ids: Vec<String> = tenants.keys().cloned().collect();

        for id in ids {
            if let Some(tenant) = tenants.get(&id)
                && tenant.state == TenantState::Deleted && tenant.updated_at < cutoff {
                    tenants.remove(&id);
                    removed += 1;
                }
        }

        removed
    }
}

impl Default for TenantIsolationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub tenant_id: String,
    pub user_count: u32,
    pub storage_used_gb: f64,
    pub max_users: Option<u32>,
    pub max_storage_gb: Option<u32>,
    pub is_within_limits: bool,
}

/// 租户统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStats {
    pub total_tenants: usize,
    pub active_tenants: usize,
    pub suspended_tenants: usize,
    pub trial_tenants: usize,
    pub total_policies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_tenant() {
        let manager = TenantIsolationManager::default_manager();

        let tenant = Tenant {
            id: "tenant_001".to_string(),
            name: "测试租户".to_string(),
            code: "test".to_string(),
            isolation_level: IsolationLevel::RowLevel,
            state: TenantState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            max_users: Some(100),
            max_storage_gb: Some(50),
            settings: HashMap::new(),
        };

        let result = manager.register_tenant(tenant.clone()).await;
        assert!(result);

        let fetched = manager.get_tenant("tenant_001" ).await;
        assert!(fetched.is_some());
        assert_eq!(fetched.expect("tenant should exist" ).name, "测试租户" );
    }

    #[tokio::test]
    async fn test_isolation_policy() {
        let manager = TenantIsolationManager::default_manager();

        let policy = IsolationPolicy {
            tenant_id: "tenant_001".to_string(),
            isolation_level: IsolationLevel::RowLevel,
            encryption_enabled: true,
            audit_logging: true,
            cross_tenant_query: false,
            retention_days: 365,
        };

        manager.set_isolation_policy(policy).await;

        let fetched = manager.get_isolation_policy("tenant_001" ).await;
        assert!(fetched.is_some());
        assert!(fetched.expect("tenant should exist" ).encryption_enabled);
    }

    #[tokio::test]
    async fn test_context_and_filter() {
        let manager = TenantIsolationManager::default_manager();

        // 设置上下文
        let context = TenantContext::new("tenant_001" )
            .with_user("user_001" )
            .with_roles(vec!["admin".to_string()]);
        manager.set_context(context).await;

        // 生成过滤条件
        let filter = manager.generate_filter("tenant_001", "tenants").await.unwrap();
        assert_eq!(filter, "tenant_id = 'tenant_001'");

        // 构建过滤查询
        let query = manager
            .build_filtered_query("tenants", Some("status = 'active'"))
            .await
            .unwrap();
        assert!(query.contains("tenant_id = 'tenant_001'"));
        assert!(query.contains("status = 'active'"));
        assert!(!query.contains("SELECT *"));

        // 清除上下文
        manager.clear_context().await;
        let query = manager.build_filtered_query("tenants", None).await.unwrap();
        assert_eq!(query, "SELECT id, name, code, domain, description, max_users, max_storage, status, expires_at, created_at, updated_at FROM tenants");
    }

    #[tokio::test]
    async fn test_cross_tenant_access() {
        let manager = TenantIsolationManager::default_manager();

        // 设置上下文
        let context = TenantContext::new("tenant_001" );
        manager.set_context(context).await;

        // 同租户访问 - 允许
        let allowed = manager.validate_cross_tenant_access("tenant_001" ).await;
        assert!(allowed);

        // 跨租户访问 - 默认拒绝
        let allowed = manager.validate_cross_tenant_access("tenant_002" ).await;
        assert!(!allowed);

        // 启用跨租户策略
        let policy = IsolationPolicy {
            tenant_id: "tenant_001".to_string(),
            cross_tenant_query: true,
            ..Default::default()
        };
        manager.set_isolation_policy(policy).await;

        let allowed = manager.validate_cross_tenant_access("tenant_002" ).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_tenant_state_update() {
        let manager = TenantIsolationManager::default_manager();

        let tenant = Tenant {
            id: "tenant_001".to_string(),
            name: "测试".to_string(),
            code: "test".to_string(),
            isolation_level: IsolationLevel::RowLevel,
            state: TenantState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            max_users: None,
            max_storage_gb: None,
            settings: HashMap::new(),
        };
        manager.register_tenant(tenant).await;

        // 暂停租户
        let updated = manager
            .update_tenant_state("tenant_001" , TenantState::Suspended)
            .await;
        assert!(updated);

        let fetched = manager.get_tenant("tenant_001" ).await;
        assert_eq!(fetched.expect("tenant should exist" ).state, TenantState::Suspended);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = TenantIsolationManager::default_manager();

        // 注册多个租户
        for i in 1..=5 {
            let tenant = Tenant {
                id: format!("tenant_{:03}" , i),
                name: format!("租户 {}" , i),
                code: format!("t{}" , i),
                isolation_level: IsolationLevel::RowLevel,
                state: if i <= 3 {
                    TenantState::Active
                } else {
                    TenantState::Trial
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                expires_at: None,
                max_users: None,
                max_storage_gb: None,
                settings: HashMap::new(),
            };
            manager.register_tenant(tenant).await;
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tenants, 5);
        assert_eq!(stats.active_tenants, 3);
        assert_eq!(stats.trial_tenants, 2);
    }

    #[tokio::test]
    async fn test_list_tenants() {
        let manager = TenantIsolationManager::default_manager();

        let tenant1 = Tenant {
            id: "t1".to_string(),
            name: "租户1".to_string(),
            code: "code1".to_string(),
            isolation_level: IsolationLevel::RowLevel,
            state: TenantState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            max_users: None,
            max_storage_gb: None,
            settings: HashMap::new(),
        };

        let tenant2 = Tenant {
            id: "t2".to_string(),
            name: "租户2".to_string(),
            code: "code2".to_string(),
            isolation_level: IsolationLevel::Schema,
            state: TenantState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            max_users: None,
            max_storage_gb: None,
            settings: HashMap::new(),
        };

        manager.register_tenant(tenant1).await;
        manager.register_tenant(tenant2).await;

        let tenants = manager.list_tenants().await;
        assert_eq!(tenants.len(), 2);
    }
}
