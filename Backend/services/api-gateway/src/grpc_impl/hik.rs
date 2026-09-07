// ============== HIK 服务 gRPC 调用封装 ==============

impl HikGrpcClient {
    pub async fn exec(
        &mut self,
        method: String,
        phone: Option<i64>,
        driver_id: Option<String>,
        plate_no: Option<String>,
        plate_color: Option<String>,
        page_no: Option<i32>,
        page_size: Option<i32>,
        park_name: Option<String>,
        park_code: Option<String>,
        request_type: Option<String>,
        unique_id: Option<String>,
        is_use_coupon: Option<String>,
        appeal_type: Option<String>,
        appeal_remark: Option<String>,
        appeal_in_time: Option<String>,
        appeal_out_time: Option<String>,
        appeal_source: Option<String>,
        arrears_ids: Option<String>,
    ) -> Result<grpc_proto::hik::GenericResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::hik::HikRequest {
            method, phone, driver_id, plate_no, plate_color,
            page_no, page_size, park_name, park_code, request_type,
            unique_id, is_use_coupon, appeal_type, appeal_remark,
            appeal_in_time, appeal_out_time, appeal_source, arrears_ids,
        });
        Ok(self.inner().clone().exec(request).await?.into_inner())
    }
    pub async fn coupon(
        &mut self,
        phone: String,
        amount: i32,
        start: i64,
        end: String,
        r#type: String,
    ) -> Result<grpc_proto::hik::GenericResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::hik::CouponRequest { phone, amount, start, end, r#type });
        Ok(self.inner().clone().coupon(request).await?.into_inner())
    }
    pub async fn signo_open(
        &mut self,
        place: String,
        name: String,
    ) -> Result<grpc_proto::hik::SignoResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::hik::SignoRequest { place, name });
        Ok(self.inner().clone().signo_open(request).await?.into_inner())
    }
}

