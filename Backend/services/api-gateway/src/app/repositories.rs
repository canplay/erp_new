//! Repository (data store) collection
//!
//! Groups all eight in-memory repository types that
//! live inside `AppState`.  Each repository wraps a
//! `PgPool` + an in-memory cache (`Arc<Mutex<Vec<T>>>`).
//!
//! The original `AppState` had 10 repository fields;
//! they are now bundled here so that the
//! top-level struct only has one `repositories` field.

use std::sync::Arc;

use tokio::sync::RwLock as AsyncRwLock;

use crate::repository::{
    DataSourceRepository, DeviceRepository, IpWhitelistRepository,
    ReportRepository, ReportTemplateRepository, SensitiveAuditRepository,
    ScheduledTaskRepository,
};

/// All repository stores bundled together.
///
/// Replaces the ten individual `Arc<AsyncRwLock<...>>`
/// fields that used to live directly in `AppState`.
#[derive(Debug, Clone)]
pub struct Repositories {
    /// Device registration records.
    pub device_store: Arc<AsyncRwLock<DeviceRepository>>,

    /// IP whitelist entries.
    pub ip_whitelist_store: Arc<AsyncRwLock<IpWhitelistRepository>>,

    /// Sensitive audit log entries.
    pub sensitive_audit_store: Arc<AsyncRwLock<SensitiveAuditRepository>>,

    /// Scheduled task definitions.
    pub scheduled_task_store: Arc<AsyncRwLock<ScheduledTaskRepository>>,

    /// Report definitions.
    pub report_store: Arc<AsyncRwLock<ReportRepository>>,

    /// Data source configurations.
    pub data_source_store: Arc<AsyncRwLock<DataSourceRepository>>,

    /// Report template definitions.
    pub report_template_store: Arc<AsyncRwLock<ReportTemplateRepository>>,
}

impl Repositories {
    /// Build all repositories from a `PgPool`.
    pub async fn new(pool: sqlx::PgPool) -> Self {
        Self {
            device_store: Arc::new(AsyncRwLock::new(DeviceRepository::new(pool.clone()).await)),
            ip_whitelist_store: Arc::new(AsyncRwLock::new(IpWhitelistRepository::new(pool.clone()).await)),
            sensitive_audit_store: Arc::new(AsyncRwLock::new(SensitiveAuditRepository::new(pool.clone()).await)),
            scheduled_task_store: Arc::new(AsyncRwLock::new(ScheduledTaskRepository::new(pool.clone()).await)),
            report_store: Arc::new(AsyncRwLock::new(ReportRepository::new(pool.clone()).await)),
            data_source_store: Arc::new(AsyncRwLock::new(DataSourceRepository::new(pool.clone()).await)),
            report_template_store: Arc::new(AsyncRwLock::new(ReportTemplateRepository::new(pool.clone()).await)),
        }
    }
}
