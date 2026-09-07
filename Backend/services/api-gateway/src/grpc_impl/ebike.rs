// ============== Ebike 服务 gRPC 调用封装 ==============

impl EbikeGrpcClient {
    pub async fn car_query(
        &mut self, code: String, provide: String, status: i64,
        time_start: String, time_end: String,
    ) -> Result<grpc_proto::ebike::CarListResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::ebike::CarQueryRequest {
            code, provide, status, time_start, time_end,
        });
        Ok(self.inner().clone().car_query(request).await?.into_inner())
    }
    pub async fn order_query(
        &mut self, code: String, provide: String, status: i64,
        time_start: String, time_end: String, order: String,
        paystatus: i64, paytype: i64, paytime: String,
    ) -> Result<grpc_proto::ebike::OrderListResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::ebike::OrderQueryRequest {
            code, provide, status, time_start, time_end,
            order, paystatus, paytype, paytime,
        });
        Ok(self.inner().clone().order_query(request).await?.into_inner())
    }
    pub async fn login(
        &mut self, username: String, password: String,
    ) -> Result<grpc_proto::ebike::LoginResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::ebike::LoginRequest { username, password });
        Ok(self.inner().clone().login(request).await?.into_inner())
    }

    pub async fn car_add(
        &mut self, req: grpc_proto::ebike::CarAddRequest,
    ) -> Result<grpc_proto::ebike::GenericResponse, tonic::Status> {
        Ok(self.inner().clone().car_add(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn car_batch_add(
        &mut self, req: grpc_proto::ebike::CarBatchAddRequest,
    ) -> Result<grpc_proto::ebike::GenericResponse, tonic::Status> {
        Ok(self.inner().clone().car_batch_add(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn car_delete(
        &mut self, code: String, provide: String,
    ) -> Result<grpc_proto::ebike::GenericResponse, tonic::Status> {
        let req = grpc_proto::ebike::CarDeleteRequest { code, provide };
        Ok(self.inner().clone().car_delete(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn car_history(
        &mut self, code: String,
    ) -> Result<grpc_proto::ebike::CarListResponse, tonic::Status> {
        let req = grpc_proto::ebike::CarHistoryRequest { code };
        Ok(self.inner().clone().car_history(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn car_alert(
        &mut self, req: grpc_proto::ebike::CarAlertRequest,
    ) -> Result<grpc_proto::ebike::CarListResponse, tonic::Status> {
        Ok(self.inner().clone().car_alert(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn storage_query(
        &mut self, code: String, provide: String, status: i64,
    ) -> Result<grpc_proto::ebike::StorageListResponse, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::ebike::StorageQueryRequest {
            code, provide, status,
        });
        Ok(self.inner().clone().storage_query(request).await?.into_inner())
    }

    pub async fn storage_add(
        &mut self, req: grpc_proto::ebike::StorageAddRequest,
    ) -> Result<grpc_proto::ebike::GenericResponse, tonic::Status> {
        Ok(self.inner().clone().storage_add(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn storage_delete(
        &mut self, code: String,
    ) -> Result<grpc_proto::ebike::GenericResponse, tonic::Status> {
        let req = grpc_proto::ebike::StorageDeleteRequest { code };
        Ok(self.inner().clone().storage_delete(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn storage_history(
        &mut self, code: String,
    ) -> Result<grpc_proto::ebike::StorageListResponse, tonic::Status> {
        let req = grpc_proto::ebike::StorageHistoryRequest { code };
        Ok(self.inner().clone().storage_history(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn options_query(
        &mut self,
    ) -> Result<grpc_proto::ebike::OptionsInfo, tonic::Status> {
        let request = tonic::Request::new(grpc_proto::ebike::Empty {});
        Ok(self.inner().clone().options_query(request).await?.into_inner())
    }

    pub async fn options_update(
        &mut self, req: grpc_proto::ebike::OptionsUpdateRequest,
    ) -> Result<grpc_proto::ebike::GenericResponse, tonic::Status> {
        Ok(self.inner().clone().options_update(tonic::Request::new(req)).await?.into_inner())
    }

    pub async fn order_add(
        &mut self, req: grpc_proto::ebike::OrderAddRequest,
    ) -> Result<grpc_proto::ebike::OrderAddResponse, tonic::Status> {
        Ok(self.inner().clone().order_add(tonic::Request::new(req)).await?.into_inner())
    }
}
