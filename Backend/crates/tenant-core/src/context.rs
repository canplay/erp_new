//! 租户上下文管理
//!
//! 使用 `std::thread::LocalKey` 实现线程安全的租户上下文传播。
//!
//! # 使用方式
//!
//! ```rust,ignore
//! use tenant_core::context::{set_tenant_context, current_tenant_id, TenantContext};
//!
//! // 设置租户上下文
//! let ctx = TenantContext::new(tenant_id!(1)).with_user_id(42);
//! set_tenant_context(ctx);
//!
//! // 获取当前租户 ID
//! if let Some(id) = current_tenant_id() {
//!     println!("当前租户: {}" , id);
//! }
//! ```

use crate::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::sync::Arc;

/// 租户上下文
///
/// 携带当前请求的租户信息，通过 thread-local 在调用链中传播。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub user_id: Option<i64>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl TenantContext {
    /// 创建新的租户上下文
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            user_id: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// 设置用户 ID
    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 设置角色列表
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// 设置权限列表
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// 检查是否包含指定角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 检查是否包含指定权限
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// 判断是否为平台管理员（root）
    pub fn is_platform_admin(&self) -> bool {
        self.has_role("platform_admin" ) || self.has_role("root" ) || self.has_permission("platform" )
    }
}

// 线程局部租户上下文存储
thread_local! {
    static TENANT_CTX: RefCell<Option<Arc<TenantContext>>> = const { RefCell::new(None) };
}

/// 设置当前线程的租户上下文
pub fn set_tenant_context(ctx: TenantContext) {
    TENANT_CTX.with(|cell| {
        *cell.borrow_mut() = Some(Arc::new(ctx));
    });
}

/// 设置当前线程的租户上下文（Arc 版本）
pub fn set_tenant_context_arc(ctx: Arc<TenantContext>) {
    TENANT_CTX.with(|cell| {
        *cell.borrow_mut() = Some(ctx);
    });
}

/// 获取当前租户 ID
pub fn current_tenant_id() -> Option<TenantId> {
    TENANT_CTX.with(|cell| {
        cell.borrow().as_ref().map(|ctx| ctx.tenant_id)
    })
}

/// 获取当前用户 ID
pub fn current_user_id() -> Option<i64> {
    TENANT_CTX.with(|cell| {
        cell.borrow().as_ref().and_then(|ctx| ctx.user_id)
    })
}

/// 检查当前用户是否为平台管理员
pub fn is_platform_admin() -> bool {
    TENANT_CTX.with(|cell| {
        cell.borrow()
            .as_ref()
            .map_or(false, |ctx| ctx.is_platform_admin())
    })
}

/// 获取当前租户上下文的 Arc 引用
pub fn current_tenant_context() -> Option<Arc<TenantContext>> {
    TENANT_CTX.with(|cell| {
        cell.borrow().clone()
    })
}

/// 清除当前上下文
pub fn clear_tenant_context() {
    TENANT_CTX.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// 在指定租户上下文中运行异步任务
pub async fn with_tenant_context<F, Fut, R>(ctx: TenantContext, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    // 注意：由于使用 thread-local，这里只是设置后执行，不保证跨 .await 传播
    // 对于真正的异步传播，需要使用 tokio::task_local!
    set_tenant_context(ctx);
    f().await
}

/// 在当前租户上下文中运行异步任务（Arc 版本）
pub async fn with_tenant_context_arc<F, Fut, R>(ctx: Arc<TenantContext>, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    set_tenant_context_arc(ctx);
    f().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_context_new() {
        let ctx = TenantContext::new(TenantId::new(1));
        assert_eq!(ctx.tenant_id.value(), 1);
        assert!(ctx.user_id.is_none());
        assert!(ctx.roles.is_empty());
    }

    #[test]
    fn test_tenant_context_with_user() {
        let ctx = TenantContext::new(TenantId::new(1))
            .with_user_id(42)
            .with_roles(vec!["admin".to_string()]);
        assert_eq!(ctx.user_id, Some(42));
        assert!(ctx.has_role("admin" ));
        assert!(!ctx.has_role("user" ));
    }

    #[test]
    fn test_tenant_context_permissions() {
        let ctx = TenantContext::new(TenantId::new(1))
            .with_permissions(vec!["tenant:read".to_string(), "tenant:write".to_string()]);
        assert!(ctx.has_permission("tenant:read" ));
        assert!(!ctx.has_permission("tenant:delete" ));
    }

    #[test]
    fn test_is_platform_admin() {
        let ctx = TenantContext::new(TenantId::new(0))
            .with_roles(vec!["platform_admin".to_string()]);
        assert!(ctx.is_platform_admin());

        let ctx = TenantContext::new(TenantId::new(0))
            .with_roles(vec!["admin".to_string()]);
        assert!(!ctx.is_platform_admin());
    }

    #[test]
    fn test_tenant_context_serialization() {
        let ctx = TenantContext::new(TenantId::new(1))
            .with_user_id(42)
            .with_roles(vec!["admin".to_string()]);

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: TenantContext = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tenant_id.value(), 1);
        assert_eq!(deserialized.user_id, Some(42));
        assert_eq!(deserialized.roles, vec!["admin".to_string()]);
    }

    #[test]
    fn test_set_and_get_tenant_context() {
        clear_tenant_context();
        assert!(current_tenant_id().is_none());

        let ctx = TenantContext::new(TenantId::new(42))
            .with_user_id(100)
            .with_roles(vec!["platform_admin".to_string()]);
        set_tenant_context(ctx);

        assert_eq!(current_tenant_id(), Some(TenantId::new(42)));
        assert_eq!(current_user_id(), Some(100));
        assert!(is_platform_admin());

        clear_tenant_context();
        assert!(current_tenant_id().is_none());
    }

    #[test]
    fn test_set_tenant_context_arc() {
        clear_tenant_context();

        let ctx = TenantContext::new(TenantId::new(99));
        let arc_ctx = Arc::new(ctx);
        set_tenant_context_arc(arc_ctx.clone());

        let fetched = current_tenant_context();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().tenant_id.value(), 99);

        clear_tenant_context();
    }
}
