// ============== CTP 服务 gRPC 调用封装 ==============

impl CtpGrpcClient {
    pub async fn receive_device_data(&mut self, device_no: String, data_type: i32, voltage: Option<String>, status_one: Option<String>, status_two: Option<String>, data_time: Option<String>) -> Result<grpc_proto::ctp::CtpResponse, tonic::Status> {
        Ok(self.inner().clone().receive_device_data(tonic::Request::new(grpc_proto::ctp::DeviceDataUpload { device_no, data_type, voltage, status_one, status_two, data_time })).await?.into_inner())
    }
    pub async fn control_lock(&mut self, factory_id: String, device_no: String, cmd_type: String, data: Option<String>) -> Result<grpc_proto::ctp::CommandResult, tonic::Status> {
        Ok(self.inner().clone().control_lock(tonic::Request::new(grpc_proto::ctp::LockControlRequest { factory_id, device_no, cmd_type, data })).await?.into_inner())
    }
    pub async fn get_device(&mut self, device_no: String) -> Result<grpc_proto::ctp::LockDevice, tonic::Status> {
        Ok(self.inner().clone().get_device(tonic::Request::new(grpc_proto::ctp::GetDeviceRequest { device_no })).await?.into_inner())
    }
    pub async fn list_devices(&mut self, park_code: Option<String>, status: Option<String>, page: i32, page_size: i32) -> Result<grpc_proto::ctp::DeviceListResponse, tonic::Status> {
        Ok(self.inner().clone().list_devices(tonic::Request::new(grpc_proto::ctp::DeviceQueryRequest { park_code, status, page, page_size })).await?.into_inner())
    }
}

