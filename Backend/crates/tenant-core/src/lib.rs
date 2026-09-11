//! 多租户核心库
//!
//! 提供租户类型定义、租户上下文管理、隔离级别和查询过滤功能。
//!
//! # 设计原则
//!
//! 1. **租户上下文** - 从 JWT 或请求头中提取 `tenant_id`，通过 thread-local 传播
//! 2. **自动隔离** - 所有查询默认按 `tenant_id` 过滤
//! 3. **平台级访问** - root 用户可通过 `.platform()` 方法绕过租户过滤
//! 4. **类型安全** - `TenantId` newtype 防止与其他 ID 混淆

pub mod context;
pub mod isolation;
pub mod middleware;
pub mod tenant_id;

pub use context::{clear_tenant_context, current_tenant_id, set_tenant_context, TenantContext};
pub use isolation::{IsolationLevel, TenantIsolationManager, TenantState};
pub use middleware::tenant_extraction_middleware;
pub use tenant_id::TenantId;

use serde::{Deserialize, Serialize};

/// 租户信息（核心结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub code: String,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub max_users: Option<u32>,
    pub max_storage_gb: Option<u32>,
    pub state: TenantState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Tenant {
    /// 检查租户是否处于活跃状态
    pub fn is_active(&self) -> bool {
        self.state == TenantState::Active
    }

    /// 检查租户是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 检查用户数量是否超限
    pub fn is_within_user_limit(&self, current_count: u32) -> bool {
        self.max_users.map_or(true, |max| current_count < max)
    }
}

/// 租户用户关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUser {
    pub tenant_id: TenantId,
    pub user_id: i64,
    pub role: String,
    pub department: Option<String>,
    pub position: Option<String>,
    pub status: i32,
}

/// 租户查询作用域
///
/// 封装了租户查询的过滤逻辑：
/// - `TenantScope::Tenant(id)` - 只查询指定租户的数据
/// - `TenantScope::Platform` - 查询所有租户数据（仅 root 可用）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantScope {
    /// 限定于特定租户
    Tenant(TenantId),
    /// 平台级访问（跨租户）
    Platform,
}

impl TenantScope {
    /// 从当前上下文创建租户作用域
    ///
    /// 如果上下文中有租户 ID 则返回 `Tenant(id)`，否则返回 `Platform`
    pub fn from_context() -> Self {
        match current_tenant_id() {
            Some(id) => Self::Tenant(id),
            None => Self::Platform,
        }
    }

    /// 检查是否为平台级访问
    pub fn is_platform(&self) -> bool {
        matches!(self, Self::Platform)
    }

    /// 获取租户 ID（如果有）
    pub fn tenant_id(&self) -> Option<&TenantId> {
        match self {
            Self::Tenant(id) => Some(id),
            Self::Platform => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_is_active() {
        let tenant = Tenant {
            id: TenantId::new(1),
            name: "Test Tenant".to_string(),
            code: "test".to_string(),
            domain: None,
            description: None,
            max_users: Some(100),
            max_storage_gb: Some(50),
            state: TenantState::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        };
        assert!(tenant.is_active());
        assert!(!tenant.is_expired());
    }

    #[test]
    fn test_tenant_is_expired() {
        let tenant = Tenant {
            id: TenantId::new(1),
            name: "Expired Tenant".to_string(),
            code: "expired".to_string(),
            domain: None,
            description: None,
            max_users: None,
            max_storage_gb: None,
            state: TenantState::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() - chrono::Duration::days(1)),
        };
        assert!(tenant.is_expired());
    }

    #[test]
    fn test_tenant_user_limit() {
        let tenant = Tenant {
            id: TenantId::new(1),
            name: "Limited Tenant".to_string(),
            code: "limited".to_string(),
            domain: None,
            description: None,
            max_users: Some(50),
            max_storage_gb: None,
            state: TenantState::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        };
        assert!(tenant.is_within_user_limit(49));
        assert!(!tenant.is_within_user_limit(50));
        assert!(!tenant.is_within_user_limit(51));
    }

    #[test]
    fn test_tenant_scope() {
        let scope = TenantScope::Platform;
        assert!(scope.is_platform());
        assert!(scope.tenant_id().is_none());

        let id = TenantId::new(42);
        let scope = TenantScope::Tenant(id.clone());
        assert!(!scope.is_platform());
        assert_eq!(scope.tenant_id(), Some(&id));
    }

    #[test]
    fn test_tenant_state_active() {
        let state = TenantState::Active;
        assert_eq!(state, TenantState::Active);
        assert_ne!(state, TenantState::Suspended);
    }
}
