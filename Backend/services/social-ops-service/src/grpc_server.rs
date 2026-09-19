//! gRPC Server — Tonic-based gRPC server for Social Ops Service
//!
//! Registers all 8 services:
//! - `AccountService`, `CrawlService`, `ContentService`
//! - `PublishService`, `LLMProviderService`, `RewriteService`
//! - `StatsService`, `InsightService`

use std::sync::Arc;
use tonic::transport::Server;
use grpc_proto::socialops::account_service_server::AccountServiceServer;
use grpc_proto::socialops::crawl_service_server::CrawlServiceServer;
use grpc_proto::socialops::content_service_server::ContentServiceServer;
use grpc_proto::socialops::publish_service_server::PublishServiceServer;
use grpc_proto::socialops::llm_provider_service_server::LlmProviderServiceServer;
use grpc_proto::socialops::rewrite_service_server::RewriteServiceServer;
use grpc_proto::socialops::stats_service_server::StatsServiceServer;
use grpc_proto::socialops::insight_service_server::InsightServiceServer;
use common::shutdown::shutdown_signal;

use crate::grpc_handlers::{
    GrpcAccountService, GrpcCrawlService, GrpcContentService,
    GrpcPublishService, GrpcLlmProviderService, GrpcRewriteService,
    GrpcStatsService, GrpcInsightService,
    AppState,
};

pub async fn start_grpc_server(
    addr: std::net::SocketAddr,
    app_state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(app_state);

    tracing::info!("Starting gRPC server on {}" , addr);

    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth::grpc_auth_interceptor))
        .add_service(AccountServiceServer::new(GrpcAccountService::new(state.clone())))
        .add_service(CrawlServiceServer::new(GrpcCrawlService::new(state.clone())))
        .add_service(ContentServiceServer::new(GrpcContentService::new(state.clone())))
        .add_service(PublishServiceServer::new(GrpcPublishService::new(state.clone())))
        .add_service(LlmProviderServiceServer::new(GrpcLlmProviderService::new(state.clone())))
        .add_service(RewriteServiceServer::new(GrpcRewriteService::new(state.clone())))
        .add_service(StatsServiceServer::new(GrpcStatsService::new(state.clone())))
        .add_service(InsightServiceServer::new(GrpcInsightService::new(state.clone())))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}