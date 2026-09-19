use std::sync::Arc;
use tonic::{Request, Response, Status};

use grpc_proto::ebike::{
    CarAddRequest, CarInfo, CarListResponse, CarQueryRequest,
    OrderAddRequest, OrderInfo, OrderListResponse,
    StorageAddRequest, StorageInfo, StorageListResponse, StorageQueryRequest,
    OptionsInfo, OptionsUpdateRequest, Empty, GenericResponse,
    LoginRequest, LoginResponse, UserBrief,
    ebike_service_server::EbikeService as EbikeServiceTrait,
};

use crate::AppState;
use crate::model::{CarInfo as ModelCarInfo, OrderInfo as ModelOrderInfo, StorageInfo as ModelStorageInfo};

/// gRPC EbikeService 实现
#[derive(Clone)]
pub struct EbikeGrpcService {
    pub state: Arc<AppState>,
}

impl EbikeGrpcService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

fn car_to_proto(c: &ModelCarInfo) -> CarInfo {
    CarInfo {
        code: c.code.clone(),
        status: c.status,
        provide: c.provide.clone(),
        speed: c.speed,
        gps_json: c.gps.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        r#type: c.r#type,
        time_json: c.time.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        create_date: c.create_date.map(|d| d.to_string()).unwrap_or_default(),
        update_date: c.update_date.map(|d| d.to_string()).unwrap_or_default(),
        delete: c.delete.unwrap_or(false),
        alert: c.alert.clone().unwrap_or_default(),
        remark: c.remark.clone().unwrap_or_default(),
        gps_type: c.gps_type,
    }
}

fn order_to_proto(o: &ModelOrderInfo) -> OrderInfo {
    OrderInfo {
        code: o.code.clone().unwrap_or_default(),
        status: o.status,
        provide: o.provide.clone(),
        speed: o.speed,
        gps_json: o.gps.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        r#type: o.r#type,
        time_json: o.time.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        create_date: o.create_date.map(|d| d.to_string()).unwrap_or_default(),
        update_date: o.update_date.map(|d| d.to_string()).unwrap_or_default(),
        delete: o.delete.unwrap_or(false),
        alert: o.alert.clone().unwrap_or_default(),
        remark: o.remark.clone().unwrap_or_default(),
        hash: o.hash.clone(),
        payable: o.payable as f64,
        pay: o.pay as f64,
        refund: o.refund,
        coupon: o.coupon,
        order: o.order.clone().unwrap_or_default(),
        pay_type: o.pay_type,
        pay_time: o.pay_time.map(|d| d.to_string()).unwrap_or_default(),
        pay_status: o.pay_status,
        gps_type: o.gps_type.unwrap_or(0),
    }
}

fn storage_to_proto(s: &ModelStorageInfo) -> StorageInfo {
    StorageInfo {
        code: s.code.clone(),
        status: s.status,
        provide: s.provide.clone(),
        gps_json: s.gps.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        r#type: s.r#type,
        sum: s.sum,
        cur: s.cur,
        create_date: s.create_date.map(|d| d.to_string()).unwrap_or_default(),
        update_date: s.update_date.map(|d| d.to_string()).unwrap_or_default(),
        delete: s.delete.unwrap_or(false),
        alert: s.alert.clone().unwrap_or_default(),
        remark: s.remark.clone().unwrap_or_default(),
        points: s.points.clone().unwrap_or_default(),
        gps_type: s.gps_type,
    }
}

#[tonic::async_trait]
impl EbikeServiceTrait for EbikeGrpcService {
    // ========== 用户认证 ==========

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let username = req.username;
        let password = req.password;

        let user = self.state.user_repo.find_by_username(&username).await
            .map_err(|e| Status::internal(format!("查询用户失败: {e}" )))?
            .ok_or_else(|| Status::unauthenticated("username or password error" ))?;

        let pwd_ok = auth_core::PasswordService.verify_bcrypt(&password, &user.password_hash);
        if !pwd_ok {
            return Err(Status::unauthenticated("username or password error" ));
        }

        let token = self.state.jwt_service.generate_access_token(
            user.id, &user.username, &user.role
        ).map_err(|e| Status::internal(format!("token generation failed: {e}" )))?;

        Ok(Response::new(LoginResponse {
            token,
            user: Some(UserBrief {
                id: user.id,
                username: user.username.clone(),
                nickname: user.nickname.clone().unwrap_or_default(),
                avatar: user.avatar.clone().unwrap_or_default(),
                role: user.role.clone(),
            }),
        }))
    }

    async fn get_user_info(
        &self,
        request: Request<grpc_proto::ebike::GetUserInfoRequest>,
    ) -> Result<Response<grpc_proto::ebike::UserInfo>, Status> {
        let req = request.into_inner();
        let user = self.state.user_repo.user_info(req.user_id).await
            .map_err(|e| Status::internal(format!("{e}" )))?
            .ok_or_else(|| Status::not_found("user not found" ))?;

        Ok(Response::new(grpc_proto::ebike::UserInfo {
            id: user.id,
            username: user.username,
            nickname: user.nickname.unwrap_or_default(),
            avatar: user.avatar.unwrap_or_default(),
            phone: user.phone.unwrap_or_default(),
            email: user.email.unwrap_or_default(),
            gender: user.gender,
            address: user.address.unwrap_or_default(),
            role: user.role,
            status: user.status,
        }))
    }

    async fn logout(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GenericResponse>, Status> {
        Ok(Response::new(GenericResponse {
            code: 0,
            message: "ok".to_string(),
            data: " ".to_string(),
        }))
    }

    // ========== 车辆管理 ==========

    async fn car_add(
        &self,
        request: Request<CarAddRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let car = ModelCarInfo {
            code: req.code.clone(),
            status: req.status,
            provide: req.provide.clone(),
            speed: req.speed,
            gps: req.gps.map(|g| {
                serde_json::json!({"lng": g.lng, "lat": g.lat})
            }),
            r#type: 0,
            time: req.time.map(|t| {
                serde_json::json!({"start": t.start, "end": t.end})
            }),
            create_date: None,
            update_date: None,
            delete: None,
            alert: if req.alert.is_empty() { None } else { Some(req.alert) },
            remark: if req.remark.is_empty() { None } else { Some(req.remark) },
            gps_type: req.gps_type,
        };
        let success = self.state.car_repo.add(&car).await
            .map_err(|e| Status::internal(format!("{e}" )))?;
        Ok(Response::new(GenericResponse {
            code: if success { 0 } else { -1 },
            message: if success { "created".to_string() } else { "failed".to_string() },
            data: req.code,
        }))
    }

    async fn car_batch_add(
        &self,
        request: Request<grpc_proto::ebike::CarBatchAddRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let mut count = 0;
        for item in req.items {
            let car = ModelCarInfo {
                code: item.code.clone(),
                status: item.status,
                provide: item.provide.clone(),
                speed: item.speed,
                gps: item.gps.map(|g| serde_json::json!({"lng": g.lng, "lat": g.lat})),
                r#type: 0,
                time: item.time.map(|t| serde_json::json!({"start": t.start, "end": t.end})),
                create_date: None,
                update_date: None,
                delete: None,
                alert: if item.alert.is_empty() { None } else { Some(item.alert) },
                remark: if item.remark.is_empty() { None } else { Some(item.remark) },
                gps_type: item.gps_type,
            };
            if self.state.car_repo.add(&car).await.is_ok() {
                count += 1;
            }
        }
        Ok(Response::new(GenericResponse {
            code: if count > 0 { 0 } else { -1 },
            message: format!("batch added {count} cars" ),
            data: format!("{count}" ),
        }))
    }

    async fn car_delete(
        &self,
        request: Request<grpc_proto::ebike::CarDeleteRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let success = self.state.car_repo.del(&req.code, &req.provide).await
            .map_err(|e| Status::internal(format!("{e}" )))?;
        Ok(Response::new(GenericResponse {
            code: if success { 0 } else { -1 },
            message: if success { "deleted".to_string() } else { "not found".to_string() },
            data: req.code,
        }))
    }

    async fn car_query(
        &self,
        request: Request<CarQueryRequest>,
    ) -> Result<Response<CarListResponse>, Status> {
        let req = request.into_inner();
        let cars = self.state.car_repo.query(
            &req.code,
            &req.provide,
            req.status,
            &req.time_start,
            &req.time_end,
            0,
            0,
        ).await.map_err(|e| Status::internal(format!("{e}" )))?;
        let proto: Vec<CarInfo> = cars.iter().map(car_to_proto).collect();
        Ok(Response::new(CarListResponse { cars: proto }))
    }

    async fn car_history(
        &self,
        request: Request<grpc_proto::ebike::CarHistoryRequest>,
    ) -> Result<Response<CarListResponse>, Status> {
        let req = request.into_inner();
        let cars = self.state.car_repo.history(&req.code)
            .await.map_err(|e| Status::internal(format!("{e}" )))?;
        let proto: Vec<CarInfo> = cars.iter().map(car_to_proto).collect();
        Ok(Response::new(CarListResponse { cars: proto }))
    }

    async fn car_alert(
        &self,
        request: Request<grpc_proto::ebike::CarAlertRequest>,
    ) -> Result<Response<CarListResponse>, Status> {
        let req = request.into_inner();
        let cars = self.state.car_repo.alert(
            &req.code,
            &req.provide,
            req.status,
            &req.time,
            &req.alert,
            &req.remark,
        ).await.map_err(|e| Status::internal(format!("{e}" )))?;
        let proto: Vec<CarInfo> = cars.iter().map(car_to_proto).collect();
        Ok(Response::new(CarListResponse { cars: proto }))
    }

    // ========== 订单管理 ==========

    async fn order_query(
        &self,
        request: Request<grpc_proto::ebike::OrderQueryRequest>,
    ) -> Result<Response<OrderListResponse>, Status> {
        let req = request.into_inner();
        let orders = self.state.order_repo.query(
            OrderQueryParams {
                code: &req.code,
                provide: &req.provide,
                status: req.status,
                order: &req.order,
                paystatus: req.paystatus,
                paytype: req.paytype,
                paytime: &req.paytime,
                limit: 0,
                offset: 0,
            }
        ).await.map_err(|e| Status::internal(format!("{e}" )))?;
        let proto: Vec<OrderInfo> = orders.iter().map(order_to_proto).collect();
        Ok(Response::new(OrderListResponse { orders: proto }))
    }

    async fn order_batch_add(
        &self,
        request: Request<grpc_proto::ebike::OrderBatchAddRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let mut count = 0;
        for item in req.items {
            let order = ModelOrderInfo {
                code: Some(item.code.clone()),
                status: item.status,
                provide: item.provide.clone(),
                speed: item.speed,
                gps: item.gps.map(|g| serde_json::json!({"lng": g.lng, "lat": g.lat})),
                r#type: 0,
                time: item.time.map(|t| serde_json::json!({"start": t.start, "end": t.end})),
                create_date: None,
                update_date: None,
                delete: None,
                alert: if item.alert.is_empty() { None } else { Some(item.alert) },
                remark: if item.remark.is_empty() { None } else { Some(item.remark) },
                hash: " ".to_string(),
                payable: item.payable as i64,
                pay: item.pay as i64,
                refund: item.refund,
                coupon: item.coupon,
                order: Some(item.order.clone()),
                pay_type: item.pay_type,
                pay_time: if item.pay_time.is_empty() { None } else { Some(chrono::NaiveDateTime::default()) },
                pay_status: item.pay_status,
                gps_type: Some(item.gps_type),
                paytype: 0,
                paytime: None,
            };
            if self.state.order_repo.add(&order).await.is_ok() {
                count += 1;
            }
        }
        Ok(Response::new(GenericResponse {
            code: if count > 0 { 0 } else { -1 },
            message: format!("batch added {count} orders" ),
            data: format!("{count}" ),
        }))
    }

    async fn order_add(
        &self,
        request: Request<OrderAddRequest>,
    ) -> Result<Response<grpc_proto::ebike::OrderAddResponse>, Status> {
        let req = request.into_inner();
        let order = ModelOrderInfo {
            code: Some(req.code.clone()),
            status: req.status,
            provide: req.provide.clone(),
            speed: req.speed,
            gps: req.gps.map(|g| serde_json::json!({"lng": g.lng, "lat": g.lat})),
            r#type: 0,
            time: req.time.map(|t| serde_json::json!({"start": t.start, "end": t.end})),
            create_date: None,
            update_date: None,
            delete: None,
            alert: if req.alert.is_empty() { None } else { Some(req.alert) },
            remark: if req.remark.is_empty() { None } else { Some(req.remark) },
            hash: " ".to_string(),
            payable: req.payable as i64,
            pay: req.pay as i64,
            refund: req.refund,
            coupon: req.coupon,
            order: Some(req.order.clone()),
            pay_type: req.pay_type,
            pay_time: if req.pay_time.is_empty() { None } else { Some(chrono::NaiveDateTime::default()) },
            pay_status: req.pay_status,
            gps_type: Some(req.gps_type),
            paytype: req.pay_type,
            paytime: None,
        };
        let hash = self.state.order_repo.add(&order).await
            .map_err(|e| Status::internal(format!("{e}" )))?;
        Ok(Response::new(grpc_proto::ebike::OrderAddResponse { hash }))
    }

    // ========== 仓储管理 ==========

    async fn storage_add(
        &self,
        request: Request<StorageAddRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let storage = ModelStorageInfo {
            code: req.code.clone(),
            status: req.status,
            provide: req.provide.clone(),
            gps: req.gps.map(|g| serde_json::json!({"lng": g.lng, "lat": g.lat})),
            r#type: 0,
            sum: req.sum,
            cur: req.cur,
            create_date: None,
            update_date: None,
            delete: None,
            alert: if req.alert.is_empty() { None } else { Some(req.alert) },
            remark: if req.remark.is_empty() { None } else { Some(req.remark) },
            points: if req.points.is_empty() { None } else { Some(req.points) },
            gps_type: req.gps_type,
        };
        let success = self.state.storage_repo.add(&storage).await
            .map_err(|e| Status::internal(format!("{e}" )))?;
        Ok(Response::new(GenericResponse {
            code: if success { 0 } else { -1 },
            message: if success { "created".to_string() } else { "failed".to_string() },
            data: req.code,
        }))
    }

    async fn storage_batch_add(
        &self,
        request: Request<grpc_proto::ebike::StorageBatchAddRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let mut count = 0;
        for item in req.items {
            let storage = ModelStorageInfo {
                code: item.code.clone(),
                status: item.status,
                provide: item.provide.clone(),
                gps: item.gps.map(|g| serde_json::json!({"lng": g.lng, "lat": g.lat})),
                r#type: 0,
                sum: item.sum,
                cur: item.cur,
                create_date: None,
                update_date: None,
                delete: None,
                alert: if item.alert.is_empty() { None } else { Some(item.alert) },
                remark: if item.remark.is_empty() { None } else { Some(item.remark) },
                points: if item.points.is_empty() { None } else { Some(item.points) },
                gps_type: item.gps_type,
            };
            if self.state.storage_repo.add(&storage).await.is_ok() {
                count += 1;
            }
        }
        Ok(Response::new(GenericResponse {
            code: if count > 0 { 0 } else { -1 },
            message: format!("batch added {count} storages" ),
            data: format!("{count}" ),
        }))
    }

    async fn storage_delete(
        &self,
        request: Request<grpc_proto::ebike::StorageDeleteRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let success = self.state.storage_repo.del(&req.code).await
            .map_err(|e| Status::internal(format!("{e}" )))?;
        Ok(Response::new(GenericResponse {
            code: if success { 0 } else { -1 },
            message: if success { "deleted".to_string() } else { "not found".to_string() },
            data: req.code,
        }))
    }

    async fn storage_query(
        &self,
        request: Request<StorageQueryRequest>,
    ) -> Result<Response<StorageListResponse>, Status> {
        let req = request.into_inner();
        let storages = self.state.storage_repo.query(
            &req.code,
            &req.provide,
            req.status,
        ).await.map_err(|e| Status::internal(format!("{e}" )))?;
        let proto: Vec<StorageInfo> = storages.iter().map(storage_to_proto).collect();
        Ok(Response::new(StorageListResponse { storages: proto }))
    }

    async fn storage_history(
        &self,
        request: Request<grpc_proto::ebike::StorageHistoryRequest>,
    ) -> Result<Response<StorageListResponse>, Status> {
        let req = request.into_inner();
        let storages = self.state.storage_repo.history(&req.code)
            .await.map_err(|e| Status::internal(format!("{e}" )))?;
        let proto: Vec<StorageInfo> = storages.iter().map(storage_to_proto).collect();
        Ok(Response::new(StorageListResponse { storages: proto }))
    }

    // ========== 系统配置 ==========

    async fn options_query(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<OptionsInfo>, Status> {
        let options = self.state.options_repo.query()
            .await.map_err(|e| Status::internal(format!("{e}" )))?;
        if options.is_empty() {
            return Ok(Response::new(OptionsInfo {
                name: "".to_string(),
                options_json: " ".to_string(),
                level: 0,
                create_date: "".to_string(),
                update_date: " ".to_string(),
                delete: false,
            }));
        }
        let first = &options[0];
        Ok(Response::new(OptionsInfo {
            name: first.name.clone(),
            options_json: first.options.to_string(),
            level: first.level,
            create_date: first.create_date.map(|d| d.to_string()).unwrap_or_default(),
            update_date: first.update_date.map(|d| d.to_string()).unwrap_or_default(),
            delete: first.delete.unwrap_or(false),
        }))
    }

    async fn options_update(
        &self,
        request: Request<OptionsUpdateRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let req = request.into_inner();
        let mut options = self.state.options_repo.query()
            .await.map_err(|e| Status::internal(format!("{e}" )))?;
        if options.is_empty() {
            return Ok(Response::new(GenericResponse {
                code: -1,
                message: "not found".to_string(),
                data: " ".to_string(),
            }));
        }
        let options = &mut options[0];
        options.level = req.level;
        let mut opts: serde_json::Value = serde_json::from_str(&options.options.to_string()).unwrap_or_default();
        opts["system" ] = serde_json::json!(req.system);
        opts["alert" ] = serde_json::json!(req.alert);
        options.options = opts;
        let success = self.state.options_repo.update(options).await
            .map_err(|e| Status::internal(format!("{e}" )))?;
        Ok(Response::new(GenericResponse {
            code: if success { 0 } else { -1 },
            message: if success { "updated".to_string() } else { "not found".to_string() },
            data: req.name,
        }))
    }
}
