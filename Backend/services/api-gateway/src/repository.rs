//! Persistent repository types for api-gateway state management
//!
//! Each repository loads data from DB on construction and persists writes.
//! All use `Arc<Mutex<Vec<T>>>` for thread-safe interior mutability.
//!
//! Tables use the `gw_` prefix (gateway-owned) to avoid clashing with
//! business tables in schema.sql. Queries use runtime SQL (no offline
//! cache required under SQLX_OFFLINE=true).

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};

// ==================== LoginDevice (re-export from device_routes) ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginDevice {
    pub id: i64,
    pub user_id: i64,
    pub device_id: String,
    pub device_type: String,
    pub device_name: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ip_address: String,
    pub is_trusted: bool,
    pub is_active: bool,
    pub created_at: String,
}

// ==================== IpWhitelistEntry ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IpWhitelistEntry {
    pub id: i64,
    pub ip: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

// ==================== SensitiveAuditEntry ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SensitiveAuditEntry {
    pub id: i64,
    pub user_id: i64,
    pub action: String,
    pub resource: String,
    pub detail: Option<String>,
    pub created_at: String,
}

// ==================== ScheduledTaskEntry ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledTaskEntry {
    pub id: i64,
    pub name: String,
    pub cron: String,
    pub handler: String,
    pub is_active: bool,
    pub created_at: String,
}

// ==================== ReportEntry ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportEntry {
    pub id: i64,
    pub name: String,
    pub template_id: Option<i64>,
    pub params: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

// ==================== DataSourceEntry ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataSourceEntry {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub config: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

// ==================== ReportTemplateEntry ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportTemplateEntry {
    pub id: i64,
    pub name: String,
    pub format: String,
    pub config: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

// ==================== DeviceRepository ====================

#[derive(Debug, Clone)]
pub struct DeviceRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<LoginDevice>>>,
}

impl DeviceRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let devices = match sqlx::query_as::<_, LoginDevice>(
            r#"SELECT id, user_id, device_id, device_type, device_name, browser, os, ip_address, is_trusted, is_active, created_at FROM gw_login_devices ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("DeviceRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = devices;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<LoginDevice> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<LoginDevice> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（只读别名）
    #[must_use]
    pub fn devices(&self) -> Vec<LoginDevice> {
        self.entries()
    }

    /// 获取所有条目（可修改别名）
    pub fn devices_mut(&self) -> Vec<LoginDevice> {
        self.entries_mut()
    }

    pub async fn insert_entry(&self, entry: LoginDevice) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_login_devices (id, user_id, device_id, device_type, device_name, browser, os, ip_address, is_trusted, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        )
        .bind(entry.id)
        .bind(entry.user_id)
        .bind(&entry.device_id)
        .bind(&entry.device_type)
        .bind(&entry.device_name)
        .bind(&entry.browser)
        .bind(&entry.os)
        .bind(&entry.ip_address)
        .bind(entry.is_trusted)
        .bind(entry.is_active)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}

// ==================== IpWhitelistRepository ====================

#[derive(Debug, Clone)]
pub struct IpWhitelistRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<IpWhitelistEntry>>>,
}

impl IpWhitelistRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let entries = match sqlx::query_as::<_, IpWhitelistEntry>(
            r#"SELECT id, ip, description, is_active, created_at FROM gw_ip_whitelist ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("IpWhitelistRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = entries;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<IpWhitelistEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<IpWhitelistEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    pub async fn insert_entry(&self, entry: IpWhitelistEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_ip_whitelist (id, ip, description, is_active, created_at) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(entry.id)
        .bind(&entry.ip)
        .bind(&entry.description)
        .bind(entry.is_active)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}

// ==================== SensitiveAuditRepository ====================

#[derive(Debug, Clone)]
pub struct SensitiveAuditRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<SensitiveAuditEntry>>>,
}

impl SensitiveAuditRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let entries = match sqlx::query_as::<_, SensitiveAuditEntry>(
            r#"SELECT id, user_id, action, resource, detail, created_at FROM gw_sensitive_audit ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("SensitiveAuditRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = entries;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<SensitiveAuditEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<SensitiveAuditEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    pub async fn insert_entry(&self, entry: SensitiveAuditEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_sensitive_audit (id, user_id, action, resource, detail, created_at) VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(entry.id)
        .bind(entry.user_id)
        .bind(&entry.action)
        .bind(&entry.resource)
        .bind(&entry.detail)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}

// ==================== ScheduledTaskRepository ====================

#[derive(Debug, Clone)]
pub struct ScheduledTaskRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<ScheduledTaskEntry>>>,
}

impl ScheduledTaskRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let entries = match sqlx::query_as::<_, ScheduledTaskEntry>(
            r#"SELECT id, name, cron, handler, is_active, created_at FROM gw_scheduled_tasks ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("ScheduledTaskRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = entries;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<ScheduledTaskEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<ScheduledTaskEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    pub async fn insert_entry(&self, entry: ScheduledTaskEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_scheduled_tasks (id, name, cron, handler, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(entry.id)
        .bind(&entry.name)
        .bind(&entry.cron)
        .bind(&entry.handler)
        .bind(entry.is_active)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}

// ==================== ReportRepository ====================

#[derive(Debug, Clone)]
pub struct ReportRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<ReportEntry>>>,
}

impl ReportRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let entries = match sqlx::query_as::<_, ReportEntry>(
            r#"SELECT id, name, template_id, params, is_active, created_at FROM gw_reports ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("ReportRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = entries;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<ReportEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<ReportEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    pub async fn insert_entry(&self, entry: ReportEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_reports (id, name, template_id, params, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(entry.id)
        .bind(&entry.name)
        .bind(entry.template_id)
        .bind(&entry.params)
        .bind(entry.is_active)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}

// ==================== DataSourceRepository ====================

#[derive(Debug, Clone)]
pub struct DataSourceRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<DataSourceEntry>>>,
}

impl DataSourceRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let entries = match sqlx::query_as::<_, DataSourceEntry>(
            r#"SELECT id, name, "type", config, is_active, created_at FROM gw_data_sources ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("DataSourceRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = entries;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<DataSourceEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<DataSourceEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    pub async fn insert_entry(&self, entry: DataSourceEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_data_sources (id, name, "type", config, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(entry.id)
        .bind(&entry.name)
        .bind(&entry.r#type)
        .bind(&entry.config)
        .bind(entry.is_active)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}

// ==================== ReportTemplateRepository ====================

#[derive(Debug, Clone)]
pub struct ReportTemplateRepository {
    pool: PgPool,
    cache: Arc<Mutex<Vec<ReportTemplateEntry>>>,
}

impl ReportTemplateRepository {
    pub async fn new(pool: PgPool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(Mutex::new(Vec::new())),
        };
        repo.load_from_db().await;
        repo
    }

    async fn load_from_db(&self) {
        let entries = match sqlx::query_as::<_, ReportTemplateEntry>(
            r#"SELECT id, name, format, config, is_active, created_at FROM gw_report_templates ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("ReportTemplateRepository: failed to load from DB: {}", e);
                Vec::new()
            }
        };
        let mut cache = self.cache.lock().unwrap();
        *cache = entries;
    }

    #[must_use]
    pub fn entries(&self) -> Vec<ReportTemplateEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    /// 获取所有条目（可修改）
    pub fn entries_mut(&self) -> Vec<ReportTemplateEntry> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }

    pub async fn insert_entry(&self, entry: ReportTemplateEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO gw_report_templates (id, name, format, config, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(entry.id)
        .bind(&entry.name)
        .bind(&entry.format)
        .bind(&entry.config)
        .bind(entry.is_active)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.lock().unwrap();
        cache.push(entry);
        Ok(())
    }
}
