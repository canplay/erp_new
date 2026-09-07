// ============== Clean 服务 gRPC 调用封装 ==============

impl CleanGrpcClient {
    pub async fn list_invoices(
        &mut self,
        page: i32,
        page_size: i32,
        no: Option<String>,
        imposing_no: Option<i32>,
        imposing_name: Option<String>,
        collection_name: Option<String>,
        fingerprint: Option<String>,
        zone: Option<String>,
    ) -> Result<grpc_proto::clean::InvoiceListResponse, tonic::Status> {
        let request = grpc_proto::clean::InvoiceQueryRequest {
            cur_page: Some(page), max_page: Some(page_size),
            no, imposing_no, imposing_name, collection_name,
            fingerprint, zone,
            id: None, create_date: None, sort_by: None, descending: None,
        };
        Ok(self.inner.get_invoice_list(request).await?.into_inner())
    }

    pub async fn list_orders(
        &mut self,
        page: i32,
        page_size: i32,
        serial_number: Option<String>,
        numbering: Option<String>,
        cashier: Option<String>,
    ) -> Result<grpc_proto::clean::OrderListResponse, tonic::Status> {
        let request = grpc_proto::clean::OrderQueryRequest {
            cur_page: Some(page), max_page: Some(page_size),
            serial_number, numbering, cashier,
            id: None, payment_time: None, sort_by: None, descending: None,
        };
        Ok(self.inner.get_order_list(request).await?.into_inner())
    }

    pub async fn list_formal_bills(
        &mut self,
        page: i32,
        page_size: i32,
        numbering: Option<String>,
        fzr: Option<String>,
        status: Option<String>,
        create_date_start: Option<String>,
        create_date_end: Option<String>,
    ) -> Result<grpc_proto::clean::FormalBillListResponse, tonic::Status> {
        let request = grpc_proto::clean::FormalBillQueryRequest {
            cur_page: Some(page), max_page: Some(page_size),
            numbering, fzr, status, create_date_start, create_date_end,
            sort_by: None, descending: None,
        };
        Ok(self.inner.get_formal_bill_list(request).await?.into_inner())
    }

    pub async fn list_payment_webs(
        &mut self,
        page: i32,
        page_size: i32,
        numbering: Option<String>,
        r#type: Option<String>,
        status: Option<String>,
        create_date_start: Option<String>,
        create_date_end: Option<String>,
        payment_date_start: Option<String>,
        payment_date_end: Option<String>,
    ) -> Result<grpc_proto::clean::PaymentWebListResponse, tonic::Status> {
        let request = grpc_proto::clean::PaymentWebQueryRequest {
            cur_page: Some(page), max_page: Some(page_size),
            numbering, r#type, status,
            create_date_start, create_date_end,
            payment_date_start, payment_date_end,
            sort_by: None, descending: None,
        };
        Ok(self.inner.get_payment_web_list(request).await?.into_inner())
    }

    pub async fn get_statistics(
        &mut self,
        statistics_type: String,
    ) -> Result<grpc_proto::clean::StatisticsResponse, tonic::Status> {
        let request = grpc_proto::clean::StatisticsRequest {
            statistics_type,
            formal_bill_query: None,
            payment_web_query: None,
        };
        Ok(self.inner.get_statistics(request).await?.into_inner())
    }
}
