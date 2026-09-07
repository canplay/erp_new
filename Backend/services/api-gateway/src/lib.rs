#![recursion_limit = "256"]

//! API Gateway Library
//!
//! API 网关 - 所有前端请求的唯一入口
//! 通过 gRPC 与各微服务通信
//!
//! # 架构
//! 前端 → api-gateway:8090 → 内部 gRPC → 微服务 (909x)
//!
//! @date 2026-05-20
//! @note 实现限流、熔断、路由等功能

use auth_core::JwtService;
use circuit_breaker_core::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerResult, CircuitState,
};
use grpc_core::ServiceDiscovery;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock as AsyncRwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

// 主要中间件模块
pub mod enhanced_logging;
pub mod grpc_clients;
pub mod grpc_error_middleware;
pub mod metrics;
pub mod http_client;
pub mod middleware;
pub mod openapi;
pub mod retry;
pub mod stats;
pub mod tests;
pub mod route_builder;
pub use route_builder::create_router;
pub mod routes;
pub mod ws_routes;

// In-memory repository types for AppState state management
pub mod repository;
use crate::repository::{
    DeviceRepository, IpWhitelistRepository, SensitiveAuditRepository,
    ScheduledTaskRepository, ReportRepository, DataSourceRepository, ReportTemplateRepository,
};

// 从 middleware.rs 重新导出
pub use enhanced_logging::{
    AccessLog, RequestContext, RequestLog, get_client_ip, get_content_length, get_trace_id,
    get_user_agent, get_user_id, health_logging_middleware, log_ws_error, request_id_middleware,
};
pub use grpc_clients::GrpcClients;
pub use metrics::{MetricsCollector, metrics_collector};
pub use middleware::{
    AuthState, JwtClaims, RateLimitState, auth_middleware, cors_layer, logging_middleware,
    rate_limit_middleware,
};
pub use retry::{
    RetryCondition, RetryConfig, RetryResult, RetryableError, Retryer,
    create_retry_config, retry_request, retry_with_backoff, create_grpc_client_with_retry,
};

// WebSocket 路由
pub use ws_routes::{WsStatus, ws_messages_handler, ws_status_handler};

// 所有微服务模块（用于 gRPC 服务发现）

// gRPC 服务构建器

/// 熔断器管理器 - 管理所有上游服务的熔断器
pub struct CircuitBreakerManager {
    /// 每个服务的熔断器
    breakers: RwLock<HashMap<String, Arc<CircuitBreaker>>>,
    /// 熔断器配置
    config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// 创建新的熔断器管理器
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// 获取或创建服务的熔断器
    pub fn get_breaker(&self, service_name: &str) -> Arc<CircuitBreaker> {
        let breakers = self.breakers.read().unwrap();
        if let Some(breaker) = breakers.get(service_name) {
            return breaker.clone();
        }
        drop(breakers);

        // 创建新的熔断器
        let breaker = Arc::new(CircuitBreaker::new(self.config.clone()));
        let mut breakers = self.breakers.write().unwrap();
        breakers.insert(service_name.to_string(), breaker.clone());
        breaker
    }

    /// 检查请求是否允许（考虑熔断器状态）
    pub fn allow_request(&self, service_name: &str) -> bool {
        let breaker = self.get_breaker(service_name);
        breaker.allow(service_name)
    }

    /// 记录请求结果
    pub fn record_result(&self, service_name: &str, success: bool) {
        let breaker = self.get_breaker(service_name);
        breaker.record(service_name, success);
    }

    /// 获取熔断器状态
    pub fn get_state(&self, service_name: &str) -> CircuitState {
        let breaker = self.get_breaker(service_name);
        breaker.state()
    }

    /// 获取熔断器检查结果
    pub fn check_circuit(&self, service_name: &str) -> CircuitBreakerResult {
        let breaker = self.get_breaker(service_name);
        let state = breaker.state();

        CircuitBreakerResult {
            allowed: breaker.allow(service_name),
            state: state.to_string(),
            retry_after_secs: None,
        }
    }

    /// 获取所有服务状态
    pub fn get_all_states(&self) -> HashMap<String, CircuitState> {
        let breakers = self.breakers.read().unwrap();
        breakers
            .iter()
            .map(|(name, breaker)| (name.clone(), breaker.state()))
            .collect()
    }

    /// 重置指定服务的熔断器
    pub fn reset(&self, service_name: &str) {
        let breaker = self.get_breaker(service_name);
        breaker.reset(service_name);
        tracing::info!("熔断器已重置: {service_name}");
    }

    /// 重置所有熔断器
    pub fn reset_all(&self) {
        let breakers = self.breakers.read().unwrap();
        for (name, breaker) in breakers.iter() {
            breaker.reset(name);
        }
        tracing::info!("所有熔断器已重置");
    }

    /// 获取服务统计信息
    pub fn get_stats(&self, service_name: &str) -> Option<circuit_breaker_core::CircuitStats> {
        let breaker = self.get_breaker(service_name);
        breaker.get_stats(service_name)
    }
}

/// 熔断器状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStatusResponse {
    pub service: String,
    pub state: String,
    pub allowed: bool,
    pub stats: Option<CircuitStatsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitStatsInfo {
    pub success_count: u32,
    pub failure_count: u32,
    pub failure_rate: f64,
    pub total_requests: u32,
}

/// 熔断器重置请求
#[derive(Debug, Deserialize)]
pub struct CircuitBreakerResetRequest {
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service: String,
    pub status: String,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealth {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub services: Vec<ServiceHealth>,
}

/// 服务发现健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryServiceStatus {
    pub name: String,
    pub registered: bool,
    pub connected: bool,
    pub instances: usize,
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryHealth {
    pub status: String,
    pub services: Vec<DiscoveryServiceStatus>,
}

/// 应用状态 - 包含 gRPC 客户端、HTTP 客户端和熔断器设置
#[derive(Clone)]
pub struct AppState {
    // 服务发现
    pub service_discovery: ServiceDiscovery,

    // gRPC 客户端管理器（使用 RwLock 支持后台重连）
    pub grpc_clients: Arc<AsyncRwLock<GrpcClients>>,

    // JWT 服务（用于生成 refresh_token）
    pub jwt_service: JwtService,

    // HTTP 客户端（用于代理请求到上游服务，单例/连接池）
    pub http_client: Arc<reqwest::Client>,

    // HTTP 客户端 URL（用于统计接口）
    pub user_service_url: String,
    pub audit_service_url: String,
    pub tenant_service_url: String,

    // 熔断器配置
    pub failure_threshold: u32,
    pub timeout_secs: u64,
    pub half_open_requests: u32,

    // 熔断器管理器
    pub circuit_breaker_manager: Arc<CircuitBreakerManager>,

    // 设备管理（持久化到 PostgreSQL）
    pub device_store: Arc<AsyncRwLock<DeviceRepository>>,
    // IP 白名单存储（持久化到 PostgreSQL）
    pub ip_whitelist_store: Arc<AsyncRwLock<IpWhitelistRepository>>,
    // 敏感审计存储（持久化到 PostgreSQL）
    pub sensitive_audit_store: Arc<AsyncRwLock<SensitiveAuditRepository>>,
    // 定时任务存储（持久化到 PostgreSQL）
    pub scheduled_task_store: Arc<AsyncRwLock<ScheduledTaskRepository>>,
    // 报表存储（持久化到 PostgreSQL）
    pub report_store: Arc<AsyncRwLock<ReportRepository>>,
    // 数据源存储（持久化到 PostgreSQL）
    pub data_source_store: Arc<AsyncRwLock<DataSourceRepository>>,
    // 报表模板存储（持久化到 PostgreSQL）
    pub report_template_store: Arc<AsyncRwLock<ReportTemplateRepository>>,

    // CORS 配置（启动时加载，每个请求复用）
    pub cors_config: crate::middleware::CorsConfig,
    // 服务启动时间
    pub start_time: std::time::Instant,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new(pool: sqlx::PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        // gRPC 客户端配置
        let config = grpc_clients::GrpcClientConfig::default();

        // 初始化服务发现（从环境变量注册所有上游服务）
        let service_discovery = ServiceDiscovery::new();
        grpc_clients::register_services_to_discovery(&service_discovery);

        // 从服务发现初始化 gRPC 客户端
        let grpc_clients = GrpcClients::from_discovery(config, &service_discovery).await;
        let grpc_clients = Arc::new(AsyncRwLock::new(grpc_clients));

        // HTTP 服务 URL（用于统计接口）
        let user_service_url = std::env::var("USER_SERVICE_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:50052".to_string());
        let audit_service_url = std::env::var("AUDIT_SERVICE_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:50059".to_string());
        let tenant_service_url = std::env::var("TENANT_SERVICE_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:50051".to_string());

        // 熔断器配置
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

        // 创建熔断器配置
        let cb_config = CircuitBreakerConfig {
            failure_threshold: f64::from(failure_threshold),
            recovery_timeout_secs: timeout_secs,
            half_open_requests,
            window_size_secs: 60,
            min_requests: 5,
        };

        // 创建 HTTP 客户端管理器（单例/连接池）
        let http_client_config = http_client::HttpClientConfig {
            timeout_secs: 30,
            max_idle_connections_per_host: 10,
        };
        let http_client_manager = http_client::HttpClientManager::new(http_client_config);
        let http_client = http_client_manager.client();

        // 初始化 JWT 服务
        // 审计修复 (B6): 删除硬编码兜底密钥, 缺失即启动失败(fail-fast)
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET 环境变量必须配置");
        let jwt_service = JwtService::new(
            &jwt_secret,
            "myai",
            "myai-users",
            3600, // access token expiry: 1 hour
            604800, // refresh token expiry: 7 days
        );

        // CORS 配置（启动时加载，每个请求复用）
        let cors_config = crate::middleware::CorsConfig::from_env();

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
            start_time: std::time::Instant::now(),
        })
    }

    /// 从环境变量创建应用状态（备用方法，异步版本）
    pub async fn from_env(_pool: sqlx::PgPool) -> Self {
        let config = grpc_clients::GrpcClientConfig::default();

        let service_discovery = ServiceDiscovery::new();
        grpc_clients::register_services_to_discovery(&service_discovery);

        let grpc_clients = GrpcClients::new(config);
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

        // 创建 HTTP 客户端管理器（单例/连接池）
        let http_client_config = http_client::HttpClientConfig {
            timeout_secs: 30,
            max_idle_connections_per_host: 10,
        };
        let http_client_manager = http_client::HttpClientManager::new(http_client_config);
        let http_client = http_client_manager.client();

        // 初始化 JWT 服务
        // 审计修复 (B6): 删除硬编码兜底密钥, 缺失即启动失败(fail-fast)
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET 环境变量必须配置");
        let jwt_service = JwtService::new(
            &jwt_secret,
            "myai",
            "myai-users",
            3600, // access token expiry: 1 hour
            604800, // refresh token expiry: 7 days
        );

        // 初始化数据库连接池（备用方法）
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/myai".to_string());
        let pool = sqlx::PgPool::connect_lazy(&db_url)
            .unwrap_or_else(|_| sqlx::PgPool::connect_lazy(&db_url).unwrap());

        Self {
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
            cors_config: crate::middleware::CorsConfig::from_env(),
            start_time: std::time::Instant::now(),
        }
    }

    /// 启动后台重连任务
    ///
    /// 定期检查所有上游 gRPC 连接，对已断开的服务自动重连。
    /// 使用 `ServiceDiscovery` 的轮询负载均衡获取可用实例地址。
    pub fn start_reconnection_task(this: &Arc<Self>) {
        let clients_arc = this.grpc_clients.clone();
        let sd = this.service_discovery.clone();
        let reconnect_config = common::config::ServiceDiscoveryConfig::from_env();
        let interval_secs = reconnect_config.reconnect_interval_secs;

        tracing::info!(
            "[服务发现] 启动后台重连任务，间隔 {interval_secs} 秒"
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            interval.tick().await;
            loop {
                interval.tick().await;

                let clients = clients_arc.read().await;
                clients.reconnect_all(&sd).await;
            }
        });
    }

    #[must_use]
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
