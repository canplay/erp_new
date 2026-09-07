// ============== Tow 服务 gRPC 调用封装 ==============

pub type TowGrpcClient = GrpcClientWrapper<grpc_proto::tow::tow_service_client::TowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;

impl TowGrpcClient {
    pub async fn list_tow_cars(&mut self, keyword: &str, status: &str, page: i32, page_size: i32) -> Result<grpc_proto::tow::ListTowCarsResponse, tonic::Status> {
        Ok(self.inner.list_tow_cars(grpc_proto::tow::ListTowCarsRequest {
            keyword: keyword.to_string(), status: status.to_string(),
            car_type: String::new(), page, page_size,
        }).await?.into_inner())
    }

    pub async fn get_tow_car(&mut self, id: i64) -> Result<grpc_proto::tow::TowCarInfo, tonic::Status> {
        Ok(self.inner.get_tow_car(grpc_proto::tow::GetTowCarRequest { id }).await?.into_inner())
    }

    pub async fn list_dict_items(&mut self, dict_type: &str) -> Result<grpc_proto::tow::ListDictItemsResponse, tonic::Status> {
        Ok(self.inner.list_dict_items(grpc_proto::tow::ListDictItemsRequest { dict_type: dict_type.to_string() }).await?.into_inner())
    }
}

