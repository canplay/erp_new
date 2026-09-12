//! Tenant Onboarding Workflow
//!
//! Implements the multi-step tenant onboarding process:
//! 1. Create tenant record
//! 2. Provision tenant schema (if schema-based isolation)
//! 3. Create default admin user
//! 4. Send welcome notification
//!
//! Supports compensating transactions for rollback on failure.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::repository::TenantRepository;

/// Onboarding error types
#[derive(Error, Debug)]
pub enum OnboardingError {
    #[error("Tenant creation failed: {0}")]
    TenantCreation(String),

    #[error("Schema provisioning failed: {0}")]
    SchemaProvisioning(String),

    #[error("Admin user creation failed: {0}")]
    AdminUserCreation(String),

    #[error("Notification failed: {0}")]
    Notification(String),

    #[error("Rollback failed: {0}")]
    Rollback(String),
}

/// Result type for onboarding operations
pub type OnboardingResult<T> = Result<T, OnboardingError>;

/// Tracks the state of each onboarding step for potential rollback
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingStep {
    /// Step 1: Tenant record created
    TenantCreated(i64),
    /// Step 2: Schema provisioned
    SchemaProvisioned(String),
    /// Step 3: Admin user created
    AdminUserCreated(i64),
    /// Step 4: Welcome notification sent
    NotificationSent,
    /// Onboarding complete
    Complete,
}

/// Onboarding request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingRequest {
    pub tenant_name: String,
    pub tenant_code: String,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub max_users: i32,
    pub max_storage: i64,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password_hash: String,
}

impl Default for OnboardingRequest {
    fn default() -> Self {
        Self {
            tenant_name: String::new(),
            tenant_code: String::new(),
            domain: None,
            description: None,
            max_users: 50,
            max_storage: 100 * 1024 * 1024 * 1024, // 100GB
            admin_username: String::new(),
            admin_email: String::new(),
            admin_password_hash: String::new(),
        }
    }
}

/// Onboarding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResponse {
    pub tenant_id: i64,
    pub tenant_code: String,
    pub admin_user_id: i64,
    pub steps_completed: Vec<OnboardingStep>,
    pub status: OnboardingStatus,
}

/// Onboarding status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingStatus {
    InProgress,
    Completed,
    Failed(String),
    RolledBack,
}

/// Onboarding service with step tracking and rollback support
pub struct OnboardingService {
    repository: TenantRepository,
}

impl OnboardingService {
    /// Create a new onboarding service
    #[must_use]
    pub const fn new(repository: TenantRepository) -> Self {
        Self { repository }
    }

    /// Execute the full onboarding workflow
    ///
    /// Steps:
    /// 1. Create tenant record
    /// 2. Provision tenant schema
    /// 3. Create default admin user
    /// 4. Send welcome notification
    pub async fn onboard(
        &self,
        request: OnboardingRequest,
    ) -> OnboardingResult<OnboardingResponse> {
        let mut steps: Vec<OnboardingStep> = Vec::new();

        // Step 1: Create tenant record
        let tenant_id = self.create_tenant_record(&request).await?;
        steps.push(OnboardingStep::TenantCreated(tenant_id));

        // Step 2: Provision tenant schema
        let schema_name = self.provision_schema(tenant_id, &request.tenant_code).await?;
        steps.push(OnboardingStep::SchemaProvisioned(schema_name));

        // Step 3: Create default admin user
        let admin_user_id = self.create_admin_user(tenant_id, &request).await?;
        steps.push(OnboardingStep::AdminUserCreated(admin_user_id));

        // Step 4: Send welcome notification
        self.send_welcome_notification(tenant_id, &request.admin_email).await?;
        steps.push(OnboardingStep::NotificationSent);

        Ok(OnboardingResponse {
            tenant_id,
            tenant_code: request.tenant_code,
            admin_user_id,
            steps_completed: steps,
            status: OnboardingStatus::Completed,
        })
    }

    /// Step 1: Create tenant record
    async fn create_tenant_record(
        &self,
        request: &OnboardingRequest,
    ) -> OnboardingResult<i64> {
        self.repository
            .create(
                &request.tenant_name,
                &request.tenant_code,
                request.domain.as_deref(),
                request.description.as_deref(),
                request.max_users,
                request.max_storage,
            )
            .await
            .map_err(|e| OnboardingError::TenantCreation(e.to_string()))
    }

    /// Step 2: Provision tenant schema
    ///
    /// For schema-based isolation, creates a new schema with the tenant code.
    /// For row-level isolation, this is a no-op.
    async fn provision_schema(
        &self,
        tenant_id: i64,
        tenant_code: &str,
    ) -> OnboardingResult<String> {
        // Generate schema name from tenant code
        let schema_name = format!("tenant_{tenant_code}");

        // In a real implementation, this would:
        // 1. Create the schema in the database
        // 2. Run migrations for the new schema
        // 3. Set up row-level security policies

        // For now, we simulate schema provisioning
        tracing::info!(
            "Provisioning schema '{}' for tenant {} (ID: {})",
            schema_name,
            tenant_code,
            tenant_id
        );

        // Simulate schema creation (in production, execute SQL)
        // sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name))
        //     .execute(&self.repository.pool())
        //     .await
        //     .map_err(|e| OnboardingError::SchemaProvisioning(e.to_string()))?;

        Ok(schema_name)
    }

    /// Step 3: Create default admin user
    async fn create_admin_user(
        &self,
        tenant_id: i64,
        request: &OnboardingRequest,
    ) -> OnboardingResult<i64> {
        // In a real implementation, this would:
        // 1. Call user-service gRPC to create the admin user
        // 2. Hash the password
        // 3. Assign admin role
        // 4. Link user to tenant via tenant_users table

        tracing::info!(
            "Creating admin user '{}' for tenant {} (ID: {})",
            request.admin_username,
            request.tenant_code,
            tenant_id
        );

        // Simulate admin user creation
        // In production, this would call user-service gRPC
        let admin_user_id = Uuid::new_v4().as_u128() as i64;

        Ok(admin_user_id)
    }

    /// Step 4: Send welcome notification
    async fn send_welcome_notification(
        &self,
        tenant_id: i64,
        admin_email: &str,
    ) -> OnboardingResult<()> {
        // In a real implementation, this would:
        // 1. Call messaging-service gRPC to send a welcome message
        // 2. Send email via email provider
        tracing::info!(
            "Sending welcome notification for tenant {} to {}",
            tenant_id,
            admin_email
        );

        Ok(())
    }

    /// Rollback onboarding on failure (compensating transactions)
    ///
    /// Reverses completed steps in reverse order.
    pub async fn rollback(
        &self,
        steps: &[OnboardingStep],
    ) -> OnboardingResult<()> {
        for step in steps.iter().rev() {
            match step {
                OnboardingStep::NotificationSent => {
                    // Notification sent - no rollback needed (idempotent)
                    tracing::info!("Rolling back: notification (no-op)");
                }
                OnboardingStep::AdminUserCreated(user_id) => {
                    // Rollback admin user creation
                    tracing::info!("Rolling back: admin user {}", user_id);
                    // In production: call user-service to delete user
                }
                OnboardingStep::SchemaProvisioned(schema_name) => {
                    // Rollback schema provisioning
                    tracing::info!("Rolling back: schema {}", schema_name);
                    // In production: DROP SCHEMA IF EXISTS schema_name CASCADE
                    let _ = schema_name;
                }
                OnboardingStep::TenantCreated(tenant_id) => {
                    // Rollback tenant creation
                    tracing::info!("Rolling back: tenant {}", tenant_id);
                    self.repository
                        .delete(*tenant_id)
                        .await
                        .map_err(|e| OnboardingError::Rollback(e.to_string()))?;
                }
                OnboardingStep::Complete => {}
            }
        }
        Ok(())
    }

    /// Execute onboarding with automatic rollback on failure
    pub async fn onboard_with_rollback(
        &self,
        request: OnboardingRequest,
    ) -> OnboardingResult<OnboardingResponse> {
        let mut steps: Vec<OnboardingStep> = Vec::new();

        // Step 1: Create tenant record
        let tenant_id = match self.create_tenant_record(&request).await {
            Ok(id) => {
                steps.push(OnboardingStep::TenantCreated(id));
                id
            }
            Err(e) => return Err(e),
        };

        // Step 2: Provision tenant schema
        let schema_name = match self.provision_schema(tenant_id, &request.tenant_code).await {
            Ok(name) => {
                steps.push(OnboardingStep::SchemaProvisioned(name.clone()));
                name
            }
            Err(e) => {
                self.rollback(&steps).await?;
                return Err(e);
            }
        };
        let _ = schema_name; // Used for potential rollback logging

        // Step 3: Create default admin user
        let admin_user_id = match self.create_admin_user(tenant_id, &request).await {
            Ok(id) => {
                steps.push(OnboardingStep::AdminUserCreated(id));
                id
            }
            Err(e) => {
                self.rollback(&steps).await?;
                return Err(e);
            }
        };

        // Step 4: Send welcome notification
        if let Err(e) = self.send_welcome_notification(tenant_id, &request.admin_email).await {
            self.rollback(&steps).await?;
            return Err(e);
        }
        steps.push(OnboardingStep::NotificationSent);

        Ok(OnboardingResponse {
            tenant_id,
            tenant_code: request.tenant_code,
            admin_user_id,
            steps_completed: steps,
            status: OnboardingStatus::Completed,
        })
    }

    /// Get onboarding status for a tenant
    pub async fn get_status(
        &self,
        tenant_id: i64,
    ) -> OnboardingResult<Option<OnboardingStatus>> {
        match self.repository.find_by_id(tenant_id).await {
            Ok(Some(_)) => Ok(Some(OnboardingStatus::Completed)),
            Ok(None) => Ok(None),
            Err(e) => Err(OnboardingError::TenantCreation(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_request_default() {
        let req = OnboardingRequest::default();
        assert_eq!(req.max_users, 50);
        assert_eq!(req.max_storage, 100 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_onboarding_step_serialization() {
        let step = OnboardingStep::TenantCreated(42);
        let json = serde_json::to_string(&step).expect("test assertion");
        assert!(json.contains("42"));
    }

    #[test]
    fn test_onboarding_status_serialization() {
        let status = OnboardingStatus::Completed;
        let json = serde_json::to_string(&status).expect("test assertion");
        assert!(json.contains("Completed"));
    }

    #[test]
    fn test_onboarding_response_creation() {
        let response = OnboardingResponse {
            tenant_id: 1,
            tenant_code: "test".to_string(),
            admin_user_id: 100,
            steps_completed: vec![
                OnboardingStep::TenantCreated(1),
                OnboardingStep::SchemaProvisioned("tenant_test".to_string()),
                OnboardingStep::AdminUserCreated(100),
                OnboardingStep::NotificationSent,
            ],
            status: OnboardingStatus::Completed,
        };

        assert_eq!(response.tenant_id, 1);
        assert_eq!(response.steps_completed.len(), 4);
        assert_eq!(response.status, OnboardingStatus::Completed);
    }
}
