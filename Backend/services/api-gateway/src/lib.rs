#![recursion_limit = "256" ]

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

use circuit_breaker_core::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerResult, CircuitState};
use grpc_core::ServiceDiscovery;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

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
pub mod app;
pub use app::state::AppState;
pub mod repository;
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

// Re-exports for convenience
pub use app::{
    CircuitBreakerConfigBundle, GrpcConfig, Repositories,
};

/// 熔断器管理器 - 管理所有上游服务的熔断器
pub struct CircuitBreakerManager {
    /// 每个服务的熔断器
    breakers: parking_lot::RwLock<HashMap<String, Arc<CircuitBreaker>>>,
    /// 熔断器配置
    config: CircuitBreakerConfig,
}

impl std::fmt::Debug for CircuitBreakerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreakerManager" )
            .field("config" , &self.config)
            .field("breakers" , &"..." )
            .finish()
    }
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
        let breakers = self.breakers.read();
        if let Some(breaker) = breakers.get(service_name) {
            return breaker.clone();
        }
        drop(breakers);

        // 创建新的熔断器
        let breaker = Arc::new(CircuitBreaker::new(self.config.clone()));
        let mut breakers = self.breakers.write();
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
        breaker.record(service_name, success)
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
        let breakers = self.breakers.read();
        breakers
            .iter()
            .map(|(name, breaker)| (name.clone(), breaker.state()))
            .collect()
    }

    /// 重置指定服务的熔断器
    pub fn reset(&self, service_name: &str) {
        let breaker = self.get_breaker(service_name);
        breaker.reset(service_name);
        tracing::info!("熔断器已重置: {service_name}" );
    }

    /// 重置所有熔断器
    pub fn reset_all(&self) {
        let breakers = self.breakers.read();
        for (name, breaker) in breakers.iter() {
            breaker.reset(name);
        }
        tracing::info!("所有熔断器已重置" );
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

// AppState moved to app/state.rs
