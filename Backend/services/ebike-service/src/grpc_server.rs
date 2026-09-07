use tonic::transport::Server;

use tokio::task::JoinHandle;
use std::sync::Arc;
use common::service_bootstrap::GrpcServiceBuilder;

use crate::services::ebike::ebike_server::EbikeGrpcService;
use crate::AppState;
use grpc_proto::ebike::ebike_service_server::EbikeServiceServer;

/// gRPC 服务构建器 —— 实现 GrpcServiceBuilder trait 供 ServiceBootstrap 使用
#[derive(Clone)]
pub struct GrpcServerBuilder {
    pub state: Arc<AppState>,
}

impl GrpcServiceBuilder for GrpcServerBuilder {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let state = self.state.clone();
        let addr = grpc_addr.parse()?;

        let server = Server::builder()
            .add_service(EbikeServiceServer::new(EbikeGrpcService::new(state.clone())));

        let handle = tokio::spawn(async move {
            if let Err(e) = server.serve(addr).await {
                tracing::error!("[ebike-service] gRPC server error: {}", e);
            }
        });

        Ok(handle)
    }
}
