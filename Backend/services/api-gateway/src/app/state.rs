//! Application State — centralized state management
//!
//! Extracted from `lib.rs` to keep the library root focused on exports
//! and to provide a dedicated home for `AppState` construction and helpers.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock as AsyncRwLock;

use common::config::ServiceDiscoveryConfig;

use circuit_breaker_core::CircuitBreakerConfig;
use crate::CircuitBreakerManager;
use crate::grpc_clients::{GrpcClients, GrpcClientConfig};
use crate::http_client::{HttpClientConfig, HttpClientManager};
use crate::middleware::CorsConfig;
use crate::repository::{
    DeviceRepository, IpWhitelistRepository, ReportRepository, ReportTemplateRepository,
    ScheduledTaskRepository, SensitiveAuditRepository, DataSourceRepository,
};
use crate::ServiceDiscovery;

/// Application state — shared across all request handlers.
///
/// Holds gRPC clients, circuit breakers, JWT service, HTTP clients,
/// in-memory repositories, and misc configuration.
#[derive(Clone)]
pub struct AppState {
    /// Service discovery for upstream gRPC services.
    pub service_discovery: ServiceDiscovery,
    /// gRPC client manager (RwLock allows background reconnection).
    pub grpc_clients: Arc<AsyncRwLock<GrpcClients>>,
    /// JWT service for token generation.
    pub jwt_service: auth_core::JwtService,
    /// HTTP client for proxying to upstream services.
    pub http_client: Arc<reqwest::Client>,
    /// HTTP URL of user-service (for stats).
    pub user_service_url: String,
    /// HTTP URL of audit-service (for stats).
    pub audit_service_url: String,
    /// HTTP URL of tenant-service (for stats).
    pub tenant_service_url: String,
    /// Circuit breaker failure threshold.
    pub failure_threshold: u32,
    /// Circuit breaker recovery timeout (seconds).
    pub timeout_secs: u64,
    /// Circuit breaker half-open request count.
    pub half_open_requests: u32,
    /// Circuit breaker manager.
    pub circuit_breaker_manager: Arc<CircuitBreakerManager>,
    /// Device store (persisted to PostgreSQL).
    pub device_store: Arc<AsyncRwLock<DeviceRepository>>,
    /// IP whitelist store (persisted to PostgreSQL).
    pub ip_whitelist_store: Arc<AsyncRwLock<IpWhitelistRepository>>,
    /// Sensitive audit store (persisted to PostgreSQL).
    pub sensitive_audit_store: Arc<AsyncRwLock<SensitiveAuditRepository>>,
    /// Scheduled task store (persisted to PostgreSQL).
    pub scheduled_task_store: Arc<AsyncRwLock<ScheduledTaskRepository>>,
    /// Report store (persisted to PostgreSQL).
    pub report_store: Arc<AsyncRwLock<ReportRepository>>,
    /// Data source store (persisted to PostgreSQL).
    pub data_source_store: Arc<AsyncRwLock<DataSourceRepository>>,
    /// Report template store (persisted to PostgreSQL).
    pub report_template_store: Arc<AsyncRwLock<ReportTemplateRepository>>,
    /// CORS configuration (loaded at startup, reused per-request).
    pub cors_config: CorsConfig,
    /// Service start time (for uptime calculation).
    pub start_time: Instant,
}

impl AppState {
    /// Creates a new `AppState` with all subsystems initialized.
    ///
    /// # Errors
    ///
    /// Returns an error if gRPC client initialization fails or required
    /// environment variables are missing.
    pub async fn new(pool: sqlx::PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        let config = GrpcClientConfig::default();
        let service_discovery = ServiceDiscovery::new();
        crate::grpc_clients::register_services_to_discovery(&service_discovery);
        let grpc_clients = GrpcClients::from_discovery(config, &service_discovery).await;
        let grpc_clients = Arc::new(AsyncRwLock::new(grpc_clients));

        let user_service_url = std::env::var("USER_SERVICE_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:50052".to_string());
        let audit_service_url = std::env::var("AUDIT_SERVICE_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:50059".to_string());
        let tenant_service_url = std::env::var("TENANT_SERVICE_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:50051".to_string());

        let failure_threshold: u32 = std::env::var("CIRCUIT_BREAKER_FAILURE_THRESHOLD")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);
        let timeout_secs: u64 = std::env::var("CIRCUIT_BREAKER_TIMEOUT_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);
        let half_open_requests: u32 = std::env::var("CIRCUIT_BREAKER_HALF_OPEN_REQUESTS")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let cb_config = CircuitBreakerConfig {
            failure_threshold: f64::from(failure_threshold),
            recovery_timeout_secs: timeout_secs,
            half_open_requests,
            window_size_secs: 60,
            min_requests: 5,
        };

        let http_client_config = HttpClientConfig {
            timeout_secs: 30,
            max_idle_connections_per_host: 10,
        };
        let http_client_manager = HttpClientManager::new(http_client_config);
        let http_client = http_client_manager.client();

        // Fail-fast: JWT_SECRET must be configured.
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET environment variable must be set");
        let jwt_service = auth_core::JwtService::new(
            &jwt_secret,
            "myai",
            "myai-users",
            3600,   // access token: 1 hour
            604800, // refresh token: 7 days
        );

        let cors_config = CorsConfig::from_env();

        Ok(Self {
            service_discovery,
            grpc_clients,
            http_client,
            jwt_service,
            user_service_url,
            audit_service_url,
            tenant_service_url,
            failure_threshold,
            timeout_secs,
            half_open_requests,
            circuit_breaker_manager: Arc::new(CircuitBreakerManager::new(cb_config)),
            device_store: Arc::new(AsyncRwLock::new(DeviceRepository::new(pool.clone()).await)),
            ip_whitelist_store: Arc::new(AsyncRwLock::new(IpWhitelistRepository::new(pool.clone()).await)),
            sensitive_audit_store: Arc::new(AsyncRwLock::new(SensitiveAuditRepository::new(pool.clone()).await)),
            scheduled_task_store: Arc::new(AsyncRwLock::new(ScheduledTaskRepository::new(pool.clone()).await)),
            report_store: Arc::new(AsyncRwLock::new(ReportRepository::new(pool.clone()).await)),
            data_source_store: Arc::new(AsyncRwLock::new(DataSourceRepository::new(pool.clone()).await)),
            report_template_store: Arc::new(AsyncRwLock::new(ReportTemplateRepository::new(pool.clone()).await)),
            cors_config,
            start_time: Instant::now(),
        })
    }

    /// Spawns a background task that periodically reconnects to all upstream gRPC services.
    pub fn start_reconnection_task(this: &Arc<Self>) {
        let clients_arc = this.grpc_clients.clone();
        let sd = this.service_discovery.clone();
        let reconnect_config = ServiceDiscoveryConfig::from_env();
        let interval_secs = reconnect_config.reconnect_interval_secs;

        tracing::info!(
            "[service-discovery] Starting reconnection task, interval {interval_secs}s"
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            interval.tick().await;
            loop {
                interval.tick().await;
                let clients = clients_arc.read().await;
                clients.reconnect_all(&sd).await;
            }
        });
    }

    /// Returns the uptime of the service in seconds.
    #[must_use]
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
