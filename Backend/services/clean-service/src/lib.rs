use common::service_bootstrap::GrpcServiceBuilder;
use grpc_proto::clean::clean_service_server::CleanServiceServer;
use std::net::SocketAddr;

pub use common::AppError;
pub use common::AppResult;

pub mod clean_service;
pub mod models;
pub mod repository;
pub mod services;

// 导出应用状态
pub use clean_service::AppState;

// 导出 gRPC 服务实现
pub use services::grpc_impl::CleanGrpcService;


impl GrpcServiceBuilder for CleanGrpcService {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let state = self.state.clone();
        let addr: SocketAddr = grpc_addr.parse().map_err(|e| format!("无效的 gRPC 地址 '{grpc_addr}': {e}"))?;
        Ok(tokio::spawn(async move {
            let server = CleanServiceServer::new(CleanGrpcService::new(state));
            if let Err(e) = tonic::transport::Server::builder().add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        }))
    }
}
