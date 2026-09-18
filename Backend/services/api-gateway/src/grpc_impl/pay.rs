// ============== Pay 服务 gRPC 调用封装 ==============

impl PayGrpcClient {
    pub async fn count(
        &mut self,
        status: String,
        pay_type: String,
        remark: String,
    ) -> Result<grpc_proto::pay::CountResponse, tonic::Status> {
        let request = grpc_proto::pay::PayQuery {
            status: Some(status), pay_type: Some(pay_type), remark: Some(remark),
            sort_by: None, descending: None, max_page: None, cur_page: None,
        };
        Ok(self.inner.count(request).await?.into_inner())
    }

    pub async fn list(
        &mut self,
        status: String,
        pay_type: String,
        remark: String,
        sort_by: String,
        descending: bool,
        page: i64,
        page_size: i64,
    ) -> Result<grpc_proto::pay::OrderListResponse, tonic::Status> {
        let request = grpc_proto::pay::PayQuery {
            status: Some(status), pay_type: Some(pay_type), remark: Some(remark),
            sort_by: Some(sort_by), descending: Some(descending),
            max_page: Some(page_size), cur_page: Some(page),
        };
        Ok(self.inner.list(request).await?.into_inner())
    }

    pub async fn latest(
        &mut self,
        user_id: String,
    ) -> Result<grpc_proto::pay::PayOrderResponse, tonic::Status> {
        let request = grpc_proto::pay::LatestRequest { user_id };
        Ok(self.inner.latest(request).await?.into_inner())
    }

    pub async fn create_order(
        &mut self,
        order: String,
        status: String,
        pay_type: String,
        order_pay_json: String,
        amount: i32,
        remark: String,
        create_params_json: String,
    ) -> Result<grpc_proto::pay::PayOrderResponse, tonic::Status> {
        let request = grpc_proto::pay::PayCreateParams {
            order: order, status: Some(status), pay_type: Some(pay_type),
            order_pay_json: Some(order_pay_json), amount: Some(amount),
            remark: Some(remark), create_params_json: Some(create_params_json),
            create_date: None,
        };
        Ok(self.inner.create_order(request).await?.into_inner())
    }

    pub async fn ccb_query(
        &mut self,
        order_id: String,
    ) -> Result<grpc_proto::pay::CcbQueryResponse, tonic::Status> {
        let request = grpc_proto::pay::CcbQueryParams { order_id };
        Ok(self.inner.ccb_query(request).await?.into_inner())
    }

    pub async fn ccb_create(
        &mut self,
        order_id: String,
        amount: i32,
        subject: String,
    ) -> Result<grpc_proto::pay::CcbOrderResponse, tonic::Status> {
        let request = grpc_proto::pay::CcbOrderParams { order_id, amount, subject };
        Ok(self.inner.ccb_create(request).await?.into_inner())
    }

    pub async fn ccb_verify(
        &mut self,
        order_id: String,
    ) -> Result<grpc_proto::pay::BoolResponse, tonic::Status> {
        let request = grpc_proto::pay::CcbVerifyRequest { order_id };
        Ok(self.inner.ccb_verify(request).await?.into_inner())
    }

    pub async fn ccb_refund(
        &mut self,
        order_id: String,
        amount: i64,
    ) -> Result<grpc_proto::pay::BoolResponse, tonic::Status> {
        let request = grpc_proto::pay::CcbRefundRequest { order_id, amount: amount as f64 };
        Ok(self.inner.ccb_refund(request).await?.into_inner())
    }

    pub async fn ums_query(
        &mut self,
        order_id: String,
    ) -> Result<grpc_proto::pay::JsonValueResponse, tonic::Status> {
        let request = grpc_proto::pay::UmsQueryParams { order_id };
        Ok(self.inner.ums_query(request).await?.into_inner())
    }

    pub async fn ums_create(
        &mut self,
        order_id: String,
        amount: i32,
        subject: String,
        description: String,
    ) -> Result<grpc_proto::pay::JsonValueResponse, tonic::Status> {
        let request = grpc_proto::pay::UmsOrderParams { order_id, amount, subject, description };
        Ok(self.inner.ums_create(request).await?.into_inner())
    }

    pub async fn ums_close(
        &mut self,
        order_id: String,
    ) -> Result<grpc_proto::pay::JsonValueResponse, tonic::Status> {
        let request = grpc_proto::pay::UmsCloseParams { order_id };
        Ok(self.inner.ums_close(request).await?.into_inner())
    }

    pub async fn ums_refund(
        &mut self,
        order_id: String,
        amount: i64,
        reason: String,
    ) -> Result<grpc_proto::pay::JsonValueResponse, tonic::Status> {
        let request = grpc_proto::pay::UmsRefundParams { order_id, amount: amount as f64, reason };
        Ok(self.inner.ums_refund(request).await?.into_inner())
    }

    pub async fn ums_info(
        &mut self,
        order: String,
    ) -> Result<grpc_proto::pay::JsonValueResponse, tonic::Status> {
        let request = grpc_proto::pay::UmsInfoRequest { order };
        Ok(self.inner.ums_info(request).await?.into_inner())
    }

    pub async fn ums_notify(
        &mut self,
        order: String,
        time: Option<String>,
    ) -> Result<grpc_proto::pay::BoolResponse, tonic::Status> {
        let request = grpc_proto::pay::UmsNotifyRequest { order, time };
        Ok(self.inner.ums_notify(request).await?.into_inner())
    }
}

