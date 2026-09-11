//! gRPC client configuration
//!
//! Manages gRPC client instances and service discovery.

use std::sync::Arc;

use grpc_core::ServiceDiscovery;
use tokio::sync::RwLock as AsyncRwLock;

use crate::grpc_clients::GrpcClientConfig;

/// gRPC client configuration bundle.
///
/// Holds the gRPC client map and the service discovery
/// responsible for locating upstream service instances.
#[derive(Clone)]
pub struct GrpcConfig {
    /// The gRPC client map keyed by service name.
    pub clients: Arc<AsyncRwLock<crate::grpc_clients::GrpcClients>>,

    /// Service discovery for locating upstream gRPC endpoints.
    pub service_discovery: ServiceDiscovery,
}

impl GrpcConfig {
    /// Build a `GrpcConfig` from environment / defaults.
    pub async fn new() -> Self {
        let config = GrpcClientConfig::default();
        let service_discovery = ServiceDiscovery::new();
        crate::grpc_clients::register_services_to_discovery(&service_discovery);
        let grpc_clients = crate::grpc_clients::GrpcClients::from_discovery(config, &service_discovery).await;
        let grpc_clients = Arc::new(AsyncRwLock::new(grpc_clients));

        Self {
            clients: grpc_clients,
            service_discovery,
        }
    }
}
