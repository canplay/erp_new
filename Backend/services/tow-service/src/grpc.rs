//! Tow 服务 gRPC 处理器
//! 实现 TowService：车辆列表/详情/字典查询

use std::net::SocketAddr;
use grpc_proto::tow::tow_service_server::TowServiceServer;
use tonic::{Request, Response, Status};
use sqlx::PgPool;

use grpc_proto::tow::{
    tow_service_server::TowService,
    ListTowCarsRequest, ListTowCarsResponse, TowCarInfo,
    GetTowCarRequest,
    ListDictItemsRequest, ListDictItemsResponse, DictItem,
};

pub struct TowGrpcService {
    pub pool: PgPool,
}

impl TowGrpcService {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl TowService for TowGrpcService {
    async fn list_tow_cars(&self, request: Request<ListTowCarsRequest>) -> Result<Response<ListTowCarsResponse>, Status> {
        let req = request.into_inner();
        let repo = crate::repository::CarRepository::new(self.pool.clone());
        let query = crate::repository::CarQuery {
            content: if req.keyword.is_empty() { None } else { Some(req.keyword) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            model: None, name: None, unit: None, key: None,
            sort_by: None, descending: None,
            in_date: None, out_date: None,
            page: Some(req.page.max(1)),
            page_size: Some(req.page_size.max(20)),
        };
        let result = repo.list(&query).await.map_err(|e| Status::internal(format!("{e}")))?;
        let cars: Vec<TowCarInfo> = result.cars.into_iter().map(|c| TowCarInfo {
            id: c.id, license: c.license, car_type: c.car_type,
            car_color: c.car_color, dc_date: c.dc_date,
            dc_address: c.dc_address, dc_party_name: c.dc_party_name,
            dc_type: c.dc_type, dc_causes: c.dc_causes, delete: c.delete,
            ..Default::default()
        }).collect();
        Ok(Response::new(ListTowCarsResponse { cars, total: result.total, page: req.page, page_size: req.page_size }))
    }

    async fn get_tow_car(&self, request: Request<GetTowCarRequest>) -> Result<Response<TowCarInfo>, Status> {
        let req = request.into_inner();
        let repo = crate::repository::CarRepository::new(self.pool.clone());
        let car = repo.find_by_id(req.id).await
            .map_err(|_| Status::internal("查询失败"))?
            .ok_or_else(|| Status::not_found("车辆未找到"))?;
        Ok(Response::new(TowCarInfo {
            id: car.id, license: car.license, car_type: car.car_type,
            car_color: car.car_color, engine: car.engine,
            dc_type: car.dc_type, dc_causes: car.dc_causes,
            dc_date: car.dc_date, dc_address: car.dc_address,
            dc_key: car.dc_key,
            dc_party_name: car.dc_party_name, dc_party_cardid: car.dc_party_cardid, dc_party_tel: car.dc_party_tel,
            p_name: car.p_name, driver: car.driver, operator: car.operator,
            remark: car.remark, delete: car.delete,
            create_date: car.create_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            update_date: car.update_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        }))
    }

    async fn list_dict_items(&self, request: Request<ListDictItemsRequest>) -> Result<Response<ListDictItemsResponse>, Status> {
        let req = request.into_inner();
        let items: Vec<DictItem> = match req.dict_type.as_str() {
            "car_type" => {
                let r = crate::repository::CarTypeRepository::new(self.pool.clone());
                r.list().await.map_err(|e| Status::internal(format!("{e}")))?
                    .into_iter().map(|d| DictItem { id: d.id, name: d.type_name.clone(), value: d.type_name, sort_order: d.index as i32 }).collect()
            }
            "car_color" => {
                let r = crate::repository::CarColorRepository::new(self.pool.clone());
                r.list().await.map_err(|e| Status::internal(format!("{e}")))?
                    .into_iter().map(|d| DictItem { id: d.id, name: d.name.clone(), value: d.name, sort_order: d.sort_order.unwrap_or(0) }).collect()
            }
            "dc_causes" => {
                let r = crate::repository::DcCausesRepository::new(self.pool.clone());
                r.list().await.map_err(|e| Status::internal(format!("{e}")))?
                    .into_iter().map(|d| DictItem { id: d.id, name: d.name.clone(), value: d.name, sort_order: d.sort_order.unwrap_or(0) }).collect()
            }
            _ => vec![]
        };
        Ok(Response::new(ListDictItemsResponse { items }))
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for TowGrpcService {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| format!("无效的 gRPC 地址 '{grpc_addr}': {e}"))?;
        let server = TowServiceServer::new(TowGrpcService::new(self.pool.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
