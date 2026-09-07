use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use grpc_proto::xlt::xlt_service_server::{XltService, XltServiceServer};
use grpc_proto::xlt::{
    BarrierCommand, BillingRequest as ProtoBillingRequest, DeviceInfo, DeviceInfoList, Empty,
    GenericResponse, GetParkingVehicleRequest, MqttCallbackRequest, ParkingVehicle as ProtoParkingVehicle,
    RecordListResponse, VehicleEvent as ProtoVehicleEvent, VehicleQueryRequest,
};

use crate::handlers::AppState;
use crate::models::{BillingRequest as ServiceBillingRequest, VehicleEvent as ServiceVehicleEvent};

pub async fn start_grpc_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("[XltService] gRPC listening on {addr}");
    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth_interceptor))
        .add_service(XltServiceServer::new(GrpcXltService { state: Arc::new(state) }))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}

use tonic::transport::Server;
use common::shutdown_signal;

#[derive(Clone)]
pub struct GrpcXltService {
    pub state: Arc<AppState>,
}

fn proto_event_to_service(event: ProtoVehicleEvent) -> ServiceVehicleEvent {
    ServiceVehicleEvent {
        plate_no: event.plate_no,
        plate_color: event.plate_color,
        park_code: event.park_code,
        lane_code: event.lane_code,
        event_time: event.event_time,
        vehicle_type: event.vehicle_type,
        image_url: event.image_url,
        amount: event.amount,
        pay_type: event.pay_type,
    }
}

#[tonic::async_trait]
impl XltService for GrpcXltService {
    async fn vehicle_entry(&self, request: Request<ProtoVehicleEvent>) -> Result<Response<GenericResponse>, Status> {
        let event = proto_event_to_service(request.into_inner());
        match self.state.parking_service.handle_entry(&event).await {
            Ok(record) => Ok(Response::new(GenericResponse {
                code: 0,
                message: "success".to_string(),
                data: serde_json::to_string(&record).unwrap_or_default(),
            })),
            Err(e) => Ok(Response::new(GenericResponse {
                code: 1,
                message: format!("{e}"),
                data: String::new(),
            })),
        }
    }

    async fn vehicle_exit(&self, request: Request<ProtoVehicleEvent>) -> Result<Response<GenericResponse>, Status> {
        let event = proto_event_to_service(request.into_inner());
        match self.state.parking_service.handle_exit(&event).await {
            Ok(record) => Ok(Response::new(GenericResponse {
                code: 0,
                message: "success".to_string(),
                data: serde_json::to_string(&record).unwrap_or_default(),
            })),
            Err(e) => Ok(Response::new(GenericResponse {
                code: 1,
                message: format!("{e}"),
                data: String::new(),
            })),
        }
    }

    async fn get_parking_vehicle(
        &self,
        request: Request<GetParkingVehicleRequest>,
    ) -> Result<Response<ProtoParkingVehicle>, Status> {
        let req = request.into_inner();
        match self.state.parking_service.get_parking_vehicle(&req.park_code, &req.plate_no).await {
            Ok(vehicle) => Ok(Response::new(ProtoParkingVehicle {
                plate_no: vehicle.plate_no,
                plate_color: vehicle.plate_color,
                park_code: vehicle.park_code,
                entry_time: vehicle.entry_time,
                lane_code: vehicle.lane_code,
                vehicle_type: vehicle.vehicle_type,
                duration_minutes: vehicle.duration_minutes,
                amount: vehicle.amount,
            })),
            Err(e) => Err(Status::not_found(format!("{e}"))),
        }
    }

    async fn calc_billing(&self, request: Request<ProtoBillingRequest>) -> Result<Response<grpc_proto::xlt::BillingResult>, Status> {
        let req = request.into_inner();
        let service_req = ServiceBillingRequest {
            plate_no: req.plate_no,
            park_code: req.park_code,
            entry_time: req.entry_time,
            exit_time: req.exit_time,
        };
        match self.state.billing_service.calculate(&service_req).await {
            Ok(result) => Ok(Response::new(grpc_proto::xlt::BillingResult {
                plate_no: result.plate_no,
                park_code: result.park_code,
                entry_time: result.entry_time,
                exit_time: result.exit_time,
                duration_minutes: result.duration_minutes,
                total_amount: result.total_amount,
                rule: result.rule,
            })),
            Err(e) => Err(Status::internal(format!("{e}"))),
        }
    }

    async fn list_records(&self, request: Request<VehicleQueryRequest>) -> Result<Response<RecordListResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(RecordListResponse {
            records_json: vec![],
            total: 0,
            page: 1,
            page_size: 20,
        }))
    }

    async fn mqtt_callback(&self, request: Request<MqttCallbackRequest>) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let envelope: Result<crate::models::MqttEnvelope, _> = serde_json::from_str(&req.payload);
        match envelope {
            Ok(env) => {
                tracing::info!("MQTT回调: topic={}, command={}, sn={}", req.topic, env.command, env.sn);
                self.state.device_manager.dispatch(env).await;
                Ok(Response::new(GenericResponse {
                    code: 200,
                    message: "success".to_string(),
                    data: String::new(),
                }))
            }
            Err(e) => Err(Status::invalid_argument(format!("解析MQTT消息失败: {e}"))),
        }
    }

    async fn list_devices(&self, _request: Request<Empty>) -> Result<Response<DeviceInfoList>, Status> {
        let devices = self.state.device_manager.list_devices().await;
        let proto_devices: Vec<DeviceInfo> = devices.into_iter().map(|d| DeviceInfo {
            sn: d.sn,
            name: String::new(),
            version: d.version,
            status: String::new(),
            last_heartbeat: d.last_heartbeat,
            dev_info: d.dev_info,
            park_code: String::new(),
        }).collect();
        Ok(Response::new(DeviceInfoList { devices: proto_devices }))
    }

    async fn open_barrier(&self, request: Request<BarrierCommand>) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        match self.state.mqtt_gateway.send_open(&req.sn, &req.request_id).await {
            Ok(()) => Ok(Response::new(GenericResponse { code: 0, message: "success".to_string(), data: String::new() })),
            Err(e) => Ok(Response::new(GenericResponse { code: 1, message: format!("{e}"), data: String::new() })),
        }
    }

    async fn close_barrier(&self, request: Request<BarrierCommand>) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        match self.state.mqtt_gateway.send_close(&req.sn, &req.request_id).await {
            Ok(()) => Ok(Response::new(GenericResponse { code: 0, message: "success".to_string(), data: String::new() })),
            Err(e) => Ok(Response::new(GenericResponse { code: 1, message: format!("{e}"), data: String::new() })),
        }
    }
}