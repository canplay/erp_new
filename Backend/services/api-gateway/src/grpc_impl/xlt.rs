// ============== XLT 服务 gRPC 调用封装 ==============

impl XltGrpcClient {
    pub async fn vehicle_entry(
        &mut self, plate_no: String, plate_color: String, park_code: String,
        lane_code: String, event_time: String, vehicle_type: String,
        image_url: String, amount: i64, pay_type: String,
    ) -> Result<grpc_proto::xlt::GenericResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::VehicleEvent {
            plate_no, plate_color, park_code, lane_code, event_time,
            vehicle_type, image_url, amount, pay_type,
        });
        Ok(self.inner().clone().vehicle_entry(request).await?.into_inner())
    }
    pub async fn vehicle_exit(
        &mut self, plate_no: String, plate_color: String, park_code: String,
        lane_code: String, event_time: String, vehicle_type: String,
        image_url: String, amount: i64, pay_type: String,
    ) -> Result<grpc_proto::xlt::GenericResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::VehicleEvent {
            plate_no, plate_color, park_code, lane_code, event_time,
            vehicle_type, image_url, amount, pay_type,
        });
        Ok(self.inner().clone().vehicle_exit(request).await?.into_inner())
    }
    pub async fn get_parking_vehicle(
        &mut self, park_code: String, plate_no: String,
    ) -> Result<grpc_proto::xlt::ParkingVehicle, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::GetParkingVehicleRequest { park_code, plate_no });
        Ok(self.inner().clone().get_parking_vehicle(request).await?.into_inner())
    }
    pub async fn calc_billing(
        &mut self, plate_no: String, park_code: String,
        entry_time: String, exit_time: String,
    ) -> Result<grpc_proto::xlt::BillingResult, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::BillingRequest { plate_no, park_code, entry_time, exit_time });
        Ok(self.inner().clone().calc_billing(request).await?.into_inner())
    }
    pub async fn list_records(
        &mut self, plate_no: Option<String>, park_code: Option<String>,
        status: Option<String>, page: i32, page_size: i32,
    ) -> Result<grpc_proto::xlt::RecordListResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::VehicleQueryRequest { plate_no, park_code, status, page, page_size });
        Ok(self.inner().clone().list_records(request).await?.into_inner())
    }
    pub async fn list_devices(&mut self) -> Result<grpc_proto::xlt::DeviceInfoList, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::Empty {});
        Ok(self.inner().clone().list_devices(request).await?.into_inner())
    }
    pub async fn open_barrier(
        &mut self, sn: String, request_id: String,
    ) -> Result<grpc_proto::xlt::GenericResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::BarrierCommand { sn, request_id });
        Ok(self.inner().clone().open_barrier(request).await?.into_inner())
    }
    pub async fn close_barrier(
        &mut self, sn: String, request_id: String,
    ) -> Result<grpc_proto::xlt::GenericResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::xlt::BarrierCommand { sn, request_id });
        Ok(self.inner().clone().close_barrier(request).await?.into_inner())
    }
}

