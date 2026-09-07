// ============== LPR 服务 gRPC 调用封装 ==============

impl LprGrpcClient {
    /// 车牌识别回调 — 代理相机推送的识别结果至 lpr-service
    pub async fn lpr_callback(
        &mut self,
        plate_no: String,
        plate_color: String,
        plate_type: String,
        vehicle_type: String,
        pass_time: String,
        image_url: String,
        device_id: String,
        park_code: String,
        direction: String,
        confidence: f64,
        device_name: String,
        lane_code: String,
    ) -> Result<grpc_proto::lpr::LprCallbackResponse, tonic::Status> {
        let request = grpc_proto::lpr::LprCallbackRequest {
            plate_no,
            plate_color,
            plate_type,
            vehicle_type,
            pass_time,
            image_url,
            device_id,
            park_code,
            direction,
            confidence,
            device_name,
            lane_code,
        };
        Ok(self.inner.lpr_callback(request).await?.into_inner())
    }

    /// 查询通行记录列表
    pub async fn list_pass_records(&mut self, plate_no: &str, park_code: &str, direction: &str, status: &str, page: i32, page_size: i32) -> Result<grpc_proto::lpr::ListPassRecordsResponse, tonic::Status> {
        Ok(self.inner.list_pass_records(grpc_proto::lpr::ListPassRecordsRequest {
            plate_no: plate_no.to_string(), park_code: park_code.to_string(),
            direction: direction.to_string(), status: status.to_string(),
            device_id: String::new(), start_time: String::new(), end_time: String::new(),
            page, page_size,
        }).await?.into_inner())
    }

    /// 获取通行记录详情
    pub async fn get_pass_record(&mut self, id: i64) -> Result<grpc_proto::lpr::PassRecordInfo, tonic::Status> {
        Ok(self.inner.get_pass_record(grpc_proto::lpr::GetPassRecordRequest { id }).await?.into_inner())
    }

    /// 获取通行统计
    pub async fn get_pass_stats(&mut self, park_code: &str, start_date: &str, end_date: &str) -> Result<grpc_proto::lpr::PassStatsResponse, tonic::Status> {
        Ok(self.inner.get_pass_stats(grpc_proto::lpr::GetPassStatsRequest {
            park_code: park_code.to_string(), start_date: start_date.to_string(), end_date: end_date.to_string(),
        }).await?.into_inner())
    }

    /// 查询车辆授权
    pub async fn get_vehicle_auth(&mut self, plate_no: &str, park_code: &str) -> Result<grpc_proto::lpr::VehicleAuthInfo, tonic::Status> {
        Ok(self.inner.get_vehicle_auth(grpc_proto::lpr::GetVehicleAuthRequest {
            plate_no: plate_no.to_string(), park_code: park_code.to_string(),
        }).await?.into_inner())
    }
}

