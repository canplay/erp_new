use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use grpc_proto::ctp::ctp_service_server::{CtpService, CtpServiceServer};
use grpc_proto::ctp::{
    CommandResult, CtpResponse as ProtoCtpResponse, DeviceDataUpload as ProtoDeviceDataUpload,
    DeviceListResponse, DeviceQueryRequest, GetDeviceRequest, LockControlRequest as ProtoLockControlRequest,
    LockDevice as ProtoLockDevice, PingRequest, PingResponse,
};

use crate::ctp::AppState;
use crate::models::{CmdType, DeviceDataUpload as ServiceDeviceDataUpload};

pub async fn start_grpc_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("[CtpService] gRPC listening on {addr}");
    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth_interceptor))
        .add_service(CtpServiceServer::new(GrpcCtpService { state: Arc::new(state) }))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}

use tonic::transport::Server;
use common::shutdown_signal;

#[derive(Clone)]
pub struct GrpcCtpService {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl CtpService for GrpcCtpService {
    async fn ping(&self, _request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse { message: "pong".to_string() }))
    }

    async fn receive_device_data(
        &self,
        request: Request<ProtoDeviceDataUpload>,
    ) -> Result<Response<ProtoCtpResponse>, Status> {
        let req = request.into_inner();
        let upload = ServiceDeviceDataUpload {
            device_no: req.device_no,
            data_type: req.data_type,
            voltage: req.voltage,
            status_one: req.status_one,
            status_two: req.status_two,
            data_time: req.data_time,
        };
        match self.state.ctp_service.upload_device_data(&upload).await {
            Ok(resp) => {
                let _ = self.state.ctp_service.handle_device_data_upload(&upload).await;
                Ok(Response::new(ProtoCtpResponse {
                    error_code: resp.error_code,
                    error_msg: resp.error_msg,
                }))
            }
            Err(e) => Ok(Response::new(ProtoCtpResponse {
                error_code: 1,
                error_msg: format!("{e}"),
            })),
        }
    }

    async fn control_lock(
        &self,
        request: Request<ProtoLockControlRequest>,
    ) -> Result<Response<CommandResult>, Status> {
        let req = request.into_inner();
        let cmd_type = match req.cmd_type.as_str() {
            "up" => CmdType::Up,
            "down" => CmdType::Down,
            "syn" => CmdType::Syn,
            _ => return Err(Status::invalid_argument("unknown cmd_type")),
        };
        match self.state.ctp_service.send_lock_command(&req.device_no, &cmd_type, req.data.as_deref()).await {
            Ok(resp) => Ok(Response::new(CommandResult {
                success: resp.error_code == 0,
                message: resp.error_msg,
                device_no: req.device_no,
                action: req.cmd_type,
            })),
            Err(e) => Ok(Response::new(CommandResult {
                success: false,
                message: format!("{e}"),
                device_no: req.device_no,
                action: req.cmd_type,
            })),
        }
    }

    async fn get_device(&self, request: Request<GetDeviceRequest>) -> Result<Response<ProtoLockDevice>, Status> {
        let req = request.into_inner();
        match self.state.ctp_service.get_device_status(&req.device_no).await {
            Ok(device) => Ok(Response::new(ProtoLockDevice {
                id: device.id,
                device_no: device.device_no,
                factory_id: device.factory_id,
                status: format!("{:?}", device.status).to_lowercase(),
                battery: device.battery,
                signal: device.signal,
                voltage: device.voltage,
                park_code: device.park_code,
                created_at: device.created_at,
                updated_at: device.updated_at,
            })),
            Err(e) => Err(Status::not_found(format!("{e}"))),
        }
    }

    async fn list_devices(
        &self,
        request: Request<DeviceQueryRequest>,
    ) -> Result<Response<DeviceListResponse>, Status> {
        let req = request.into_inner();
        match self.state.ctp_service.list_devices(req.park_code.as_deref(), req.page, req.page_size).await {
            Ok(json) => {
                let devices = json["devices"].as_array().map(|arr| {
                    arr.iter().map(|d| ProtoLockDevice {
                        id: d["id"].as_str().unwrap_or_default().to_string(),
                        device_no: d["device_no"].as_str().unwrap_or_default().to_string(),
                        factory_id: d["factory_id"].as_str().unwrap_or_default().to_string(),
                        status: d["status"].as_str().unwrap_or_default().to_string(),
                        battery: d["battery"].as_i64().map(|v| v as i32),
                        signal: d["signal"].as_i64().map(|v| v as i32),
                        voltage: d["voltage"].as_str().map(std::string::ToString::to_string),
                        park_code: d["park_code"].as_str().unwrap_or_default().to_string(),
                        created_at: d["created_at"].as_str().unwrap_or_default().to_string(),
                        updated_at: d["updated_at"].as_str().unwrap_or_default().to_string(),
                    }).collect()
                }).unwrap_or_default();
                Ok(Response::new(DeviceListResponse {
                    devices,
                    total: json["total"].as_i64().unwrap_or(0),
                    page: req.page,
                    page_size: req.page_size,
                }))
            }
            Err(e) => Err(Status::internal(format!("{e}"))),
        }
    }
}