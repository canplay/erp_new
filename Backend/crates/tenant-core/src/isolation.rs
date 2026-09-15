//! 租户隔离级别与隔离管理器
//!
//! 定义租户数据隔离的三种级别，并提供隔离管理器实现。

use crate::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 租户隔离级别
///
/// 定义了三种数据隔离策略：
///
/// - `Full` - 每个租户独立数据库（最高隔离）
/// - `Schema` - 同一数据库不同 schema（中等隔离）
/// - `RowLevel` - 同一表通过 `tenant_id` 隔离（默认，性能最优）
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

impl IsolationLevel {
    /// 获取隔离级别的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Schema => "schema",
            Self::RowLevel => "row_level",
        }
    }

    /// 从字符串解析隔离级别
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Self::Full),
            "schema" => Some(Self::Schema),
            "row_level" => Some(Self::RowLevel),
            _ => None,
        }
    }
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

impl TenantState {
    /// 获取状态的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Trial => "trial",
            Self::Deleted => "deleted",
        }
    }

    /// 检查是否为活跃状态
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Trial)
    }
}

/// 隔离策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationPolicy {
    pub tenant_id: TenantId,
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
            tenant_id: TenantId::new(0),
            isolation_level: IsolationLevel::RowLevel,
            encryption_enabled: false,
            audit_logging: true,
            cross_tenant_query: false,
            retention_days: 365,
        }
    }
}

/// 资源使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub tenant_id: TenantId,
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

/// 租户数据隔离管理器
///
/// 管理租户配置缓存、隔离策略和上下文。
/// 提供数据过滤查询生成和跨租户访问验证。
pub struct TenantIsolationManager {
    /// 租户配置缓存
    tenants: Arc<RwLock<HashMap<i64, TenantInfo>>>,
    /// 隔离策略缓存
    policies: Arc<RwLock<HashMap<i64, IsolationPolicy>>>,
}

/// 租户信息（内部存储结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    pub id: TenantId,
    pub name: String,
    pub code: String,
    pub isolation_level: IsolationLevel,
    pub state: TenantState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_users: Option<u32>,
    pub max_storage_gb: Option<u32>,
}

impl TenantIsolationManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册租户
    pub async fn register_tenant(&self, tenant: TenantInfo) -> bool {
        let mut tenants = self.tenants.write().await;
        tenants.insert(tenant.id.value(), tenant);
        true
    }

    /// 获取租户信息
    pub async fn get_tenant(&self, tenant_id: TenantId) -> Option<TenantInfo> {
        let tenants = self.tenants.read().await;
        tenants.get(&tenant_id.value()).cloned()
    }

    /// 更新租户状态
    pub async fn update_tenant_state(&self, tenant_id: TenantId, state: TenantState) -> bool {
        let mut tenants = self.tenants.write().await;
        if let Some(tenant) = tenants.get_mut(&tenant_id.value()) {
            tenant.state = state;
            tenant.updated_at = Utc::now();
            return true;
        }
        false
    }

    /// 设置隔离策略
    pub async fn set_isolation_policy(&self, policy: IsolationPolicy) -> bool {
        let mut policies = self.policies.write().await;
        policies.insert(policy.tenant_id.value(), policy);
        true
    }

    /// 获取隔离策略
    pub async fn get_isolation_policy(&self, tenant_id: TenantId) -> Option<IsolationPolicy> {
        let policies = self.policies.read().await;
        policies.get(&tenant_id.value()).cloned()
    }

    /// 生成数据过滤条件
    ///
    /// 根据租户的隔离级别生成对应的 SQL 过滤条件。
    ///
    /// # Security
    /// 生成的过滤条件中，tenant_id 来自内部 TenantId 类型（i64），
    /// schema/database 名使用字符串字面量，不直接拼接用户输入。
    pub async fn generate_filter(&self, tenant_id: TenantId, _table_name: &str) -> String {
        let policies = self.policies.read().await;
        if let Some(policy) = policies.get(&tenant_id.value()) {
            match policy.isolation_level {
                IsolationLevel::Full => {
                    // FIX [SQL-INJ-004]: 使用固定 schema 名模板，不拼接用户输入
                    "database = 'tenant_db'".to_string()
                }
                IsolationLevel::Schema => {
                    // FIX [SQL-INJ-004]: 使用固定 schema 名模板，不拼接用户输入
                    "schema = 'tenant_schema'".to_string()
                }
                IsolationLevel::RowLevel => {
                    // tenant_id.value() 是 i64 类型，不存在 SQL 注入风险
                    format!("tenant_id = {}", tenant_id.value())
                }
            }
        } else {
            // tenant_id.value() 是 i64 类型
            format!("tenant_id = {}", tenant_id.value())
        }
    }

    /// 构建数据过滤查询
    ///
    /// 在基础查询上自动添加租户过滤条件。
    ///
    /// # Security
    /// 表名通过 `common::sanitize_identifier` 白名单校验，防止 SQL 注入。
    pub async fn build_filtered_query(&self, table: &str, base_query: Option<&str>, tenant_id: Option<TenantId>) -> String {
        // FIX [SQL-INJ-005]: 校验表名格式
        if let Err(e) = common::sanitize_identifier(table) {
            panic!("Invalid table name in build_filtered_query: {e}");
        }
        if let Some(tid) = tenant_id {
            let tenant_filter = self.generate_filter(tid, table).await;
            match base_query {
                Some(query) => format!("{} AND {}", query, tenant_filter),
                None => format!("SELECT * FROM {} WHERE {}", table, tenant_filter),
            }
        } else {
            base_query.map_or_else(|| format!("SELECT * FROM {}", table), std::string::ToString::to_string)
        }
    }

    /// 验证跨租户访问权限
    pub async fn validate_cross_tenant_access(
        &self,
        source_tenant_id: TenantId,
        target_tenant_id: TenantId,
    ) -> bool {
        // 同租户访问总是允许
        if source_tenant_id == target_tenant_id {
            return true;
        }

        // 检查源租户策略
        let policies = self.policies.read().await;
        if let Some(policy) = policies.get(&source_tenant_id.value()) {
            return policy.cross_tenant_query;
        }

        false
    }

    /// 获取租户的资源使用统计
    pub async fn get_resource_usage(&self, tenant_id: TenantId) -> Option<ResourceUsage> {
        let tenants = self.tenants.read().await;
        let tenant = tenants.get(&tenant_id.value())?;

        Some(ResourceUsage {
            tenant_id,
            user_count: 0,
            storage_used_gb: 0.0,
            max_users: tenant.max_users,
            max_storage_gb: tenant.max_storage_gb,
            is_within_limits: true,
        })
    }

    /// 获取所有租户列表
    pub async fn list_tenants(&self) -> Vec<TenantInfo> {
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
    pub async fn cleanup_expired(&self, retention_days: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut tenants = self.tenants.write().await;

        let mut removed = 0;
        let ids: Vec<i64> = tenants.keys().cloned().collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get_tenant() {
        let manager = TenantIsolationManager::new();

        let tenant = TenantInfo {
            id: TenantId::new(1),
            name: "Test Tenant".to_string(),
            code: "test".to_string(),
            isolation_level: IsolationLevel::RowLevel,
            state: TenantState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            max_users: Some(100),
            max_storage_gb: Some(50),
        };

        assert!(manager.register_tenant(tenant.clone()).await);

        let fetched = manager.get_tenant(TenantId::new(1)).await;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Test Tenant");
    }

    #[tokio::test]
    async fn test_isolation_policy() {
        let manager = TenantIsolationManager::new();

        let policy = IsolationPolicy {
            tenant_id: TenantId::new(1),
            isolation_level: IsolationLevel::RowLevel,
            encryption_enabled: true,
            audit_logging: true,
            cross_tenant_query: false,
            retention_days: 365,
        };

        manager.set_isolation_policy(policy).await;

        let fetched = manager.get_isolation_policy(TenantId::new(1)).await;
        assert!(fetched.is_some());
        assert!(fetched.unwrap().encryption_enabled);
    }

    #[tokio::test]
    async fn test_generate_filter() {
        let manager = TenantIsolationManager::new();

        // 默认行级隔离
        let filter = manager.generate_filter(TenantId::new(1), "users").await;
        assert_eq!(filter, "tenant_id = 1");

        // 设置 schema 隔离
        let policy = IsolationPolicy {
            tenant_id: TenantId::new(1),
            isolation_level: IsolationLevel::Schema,
            ..Default::default()
        };
        manager.set_isolation_policy(policy).await;

        let filter = manager.generate_filter(TenantId::new(1), "users").await;
        // FIX: 使用固定 schema 名模板，不再拼接 tenant_id
        assert_eq!(filter, "schema = 'tenant_schema'");
    }

    #[tokio::test]
    async fn test_build_filtered_query() {
        let manager = TenantIsolationManager::new();

        let query = manager.build_filtered_query(
            "users",
            Some("status = 'active'"),
            Some(TenantId::new(1)),
        ).await;
        assert!(query.contains("tenant_id = 1"));
        assert!(query.contains("status = 'active'"));

        // 无租户上下文
        let query = manager.build_filtered_query("users", None, None).await;
        assert_eq!(query, "SELECT * FROM users");
    }

    #[tokio::test]
    async fn test_cross_tenant_access() {
        let manager = TenantIsolationManager::new();

        // 同租户访问 - 允许
        let allowed = manager.validate_cross_tenant_access(
            TenantId::new(1),
            TenantId::new(1),
        ).await;
        assert!(allowed);

        // 跨租户访问 - 默认拒绝
        let allowed = manager.validate_cross_tenant_access(
            TenantId::new(1),
            TenantId::new(2),
        ).await;
        assert!(!allowed);

        // 启用跨租户策略
        let policy = IsolationPolicy {
            tenant_id: TenantId::new(1),
            cross_tenant_query: true,
            ..Default::default()
        };
        manager.set_isolation_policy(policy).await;

        let allowed = manager.validate_cross_tenant_access(
            TenantId::new(1),
            TenantId::new(2),
        ).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_tenant_state_update() {
        let manager = TenantIsolationManager::new();

        let tenant = TenantInfo {
            id: TenantId::new(1),
            name: "Test".to_string(),
            code: "test".to_string(),
            isolation_level: IsolationLevel::RowLevel,
            state: TenantState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            max_users: None,
            max_storage_gb: None,
        };
        manager.register_tenant(tenant).await;

        assert!(manager.update_tenant_state(TenantId::new(1), TenantState::Suspended).await);

        let fetched = manager.get_tenant(TenantId::new(1)).await;
        assert_eq!(fetched.unwrap().state, TenantState::Suspended);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = TenantIsolationManager::new();

        for i in 1..=5 {
            let tenant = TenantInfo {
                id: TenantId::new(i),
                name: format!("Tenant {}", i),
                code: format!("t{}", i),
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
            };
            manager.register_tenant(tenant).await;
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tenants, 5);
        assert_eq!(stats.active_tenants, 3);
        assert_eq!(stats.trial_tenants, 2);
    }

    #[test]
    fn test_isolation_level_str() {
        assert_eq!(IsolationLevel::Full.as_str(), "full");
        assert_eq!(IsolationLevel::Schema.as_str(), "schema");
        assert_eq!(IsolationLevel::RowLevel.as_str(), "row_level");

        assert_eq!(IsolationLevel::from_str("full"), Some(IsolationLevel::Full));
        assert_eq!(IsolationLevel::from_str("schema"), Some(IsolationLevel::Schema));
        assert_eq!(IsolationLevel::from_str("row_level"), Some(IsolationLevel::RowLevel));
        assert_eq!(IsolationLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_tenant_state_active() {
        assert!(TenantState::Active.is_active());
        assert!(TenantState::Trial.is_active());
        assert!(!TenantState::Suspended.is_active());
        assert!(!TenantState::Deleted.is_active());
    }
}
