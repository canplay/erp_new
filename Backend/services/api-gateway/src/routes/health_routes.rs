//! 健康检查路由 — 存活探针、就绪探针、详细健康、服务发现、数据库连接

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    routing::get,
    response::IntoResponse,
};
use common::ApiResponse;

use crate::{
    AppState, DetailedHealth, DiscoveryHealth, DiscoveryServiceStatus, ServiceHealth, grpc_clients,
};

async fn health_handler() -> &'static str {
    "OK"
}

/// K8s readiness probe: 检查所有上游 gRPC 服务是否就绪
async fn ready_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let clients = state.grpc_clients.read().await;
    let all_ready = !clients.available_services().await.is_empty();

    if all_ready {
        (axum::http::StatusCode::OK, "OK")
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

async fn detailed_health_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let clients = state.grpc_clients.read().await;
    let services = vec![
        ("auth-service", clients.auth_service.is_connected().await),
        ("user-service", clients.user_service.is_connected().await),
        ("cms-service", clients.cms_service.is_connected().await),
        ("message-service", clients.message_service.is_connected().await),
        ("feedback-service", clients.feedback_service.is_connected().await),
        ("tenant-service", clients.tenant_service.is_connected().await),
        ("file-service", clients.file_service.is_connected().await),
        ("workflow-service", clients.workflow_service.is_connected().await),
        ("audit-service", clients.audit_service.is_connected().await),
        ("api-key-service", clients.api_key_service.is_connected().await),
    ];
    drop(clients);

    let services_health: Vec<ServiceHealth> = services
        .iter()
        .map(|(name, available)| ServiceHealth {
            service: name.to_string(),
            status: if *available {
                "healthy".to_string()
            } else {
                "unavailable".to_string()
            },
            latency_ms: None,
        })
        .collect();

    let overall_status = if services.iter().all(|(_, available)| *available) {
        "healthy"
    } else if services.iter().any(|(_, available)| *available) {
        "degraded"
    } else {
        "unhealthy"
    };

    let detailed = DetailedHealth {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.uptime_secs(),
        services: services_health,
    };

    Json(ApiResponse::success(detailed))
}

async fn discovery_health_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let clients = state.grpc_clients.read().await;
    let mut services = Vec::new();

    for def in grpc_clients::SERVICE_DEFS {
        let instances = state.service_discovery.get_instances(def.name);
        let addresses: Vec<String> = instances.iter().map(grpc_core::ServiceInstance::grpc_addr).collect();
        let connected = match def.key {
            "auth" => clients.auth_service.is_connected().await,
            "user" => clients.user_service.is_connected().await,
            "cms" => clients.cms_service.is_connected().await,
            "workflow" => clients.workflow_service.is_connected().await,
            "audit" => clients.audit_service.is_connected().await,
            "tenant" => clients.tenant_service.is_connected().await,
            "feedback" => clients.feedback_service.is_connected().await,
            "message" => clients.message_service.is_connected().await,
            "file" => clients.file_service.is_connected().await,
            "api-key" => clients.api_key_service.is_connected().await,
            _ => false,
        };

        services.push(DiscoveryServiceStatus {
            name: def.name.to_string(),
            registered: !instances.is_empty(),
            connected,
            instances: instances.len(),
            addresses,
        });
    }
    drop(clients);

    let overall = if services.iter().all(|s| s.connected) {
        "healthy"
    } else if services.iter().any(|s| s.connected) {
        "degraded"
    } else {
        "unhealthy"
    };

    Json(ApiResponse::success(DiscoveryHealth {
        status: overall.to_string(),
        services,
    }))
}

/// 数据库连接健康状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseConnectionHealth {
    pub service: String,
    pub status: String,
    pub schema: Option<String>,
    pub pool_size: Option<u32>,
    pub error: Option<String>,
}

/// 数据库健康检查响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct GatewayDatabaseHealth {
    pub status: String,
    pub connections: Vec<DatabaseConnectionHealth>,
}

/// 网关级数据库健康检查
async fn database_health_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let clients = state.grpc_clients.read().await;
    let service_defs = [
        ("auth-service", "auth"),
        ("user-service", "user"),
        ("cms-service", "cms"),
        ("message-service", "message"),
        ("feedback-service", "feedback"),
        ("tenant-service", "tenant"),
        ("file-service", "file"),
        ("workflow-service", "workflow"),
        ("audit-service", "audit"),
        ("api-key-service", "api-key"),
    ];

    let mut connections = Vec::new();
    for (service_name, key) in &service_defs {
        let available = match *key {
            "auth" => clients.auth_service.is_connected().await,
            "user" => clients.user_service.is_connected().await,
            "cms" => clients.cms_service.is_connected().await,
            "message" => clients.message_service.is_connected().await,
            "feedback" => clients.feedback_service.is_connected().await,
            "tenant" => clients.tenant_service.is_connected().await,
            "file" => clients.file_service.is_connected().await,
            "workflow" => clients.workflow_service.is_connected().await,
            "audit" => clients.audit_service.is_connected().await,
            "api-key" => clients.api_key_service.is_connected().await,
            _ => false,
        };

        connections.push(DatabaseConnectionHealth {
            service: service_name.to_string(),
            status: if available { "connected".to_string() } else { "unreachable".to_string() },
            schema: None,
            pool_size: None,
            error: if available { None } else { Some("gRPC connection not established".to_string()) },
        });
    }
    drop(clients);

    let all_connected = connections.iter().all(|c| c.status == "connected");
    let status = if all_connected {
        "healthy"
    } else {
        "degraded"
    };

    Json(ApiResponse::success(GatewayDatabaseHealth {
        status: status.to_string(),
        connections,
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/health/detailed", get(detailed_health_handler))
        .route("/health/discovery", get(discovery_health_handler))
        .route("/health/database", get(database_health_handler))
}
