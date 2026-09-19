//! Tenant Offboarding Workflow
//!
//! Implements the multi-step tenant offboarding process:
//! 1. Export tenant data (GDPR-compliant)
//! 2. Anonymize user data
//! 3. Drop tenant schema
//! 4. Send farewell notification
//!
//! Supports compensating transactions for rollback on failure.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::repository::TenantRepository;

/// Offboarding error types
#[derive(Error, Debug)]
pub enum OffboardingError {
    #[error("Data export failed: {0}" )]
    DataExport(String),

    #[error("User anonymization failed: {0}" )]
    UserAnonymization(String),

    #[error("Schema drop failed: {0}" )]
    SchemaDrop(String),

    #[error("Notification failed: {0}" )]
    Notification(String),

    #[error("Tenant not found: {0}" )]
    TenantNotFound(i64),

    #[error("Rollback failed: {0}" )]
    Rollback(String),
}

impl From<OffboardingError> for common::AppError {
    fn from(err: OffboardingError) -> Self {
        match err {
            OffboardingError::DataExport(msg) => Self::TenantOffboarding(msg),
            OffboardingError::UserAnonymization(msg) => Self::TenantOffboarding(msg),
            OffboardingError::SchemaDrop(msg) => Self::TenantOffboarding(msg),
            OffboardingError::Notification(msg) => Self::TenantOffboarding(msg),
            OffboardingError::TenantNotFound(_) => Self::TenantNotFound,
            OffboardingError::Rollback(msg) => Self::TenantOffboarding(msg),
        }
    }
}

/// Result type for offboarding operations
pub type OffboardingResult<T> = Result<T, OffboardingError>;

/// Tracks the state of each offboarding step for potential rollback
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OffboardingStep {
    /// Step 1: Data exported
    DataExported(String), // export path
    /// Step 2: Users anonymized
    UsersAnonymized(i64), // count
    /// Step 3: Schema dropped
    SchemaDropped(String), // schema name
    /// Step 4: Farewell notification sent
    NotificationSent,
    /// Offboarding complete
    Complete,
}

/// Offboarding request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffboardingRequest {
    pub tenant_id: i64,
    pub reason: Option<String>,
    pub requested_by: String,
    pub export_format: DataExportFormat,
    pub retain_days: Option<u32>, // Days to retain data before permanent deletion
}

impl Default for OffboardingRequest {
    fn default() -> Self {
        Self {
            tenant_id: 0,
            reason: None,
            requested_by: String::new(),
            export_format: DataExportFormat::Json,
            retain_days: Some(30),
        }
    }
}

/// Data export format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataExportFormat {
    Json,
    Csv,
    Xml,
}

/// Offboarding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffboardingResponse {
    pub tenant_id: i64,
    pub export_path: Option<String>,
    pub users_anonymized: i64,
    pub steps_completed: Vec<OffboardingStep>,
    pub status: OffboardingStatus,
    pub completed_at: Option<chrono::DateTime<Utc>>,
}

/// Offboarding status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OffboardingStatus {
    InProgress,
    DataExported,
    UsersAnonymized,
    SchemaDropped,
    Completed,
    Failed(String),
    RolledBack,
}

/// GDPR-compliant data export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDataExport {
    pub tenant_id: i64,
    pub tenant_code: String,
    pub exported_at: chrono::DateTime<Utc>,
    pub export_format: DataExportFormat,
    pub users: Vec<UserExport>,
    pub metadata: HashMap<String, String>,
}

/// User data for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserExport {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub department: Option<String>,
    pub created_at: String,
    pub last_login: Option<String>,
}

/// Offboarding service with step tracking and rollback support
pub struct OffboardingService {
    repository: TenantRepository,
}

impl OffboardingService {
    /// Create a new offboarding service
    #[must_use]
    pub const fn new(repository: TenantRepository) -> Self {
        Self { repository }
    }

    /// Execute the full offboarding workflow
    ///
    /// Steps:
    /// 1. Export tenant data (GDPR-compliant)
    /// 2. Anonymize user data
    /// 3. Drop tenant schema
    /// 4. Send farewell notification
    pub async fn offboard(
        &self,
        request: OffboardingRequest,
    ) -> OffboardingResult<OffboardingResponse> {
        let mut steps: Vec<OffboardingStep> = Vec::new();
        let mut response = OffboardingResponse {
            tenant_id: request.tenant_id,
            export_path: None,
            users_anonymized: 0,
            steps_completed: vec![],
            status: OffboardingStatus::InProgress,
            completed_at: None,
        };

        // Step 1: Export tenant data
        let export_path = self.export_data(request.tenant_id, &request.export_format).await?;
        steps.push(OffboardingStep::DataExported(export_path.clone()));
        response.export_path = Some(export_path.clone());
        response.status = OffboardingStatus::DataExported;

        // Step 2: Anonymize user data
        let anonymized_count = self.anonymize_users(request.tenant_id).await?;
        steps.push(OffboardingStep::UsersAnonymized(anonymized_count));
        response.users_anonymized = anonymized_count;
        response.status = OffboardingStatus::UsersAnonymized;

        // Step 3: Drop tenant schema
        let schema_name = self.drop_schema(request.tenant_id).await?;
        steps.push(OffboardingStep::SchemaDropped(schema_name));
        response.status = OffboardingStatus::SchemaDropped;

        // Step 4: Send farewell notification
        self.send_farewell_notification(request.tenant_id, &request.requested_by).await?;
        steps.push(OffboardingStep::NotificationSent);

        response.steps_completed = steps;
        response.status = OffboardingStatus::Completed;
        response.completed_at = Some(Utc::now());

        Ok(response)
    }

    /// Step 1: Export tenant data (GDPR-compliant)
    async fn export_data(
        &self,
        tenant_id: i64,
        format: &DataExportFormat,
    ) -> OffboardingResult<String> {
        tracing::info!(
            "Exporting data for tenant {} in {:?} format" ,
            tenant_id,
            format
        );

        // In a real implementation, this would:
        // 1. Query all tenant data from database
        // 2. Serialize to the requested format
        // 3. Store in a secure location (e.g., encrypted S3 bucket)
        // 4. Generate a download link with expiration

        let export_path = format!(
            "/exports/tenant_{}/data_{}.{}" ,
            tenant_id,
            Utc::now().timestamp(),
            match format {
                DataExportFormat::Json => "json" ,
                DataExportFormat::Csv => "csv" ,
                DataExportFormat::Xml => "xml" ,
            }
        );

        // Simulate data export
        let _ = format;

        Ok(export_path)
    }

    /// Step 2: Anonymize user data
    async fn anonymize_users(&self, tenant_id: i64) -> OffboardingResult<i64> {
        tracing::info!("Anonymizing users for tenant {}" , tenant_id);

        // In a real implementation, this would:
        // 1. Replace PII with anonymized values
        // 2. Keep non-PII data for analytics
        // 3. Log the anonymization for audit

        // Simulate anonymization - return count of affected users
        let anonymized_count = 10i64; // Placeholder

        Ok(anonymized_count)
    }

    /// Step 3: Drop tenant schema
    async fn drop_schema(&self, tenant_id: i64) -> OffboardingResult<String> {
        tracing::info!("Dropping schema for tenant {}" , tenant_id);

        // In a real implementation, this would:
        // 1. Find the tenant's schema name
        // 2. DROP SCHEMA IF EXISTS schema_name CASCADE
        // 3. Clean up related resources

        // FIX [SQL-INJ-008]: 验证 tenant_id 格式（必须是正整数）
        // schema 名格式：tenant_tenant_{tenant_id}
        // tenant_id 是 i64 类型，不存在 SQL 注入风险
        let schema_name = format!("tenant_tenant_{}" , tenant_id);

        // 验证生成的 schema 名格式正确
        common::utils::sanitize_schema_name(&schema_name)
            .map_err(|e| OffboardingError::SchemaDrop(e))?;

        Ok(schema_name)
    }

    /// Step 4: Send farewell notification
    async fn send_farewell_notification(
        &self,
        tenant_id: i64,
        requested_by: &str,
    ) -> OffboardingResult<()> {
        tracing::info!(
            "Sending farewell notification for tenant {} (requested by {})" ,
            tenant_id,
            requested_by
        );

        // In a real implementation, this would:
        // 1. Call messaging-service gRPC to send notification
        // 2. Send email to tenant admin
        // 3. Log the offboarding event

        Ok(())
    }

    /// Rollback offboarding on failure (compensating transactions)
    ///
    /// Reverses completed steps in reverse order.
    pub async fn rollback(
        &self,
        steps: &[OffboardingStep],
    ) -> OffboardingResult<()> {
        for step in steps.iter().rev() {
            match step {
                OffboardingStep::NotificationSent => {
                    // Notification sent - no rollback needed
                    tracing::info!("Rolling back: notification (no-op)" );
                }
                OffboardingStep::SchemaDropped(schema_name) => {
                    // Rollback schema drop - recreate schema
                    tracing::info!("Rolling back: schema {}" , schema_name);
                    // In production: recreate schema from backup
                }
                OffboardingStep::UsersAnonymized(count) => {
                    // Rollback anonymization - restore from backup
                    tracing::info!("Rolling back: {} anonymized users" , count);
                    // In production: restore from pre-anonymization backup
                }
                OffboardingStep::DataExported(path) => {
                    // Data exported - no rollback needed (export is read-only)
                    tracing::info!("Rolling back: data export {} (no-op)" , path);
                }
                OffboardingStep::Complete => {}
            }
        }
        Ok(())
    }

    /// Execute offboarding with automatic rollback on failure
    pub async fn offboard_with_rollback(
        &self,
        request: OffboardingRequest,
    ) -> OffboardingResult<OffboardingResponse> {
        let mut steps: Vec<OffboardingStep> = Vec::new();

        // Step 1: Export tenant data
        let export_path = match self.export_data(request.tenant_id, &request.export_format).await {
            Ok(path) => {
                steps.push(OffboardingStep::DataExported(path.clone()));
                path
            }
            Err(e) => return Err(e),
        };

        // Step 2: Anonymize user data
        let anonymized_count = match self.anonymize_users(request.tenant_id).await {
            Ok(count) => {
                steps.push(OffboardingStep::UsersAnonymized(count));
                count
            }
            Err(e) => {
                self.rollback(&steps).await?;
                return Err(e);
            }
        };

        // Step 3: Drop tenant schema
        let schema_name = match self.drop_schema(request.tenant_id).await {
            Ok(name) => {
                steps.push(OffboardingStep::SchemaDropped(name.clone()));
                name
            }
            Err(e) => {
                self.rollback(&steps).await?;
                return Err(e);
            }
        };
        let _ = schema_name; // Used for potential rollback logging

        // Step 4: Send farewell notification
        if let Err(e) = self.send_farewell_notification(request.tenant_id, &request.requested_by).await
        {
            self.rollback(&steps).await?;
            return Err(e);
        }
        steps.push(OffboardingStep::NotificationSent);

        Ok(OffboardingResponse {
            tenant_id: request.tenant_id,
            export_path: Some(export_path),
            users_anonymized: anonymized_count,
            steps_completed: steps,
            status: OffboardingStatus::Completed,
            completed_at: Some(Utc::now()),
        })
    }

    /// Get offboarding status for a tenant
    pub async fn get_status(
        &self,
        tenant_id: i64,
    ) -> OffboardingResult<Option<OffboardingStatus>> {
        match self.repository.find_by_id(tenant_id).await {
            Ok(Some(_)) => Ok(Some(OffboardingStatus::Completed)),
            Ok(None) => Ok(None),
            Err(e) => Err(OffboardingError::DataExport(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offboarding_request_default() {
        let req = OffboardingRequest::default();
        assert_eq!(req.retain_days, Some(30));
        assert_eq!(req.export_format, DataExportFormat::Json);
    }

    #[test]
    fn test_offboarding_step_serialization() {
        let step = OffboardingStep::DataExported("/path/to/export.json".to_string());
        let json = serde_json::to_string(&step).expect("test assertion" );
        assert!(json.contains("export.json" ));
    }

    #[test]
    fn test_offboarding_status_serialization() {
        let status = OffboardingStatus::Completed;
        let json = serde_json::to_string(&status).expect("test assertion" );
        assert!(json.contains("Completed" ));
    }

    #[test]
    fn test_data_export_format_serialization() {
        let format = DataExportFormat::Csv;
        let json = serde_json::to_string(&format).expect("test assertion" );
        assert!(json.contains("Csv" ));
    }

    #[test]
    fn test_tenant_data_export_creation() {
        let export = TenantDataExport {
            tenant_id: 1,
            tenant_code: "test".to_string(),
            exported_at: Utc::now(),
            export_format: DataExportFormat::Json,
            users: vec![],
            metadata: HashMap::new(),
        };

        assert_eq!(export.tenant_id, 1);
        assert_eq!(export.export_format, DataExportFormat::Json);
    }
}
