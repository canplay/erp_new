use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use chrono::Utc;

use grpc_proto::pay::pay_service_server::{PayService, PayServiceServer};
use grpc_proto::pay::{
    BoolResponse, CcbOrderParams, CcbOrderResponse, CcbQueryParams, CcbQueryResponse,
    CcbRefundRequest, CcbVerifyRequest, CountResponse, JsonValueResponse, LatestRequest,
    OrderListResponse, PayCreateParams, PayOrder as ProtoPayOrder, PayOrderResponse, PayQuery,
    UmsCloseParams, UmsInfoRequest, UmsOrderParams, UmsQueryParams, UmsRefundParams,
};

use crate::AppState;
use crate::services::ccb::{CcbConfig, CcbService};
use crate::services::ums::{UmsConfig, UmsService};

pub async fn start_grpc_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("[PayService] gRPC listening on {addr}" );
    Server::builder().layer(tonic::service::interceptor::InterceptorLayer::new(common::grpc_auth_interceptor))
        .add_service(PayServiceServer::new(GrpcPayService { state: Arc::new(state) }))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}

use tonic::transport::Server;
use common::shutdown_signal;

#[derive(Clone)]
pub struct GrpcPayService {
    pub state: Arc<AppState>,
}

fn pay_order_to_proto(o: crate::models::PayOrder) -> ProtoPayOrder {
    ProtoPayOrder {
        id: o.id,
        order: o.order,
        status: o.status,
        pay_type: o.pay_type,
        order_pay_json: o.order_pay.map(|v| v.to_string()).unwrap_or_default(),
        amount: o.amount,
        remark: o.remark,
        create_service: o.create_service,
        create_params_json: o.create_params.map(|v| v.to_string()).unwrap_or_default(),
        create_date: o.create_date,
        update_date: o.update_date,
    }
}

fn create_ccb_config() -> CcbConfig {
    CcbConfig {
        merchantid: std::env::var("CCB_MERCHANT_ID" ).unwrap_or_default(),
        branchid: std::env::var("CCB_BRANCH_ID" ).unwrap_or_default(),
        posid: std::env::var("CCB_POS_ID" ).unwrap_or_default(),
        qupwd: std::env::var("CCB_QUPWD" ).unwrap_or_default(),
        pub_key: std::env::var("CCB_PUB_KEY" ).unwrap_or_default(),
    }
}

fn create_ums_config() -> UmsConfig {
    UmsConfig {
        appid: std::env::var("UMS_APP_ID" ).unwrap_or_default(),
        appkey: std::env::var("UMS_APP_KEY" ).unwrap_or_default(),
        mid: std::env::var("UMS_MID" ).unwrap_or_default(),
        tid: std::env::var("UMS_TID" ).unwrap_or_default(),
        ysjc: std::env::var("UMS_YSJC" ).ok(),
    }
}

fn now_date_str() -> String {
    Utc::now().format("%Y-%m-%d" ).to_string()
}

fn now_time_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S" ).to_string()
}

#[tonic::async_trait]
impl PayService for GrpcPayService {
    async fn count(&self, request: Request<PayQuery>) -> Result<Response<CountResponse>, Status> {
        let req = request.into_inner();
        let service_query = crate::models::PayQuery {
            status: req.status,
            pay_type: req.pay_type,
            remark: req.remark,
            sort_by: req.sort_by,
            descending: req.descending,
            max_page: req.max_page,
            cur_page: req.cur_page,
        };
        match self.state.pay_service.count(&service_query).await {
            Ok(count) => Ok(Response::new(CountResponse { count, message: "success".to_string() })),
            Err(e) => Err(Status::internal(format!("{e}" ))),
        }
    }

    async fn list(&self, request: Request<PayQuery>) -> Result<Response<OrderListResponse>, Status> {
        let req = request.into_inner();
        let service_query = crate::models::PayQuery {
            status: req.status,
            pay_type: req.pay_type,
            remark: req.remark,
            sort_by: req.sort_by,
            descending: req.descending,
            max_page: req.max_page,
            cur_page: req.cur_page,
        };
        match self.state.pay_service.list(&service_query).await {
            Ok(orders) => Ok(Response::new(OrderListResponse {
                orders: orders.into_iter().map(pay_order_to_proto).collect(),
                message: "success".to_string(),
            })),
            Err(e) => Err(Status::internal(format!("{e}" ))),
        }
    }

    async fn latest(&self, request: Request<LatestRequest>) -> Result<Response<PayOrderResponse>, Status> {
        let user_id = request.into_inner().user_id;
        match self.state.pay_service.latest(&user_id).await {
            Ok(Some(order)) => Ok(Response::new(PayOrderResponse {
                order: Some(pay_order_to_proto(order)),
                message: "success".to_string(),
            })),
            Ok(None) => Err(Status::not_found("no order found" )),
            Err(e) => Err(Status::internal(format!("{e}" ))),
        }
    }

    async fn create_order(&self, request: Request<PayCreateParams>) -> Result<Response<PayOrderResponse>, Status> {
        let req = request.into_inner();
        let params = crate::models::PayCreateParams {
            order: req.order,
            status: req.status,
            pay_type: req.pay_type,
            order_pay: None,
            amount: req.amount,
            remark: req.remark,
            create_params: None,
            create_date: req.create_date,
        };
        match self.state.pay_service.create_order(&params).await {
            Ok(order) => Ok(Response::new(PayOrderResponse {
                order: Some(pay_order_to_proto(order)),
                message: "success".to_string(),
            })),
            Err(e) => Err(Status::internal(format!("{e}" ))),
        }
    }

    async fn ccb_query(&self, request: Request<CcbQueryParams>) -> Result<Response<CcbQueryResponse>, Status> {
        let params = request.into_inner();
        let service = CcbService::new(create_ccb_config());
        let svc_params = crate::services::ccb::CcbQueryParams {
            posid: Some(params.order_id.clone()),
            date: None,
            time_start: None,
            time_end: None,
            order_no: Some(params.order_id),
            query_type: None,
            kind: None,
            status: None,
            page: None,
        };
        service.query(&svc_params).await
            .map(|resp| Response::new(CcbQueryResponse { data_json: serde_json::to_string(&resp).unwrap_or_default() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ccb_create(&self, request: Request<CcbOrderParams>) -> Result<Response<CcbOrderResponse>, Status> {
        let params = request.into_inner();
        let service = CcbService::new(create_ccb_config());
        let svc_params = crate::services::ccb::CcbOrderParams {
            order: params.order_id,
            amount: (params.amount * 100) as i64,
            remark: None,
            remark1: None,
            remark2: None,
            goods: Some(params.subject),
            date: None,
            order_pay: None,
            no: None,
        };
        service.create_order(&svc_params).await
            .map(|resp| Response::new(CcbOrderResponse { data_json: serde_json::to_string(&resp).unwrap_or_default() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ccb_verify(&self, request: Request<CcbVerifyRequest>) -> Result<Response<BoolResponse>, Status> {
        let order_id = request.into_inner().order_id;
        let service = CcbService::new(create_ccb_config());
        service.verify_payment(&order_id).await
            .map(|success| Response::new(BoolResponse { success, message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ccb_refund(&self, request: Request<CcbRefundRequest>) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        let service = CcbService::new(create_ccb_config());
        service.refund(&req.order_id, req.amount as i64).await
            .map(|success| Response::new(BoolResponse { success, message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ums_query(&self, request: Request<UmsQueryParams>) -> Result<Response<JsonValueResponse>, Status> {
        let params = request.into_inner();
        let service = UmsService::new(create_ums_config(), self.state.pay_service.pool.clone());
        let token = service.get_access_token().await.map_err(|e| Status::internal(e.to_string()))?;
        let svc_params = crate::services::ums::UmsQueryParams {
            time: now_date_str(),
            no: Some(params.order_id),
        };
        service.query(&svc_params, &token).await
            .map(|data| Response::new(JsonValueResponse { data_json: data.to_string(), message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ums_create(&self, request: Request<UmsOrderParams>) -> Result<Response<JsonValueResponse>, Status> {
        let params = request.into_inner();
        let service = UmsService::new(create_ums_config(), self.state.pay_service.pool.clone());
        let token = service.get_access_token().await.map_err(|e| Status::internal(e.to_string()))?;
        let svc_params = crate::services::ums::UmsOrderParams {
            time: now_time_str(),
            no: params.order_id,
            no_pay: None,
            amount: (params.amount * 100) as i64,
            desc: Some(params.subject),
            return_url: None,
            notify: None,
            zone: None,
        };
        service.create_order(&svc_params, &token).await
            .map(|data| Response::new(JsonValueResponse { data_json: data.to_string(), message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ums_close(&self, request: Request<UmsCloseParams>) -> Result<Response<JsonValueResponse>, Status> {
        let params = request.into_inner();
        let service = UmsService::new(create_ums_config(), self.state.pay_service.pool.clone());
        let token = service.get_access_token().await.map_err(|e| Status::internal(e.to_string()))?;
        let svc_params = crate::services::ums::UmsCloseParams {
            no: params.order_id,
            time: now_time_str(),
        };
        service.close(&svc_params, &token).await
            .map(|data| Response::new(JsonValueResponse { data_json: data.to_string(), message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ums_refund(&self, request: Request<UmsRefundParams>) -> Result<Response<JsonValueResponse>, Status> {
        let params = request.into_inner();
        let service = UmsService::new(create_ums_config(), self.state.pay_service.pool.clone());
        let token = service.get_access_token().await.map_err(|e| Status::internal(e.to_string()))?;
        let svc_params = crate::services::ums::UmsRefundParams {
            time: now_time_str(),
            no: params.order_id.clone(),
            desc: Some(params.reason),
            amount: params.amount as i64,
            refundno: format!("REFUND_{}" , params.order_id),
            zone: None,
        };
        service.refund(&svc_params, &token).await
            .map(|data| Response::new(JsonValueResponse { data_json: data.to_string(), message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }

    async fn ums_info(&self, request: Request<UmsInfoRequest>) -> Result<Response<JsonValueResponse>, Status> {
        let order = request.into_inner().order;
        let service = UmsService::new(create_ums_config(), self.state.pay_service.pool.clone());
        let token = service.get_access_token().await.map_err(|e| Status::internal(e.to_string()))?;
        service.query_ums_info(&order, &token).await
            .map(|data| Response::new(JsonValueResponse { data_json: data.to_string(), message: "success".to_string() }))
            .map_err(|e| Status::internal(format!("{e}" )))
    }
}