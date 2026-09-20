//! 租户生命周期管理服务
//!
//! 管理租户从创建到销毁的完整生命周期。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};


use tenant_core::{
    context::TenantContext,
    lifecycle::{LifecycleManager, LifecycleState},
    provisioning::ProvisioningConfig,
    TenantId, TenantResult, TenantError,
};

/// 租户生命周期服务
#[derive(Clone)]
pub struct TenantLifecycleService {
    pool: PgPool,
    lifecycle_manager: LifecycleManager,
    provisioning_config: ProvisioningConfig,
}

impl TenantLifecycleService {
    /// 创建新的租户生命周期服务
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            lifecycle_manager: LifecycleManager::default(),
            provisioning_config: ProvisioningConfig::default(),
        }
    }

    /// 获取数据库连接池引用
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 创建租户
    pub async fn create_tenant(
        &self,
        name: &str,
        code: &str,
        _admin_email: &str,
    ) -> TenantResult<TenantContext> {
        let tenant_id = TenantId::new(
            sqlx::query_scalar::<_, i64>(
                "INSERT INTO tenants (name, code, lifecycle_state, created_at, updated_at) 
                 VALUES ($1, $2, 'provisioning', now(), now()) RETURNING id"
            )
            .bind(name)
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?
        );

        // 创建默认设置
        sqlx::query(
            "INSERT INTO tenant_settings (tenant_id, theme, settings, features) 
             VALUES ($1, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb)"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // 创建默认订阅（试用）— v2 计费表（subscription_plans / subscriptions）
        sqlx::query(
            "INSERT INTO subscriptions (tenant_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end)
             SELECT $1, p.id, 'trialing', now(), now() + interval '14 days', now(), now() + interval '14 days'
             FROM subscription_plans p WHERE p.id = 'free' AND p.active = true LIMIT 1"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // 记录生命周期事件
        sqlx::query(
            "INSERT INTO tenant_lifecycle_events (tenant_id, event_type, to_state, reason)
             VALUES ($1, 'tenant_created', 'provisioning', 'Tenant created')"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        info!(tenant_id = %tenant_id, name = %name, "Tenant created" );

        Ok(TenantContext::new(tenant_id))
    }

    /// 激活租户
    pub async fn activate_tenant(&self, tenant_id: TenantId) -> TenantResult<()> {
        let current_state: String = sqlx::query_scalar(
            "SELECT lifecycle_state FROM tenants WHERE id = $1"
        )
        .bind(tenant_id.value())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        let current = LifecycleState::from_str(&current_state)
            .ok_or_else(|| TenantError::InvalidStateTransition { 
                from: current_state.clone(), 
                to: "active".to_string() 
            })?;

        if !current.can_transition_to(&LifecycleState::Active) {
            return Err(TenantError::InvalidStateTransition {
                from: current_state,
                to: "active".to_string(),
            });
        }

        sqlx::query(
            "UPDATE tenants SET lifecycle_state = 'active', provisioned_at = now(), updated_at = now() WHERE id = $1"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // 记录事件
        sqlx::query(
            "INSERT INTO tenant_lifecycle_events (tenant_id, event_type, from_state, to_state, reason)
             VALUES ($1, 'tenant_activated', $2, 'active', 'Tenant activated')"
        )
        .bind(tenant_id.value())
        .bind(&current_state)
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        info!(tenant_id = %tenant_id, "Tenant activated" );

        Ok(())
    }

    /// 暂停租户
    pub async fn suspend_tenant(&self, tenant_id: TenantId, reason: &str) -> TenantResult<()> {
        let current_state: String = sqlx::query_scalar(
            "SELECT lifecycle_state FROM tenants WHERE id = $1"
        )
        .bind(tenant_id.value())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        let current = LifecycleState::from_str(&current_state)
            .ok_or_else(|| TenantError::InvalidStateTransition { 
                from: current_state.clone(), 
                to: "suspended".to_string() 
            })?;

        if !current.can_transition_to(&LifecycleState::Suspended) {
            return Err(TenantError::InvalidStateTransition {
                from: current_state,
                to: "suspended".to_string(),
            });
        }

        sqlx::query(
            "UPDATE tenants SET lifecycle_state = 'suspended', updated_at = now() WHERE id = $1"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO tenant_lifecycle_events (tenant_id, event_type, from_state, to_state, reason)
             VALUES ($1, 'tenant_suspended', $2, 'suspended', $3)"
        )
        .bind(tenant_id.value())
        .bind(&current_state)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        warn!(tenant_id = %tenant_id, reason = %reason, "Tenant suspended" );

        Ok(())
    }

    /// 续期租户
    pub async fn renew_tenant(&self, tenant_id: TenantId, days: i64) -> TenantResult<()> {
        sqlx::query(
            "UPDATE tenants SET 
                lifecycle_state = 'active', 
                expires_at = now() + make_interval(days => $2),
                updated_at = now() 
             WHERE id = $1"
        )
        .bind(tenant_id.value())
        .bind(days)
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO tenant_lifecycle_events (tenant_id, event_type, to_state, reason)
             VALUES ($1, 'tenant_renewed', 'active', $2)"
        )
        .bind(tenant_id.value())
        .bind(format!("Renewed for {} days" , days))
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        info!(tenant_id = %tenant_id, days = days, "Tenant renewed" );

        Ok(())
    }

    /// 删除租户
    pub async fn delete_tenant(&self, tenant_id: TenantId) -> TenantResult<()> {
        sqlx::query(
            "UPDATE tenants SET lifecycle_state = 'deleted', updated_at = now() WHERE id = $1"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO tenant_lifecycle_events (tenant_id, event_type, to_state, reason)
             VALUES ($1, 'tenant_deleted', 'deleted', 'Tenant deleted')"
        )
        .bind(tenant_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        info!(tenant_id = %tenant_id, "Tenant deleted" );

        Ok(())
    }

    /// 获取租户状态
    pub async fn get_tenant_state(&self, tenant_id: TenantId) -> TenantResult<TenantState> {
        let state: String = sqlx::query_scalar(
            "SELECT lifecycle_state FROM tenants WHERE id = $1"
        )
        .bind(tenant_id.value())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        let lifecycle_state = LifecycleState::from_str(&state)
            .ok_or_else(|| TenantError::ConfigurationError(format!("Unknown state: {}" , state)))?;

        Ok(TenantState {
            tenant_id,
            state: lifecycle_state,
            expires_at: self.get_expiry(tenant_id).await?,
        })
    }

    /// 获取租户过期时间
    async fn get_expiry(&self, tenant_id: TenantId) -> TenantResult<Option<DateTime<Utc>>> {
        sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            "SELECT expires_at FROM tenants WHERE id = $1"
        )
        .bind(tenant_id.value())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))
    }

    /// 检查租户是否活跃
    pub async fn is_tenant_active(&self, tenant_id: TenantId) -> bool {
        self.get_tenant_state(tenant_id)
            .await
            .map(|s| s.state.is_operational())
            .unwrap_or(false)
    }

    /// 获取即将过期的租户
    pub async fn get_expiring_tenants(&self, within_days: i64) -> TenantResult<Vec<TenantId>> {
        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM tenants 
             WHERE lifecycle_state IN ('active', 'trial')
             AND expires_at IS NOT NULL 
             AND expires_at <= now() + make_interval(days => $1)
             ORDER BY expires_at ASC"
        )
        .bind(within_days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| TenantId::new(id)).collect())
    }
}

/// 租户状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantState {
    pub tenant_id: TenantId,
    pub state: LifecycleState,
    pub expires_at: Option<DateTime<Utc>>,
}
