//! LPR 服务 gRPC 处理器
//!
//! 实现 `LprService` gRPC trait：车牌识别回调 + 通行记录查询

use std::sync::Arc;
use tonic::{Request, Response, Status};

use chrono::NaiveDateTime;
use grpc_proto::lpr::{
    lpr_service_server::LprService,
    LprCallbackRequest, LprCallbackResponse,
    ListPassRecordsRequest, ListPassRecordsResponse,
    GetPassRecordRequest, PassRecordInfo,
    GetPassStatsRequest, PassStatsResponse,
    GetVehicleAuthRequest, VehicleAuthInfo,
};

use crate::models::CreatePassRecord;
use crate::services::PassService;

pub struct LprGrpcService {
    pub pass_service: Arc<PassService>,
}

impl LprGrpcService {
    #[must_use]
    pub fn new(pass_service: Arc<PassService>) -> Self {
        Self { pass_service }
    }

    fn record_to_proto(r: &crate::models::PassRecord) -> PassRecordInfo {
        PassRecordInfo {
            id: r.id,
            plate_no: r.plate_no.clone(),
            plate_color: r.plate_color.clone(),
            plate_type: r.plate_type.clone(),
            vehicle_type: r.vehicle_type.clone(),
            device_id: r.device_id.clone(),
            device_name: r.device_name.clone(),
            park_code: r.park_code.clone(),
            lane_code: r.lane_code.clone(),
            direction: r.direction.clone(),
            pass_time: r.pass_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            image_url: r.image_url.clone(),
            confidence: f64::from(r.confidence),
            status: r.status.clone(),
            related_order_id: r.related_order_id.clone(),
            remark: r.remark.clone(),
            created_at: r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            updated_at: r.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        }
    }
}

#[tonic::async_trait]
impl LprService for LprGrpcService {
    async fn lpr_callback(&self, request: Request<LprCallbackRequest>) -> Result<Response<LprCallbackResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("车牌识别回调: 车牌={}, 方向={}", req.plate_no, req.direction);

        let create_req = CreatePassRecord {
            plate_no: req.plate_no.clone(),
            plate_color: req.plate_color.clone(),
            plate_type: req.plate_type.clone(),
            vehicle_type: req.vehicle_type.clone(),
            device_id: req.device_id.clone(),
            device_name: req.device_name.clone(),
            park_code: req.park_code.clone(),
            lane_code: req.lane_code.clone(),
            direction: req.direction.clone(),
            pass_time: parse_pass_time(&req.pass_time),
            image_url: req.image_url.clone(),
            confidence: req.confidence as f32,
        };

        match self.pass_service.process_callback(&create_req).await {
            Ok(result) => {
                tracing::info!("车牌识别完成: id={}", result.record_id);
                Ok(Response::new(LprCallbackResponse {
                    success: true,
                    message: format!("车牌 {} 处理完成, 开闸={}", req.plate_no, if result.gate_opened {"是"} else {"否"}),
                    code: 0,
                }))
            }
            Err(e) => {
                tracing::error!("车牌识别处理失败: {e}");
                Ok(Response::new(LprCallbackResponse { success: false, message: format!("失败: {e}"), code: 1 }))
            }
        }
    }

    async fn list_pass_records(&self, request: Request<ListPassRecordsRequest>) -> Result<Response<ListPassRecordsResponse>, Status> {
        let req = request.into_inner();
        let records = self.pass_service.list_records(
            &req.plate_no, &req.park_code, &req.direction, &req.status,
            req.page.max(1), req.page_size.max(20),
        ).await.map_err(|e| Status::internal(format!("{e}")))?;

        let proto_records: Vec<PassRecordInfo> = records.iter().map(Self::record_to_proto).collect();
        Ok(Response::new(ListPassRecordsResponse {
            records: proto_records,
            total: records.len() as i64,
            page: req.page, page_size: req.page_size,
        }))
    }

    async fn get_pass_record(&self, request: Request<GetPassRecordRequest>) -> Result<Response<PassRecordInfo>, Status> {
        let req = request.into_inner();
        let record = self.pass_service.get_record(req.id).await
            .map_err(|e| Status::not_found(format!("{e}")))?;
        Ok(Response::new(Self::record_to_proto(&record)))
    }

    async fn get_pass_stats(&self, _request: Request<GetPassStatsRequest>) -> Result<Response<PassStatsResponse>, Status> {
        let stats = self.pass_service.get_stats().await
            .map_err(|e| Status::internal(format!("{e}")))?;
        Ok(Response::new(PassStatsResponse {
            total_pass: stats.total_pass,
            total_entry: stats.total_entry,
            total_exit: stats.total_exit,
            high_confidence: stats.high_confidence,
            park_code: stats.park_code,
        }))
    }

    async fn get_vehicle_auth(&self, request: Request<GetVehicleAuthRequest>) -> Result<Response<VehicleAuthInfo>, Status> {
        let req = request.into_inner();
        let auth = self.pass_service.get_vehicle_auth(&req.plate_no, &req.park_code).await
            .map_err(|e| Status::internal(format!("{e}")))?;
        Ok(Response::new(VehicleAuthInfo {
            is_authorized: auth.is_authorized,
            auth_type: auth.auth_type,
            driver_name: auth.driver_name,
            driver_phone: auth.driver_phone,
            valid_until: auth.valid_until.unwrap_or_default(),
        }))
    }
}

fn parse_pass_time(time_str: &str) -> NaiveDateTime {
    if time_str.is_empty() { return chrono::Utc::now().naive_utc(); }
    if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S") { return dt; }
    if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S%.f") { return dt; }
    chrono::Utc::now().naive_utc()
}
