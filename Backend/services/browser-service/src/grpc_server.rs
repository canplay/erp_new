//! gRPC 服务端启动

use std::net::SocketAddr;
use std::sync::Arc;

use tonic::transport::Server;

use grpc_proto::browser::browser_service_server::BrowserServiceServer;

use crate::browser_pool::BrowserPool;
use crate::grpc_handlers::GrpcAppState;
use common::shutdown::shutdown_signal;

/// 启动 gRPC 服务器
pub async fn start_grpc_server(
    addr: SocketAddr,
    pool: Arc<BrowserPool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = GrpcAppState::new(pool);

    tracing::info!("[BrowserService] gRPC listening on {}" , addr);

    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth::grpc_auth_interceptor))
        .add_service(BrowserServiceServer::new(state))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}