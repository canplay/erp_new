use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use grpc_proto::hik::hik_service_server::{HikService, HikServiceServer};
use grpc_proto::hik::{
    CouponRequest as ProtoCouponRequest, GenericResponse, HikRequest as ProtoHikRequest,
    SignoRequest as ProtoSignoRequest, SignoResponse,
};

use crate::handlers::hik_service::AppState;
use crate::models::HikRequest as ServiceHikRequest;

pub async fn start_grpc_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("[HikService] gRPC listening on {addr}");
    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth_interceptor))
        .add_service(HikServiceServer::new(GrpcHikService { state: Arc::new(state) }))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}

use tonic::transport::Server;
use common::shutdown_signal;

#[derive(Clone)]
pub struct GrpcHikService {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl HikService for GrpcHikService {
    async fn exec(&self, request: Request<ProtoHikRequest>) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let service_req = ServiceHikRequest {
            method: req.method,
            phone: req.phone,
            driver_id: req.driver_id,
            plate_no: req.plate_no,
            plate_color: req.plate_color,
            page_no: req.page_no,
            page_size: req.page_size,
            park_name: req.park_name,
            park_code: req.park_code,
            request_type: req.request_type,
            unique_id: req.unique_id,
            is_use_coupon: req.is_use_coupon,
            appeal_type: req.appeal_type,
            appeal_remark: req.appeal_remark,
            appeal_in_time: req.appeal_in_time,
            appeal_out_time: req.appeal_out_time,
            appeal_source: req.appeal_source,
            arrears_ids: req.arrears_ids,
        };
        match self.state.hik_service.exec(&service_req).await {
            Ok(data) => Ok(Response::new(GenericResponse {
                code: 0,
                message: "success".to_string(),
                data: data.to_string(),
            })),
            Err(e) => Ok(Response::new(GenericResponse {
                code: 1,
                message: format!("{e}"),
                data: String::new(),
            })),
        }
    }

    async fn coupon(&self, request: Request<ProtoCouponRequest>) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        if req.r#type == "car" {
            match self.state.hik_service.send_coupon(&req.phone, req.amount, req.start, &req.end).await {
                Ok(data) => Ok(Response::new(GenericResponse {
                    code: 0,
                    message: "success".to_string(),
                    data: data.to_string(),
                })),
                Err(e) => Ok(Response::new(GenericResponse {
                    code: 1,
                    message: format!("{e}"),
                    data: String::new(),
                })),
            }
        } else {
            Ok(Response::new(GenericResponse {
                code: 0,
                message: "success".to_string(),
                data: String::new(),
            }))
        }
    }

    async fn signo_open(&self, request: Request<ProtoSignoRequest>) -> Result<Response<SignoResponse>, Status> {
        let req = request.into_inner();
        match self.state.signo_service.open_gate(&req.place, &req.name).await {
            Ok(msg) => Ok(Response::new(SignoResponse { message: msg, status: 1 })),
            Err(e) => Ok(Response::new(SignoResponse { message: format!("{e}"), status: 0 })),
        }
    }
}