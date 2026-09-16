use std::net::SocketAddr;

use tonic::transport::Server;

use grpc_proto::lpr::lpr_service_server::LprServiceServer;

use crate::grpc::LprGrpcService;

pub async fn start_grpc_server(addr: SocketAddr, service: LprGrpcService) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("[LprService] gRPC listening on {addr}" );
    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth_interceptor))
        .add_service(LprServiceServer::new(service))
        .serve_with_shutdown(addr, common::shutdown_signal())
        .await
        .map_err(|e| format!("{} gRPC 服务启动失败: {}" , "LprService" , e))?;
    Ok(())
}
