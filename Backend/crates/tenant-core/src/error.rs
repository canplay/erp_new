//! 租户错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("Tenant not found: {0}")]
    NotFound(String),

    #[error("Tenant is not active: {0}")]
    NotActive(String),

    #[error("Tenant has expired: {0}")]
    Expired(String),

    #[error("Tenant quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Tenant access denied: {0}")]
    AccessDenied(String),

    #[error("Invalid tenant state transition: from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Tenant provisioning failed: {0}")]
    ProvisioningFailed(String),

    #[error("Tenant already exists: {0}")]
    AlreadyExists(String),

    #[error("Tenant database error: {0}")]
    DatabaseError(String),

    #[error("Tenant configuration error: {0}")]
    ConfigurationError(String),
}

pub type TenantResult<T> = Result<T, TenantError>;
