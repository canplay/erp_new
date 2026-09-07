//! HTTP Handlers 模块
//!
//! 提供 HTTP REST API 接口（简化版）
//! 只使用 repository 模式，避免循环依赖

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::repository::{
    CarClassRepository, CarColorRepository, CarQuery, CarRepository, CarTypeRepository,
    DcCausesRepository, DcTypeRepository,
};

/// HTTP 应用状态（简化版）
#[derive(Clone)]
pub struct HttpAppState {
    pub pool: sqlx::PgPool,
}

// ============ 请求/响应结构 ============

/// 车辆查询参数（导出供外部使用）
#[derive(Debug, Deserialize)]
pub struct CarQueryParams {
    pub status: Option<String>,
    pub in_date: Option<String>,
    pub out_date: Option<String>,
    pub content: Option<String>,
    pub model: Option<String>,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub key: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub sort_by: Option<String>,
    pub descending: Option<bool>,
}

/// 车辆扣押请求
#[derive(Debug, Deserialize)]
pub struct DetainRequest {
    pub plate: String,
}

/// 车辆响应
#[derive(Debug, Serialize)]
pub struct CarResponse {
    pub id: i64,
    pub license: String,
    pub vehicle: Option<serde_json::Value>,
    pub engine: String,
    pub car_type: String,
    pub dc_type: String,
    pub dc_causes: String,
    pub car_color: String,
    pub dc_date: String,
    pub dc_address: String,
    pub dc_key: String,
    pub dc_party_name: String,
    pub dc_party_cardid: String,
    pub dc_party_tel: String,
    pub dc_name: String,
    pub cmd_unit: String,
    pub cv: String,
    pub tv: String,
    pub parking_unit: String,
    pub remark: String,
    pub rs_name: String,
    pub rs_date: String,
    pub delete: bool,
}

impl From<crate::repository::CarListItem> for CarResponse {
    fn from(car: crate::repository::CarListItem) -> Self {
        Self {
            id: car.id,
            license: car.license,
            vehicle: car.vehicle,
            engine: car.engine,
            car_type: car.car_type,
            dc_type: car.dc_type,
            dc_causes: car.dc_causes,
            car_color: car.car_color,
            dc_date: car.dc_date,
            dc_address: car.dc_address,
            dc_key: car.dc_key,
            dc_party_name: car.dc_party_name,
            dc_party_cardid: car.dc_party_cardid,
            dc_party_tel: car.dc_party_tel,
            dc_name: car.dc_name,
            cmd_unit: car.cmd_unit,
            cv: car.cv,
            tv: car.tv,
            parking_unit: car.parking_unit,
            remark: car.remark,
            rs_name: car.rs_name,
            rs_date: car.rs_date,
            delete: car.delete,
        }
    }
}

// ============ 处理器实现 ============

/// 获取车辆列表
pub async fn list_cars(
    State(state): State<HttpAppState>,
    Query(params): Query<CarQueryParams>,
) -> impl IntoResponse {
    let repo = CarRepository::new(state.pool.clone());
    let query = CarQuery {
        status: params.status,
        in_date: params.in_date,
        out_date: params.out_date,
        content: params.content,
        model: params.model,
        name: params.name,
        unit: params.unit,
        key: params.key,
        page: params.page,
        page_size: params.page_size,
        sort_by: params.sort_by,
        descending: params.descending,
    };

    match repo.list(&query).await {
        Ok(result) => {
            let cars: Vec<CarResponse> = result.cars.into_iter().map(std::convert::Into::into).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": 1,
                    "message": "success",
                    "data": {
                        "list": cars,
                        "total": result.total,
                        "page": result.page,
                        "page_size": result.page_size
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("查询车辆列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 统计车辆数量
pub async fn count_cars(
    State(state): State<HttpAppState>,
    Query(params): Query<CarQueryParams>,
) -> impl IntoResponse {
    let repo = CarRepository::new(state.pool.clone());
    let query = CarQuery {
        status: params.status,
        in_date: params.in_date,
        out_date: params.out_date,
        content: params.content,
        model: params.model,
        name: params.name,
        unit: params.unit,
        key: params.key,
        ..Default::default()
    };

    match repo.count(&query).await {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": count
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("统计车辆数量失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 车辆扣押查询
pub async fn detain_car(
    State(state): State<HttpAppState>,
    Json(req): Json<DetainRequest>,
) -> impl IntoResponse {
    let repo = CarRepository::new(state.pool.clone());

    match repo.find_by_license(&req.plate).await {
        Ok(Some(car)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": {
                    "id": car.id,
                    "license": car.license,
                    "dc_causes": car.dc_causes,
                    "dc_date": car.dc_date,
                    "dc_address": car.dc_address,
                    "dc_into_date": car.dc_into_date,
                    "cmd_unit": car.cmd_unit,
                    "attachment": car.attachment,
                    "cv": car.cv,
                    "tv": car.tv
                }
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": serde_json::json!([])
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("车辆扣押查询失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 获取车辆分类列表
pub async fn list_car_class(State(state): State<HttpAppState>) -> impl IntoResponse {
    let repo = CarClassRepository::new(state.pool.clone());

    match repo.list().await {
        Ok(items) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": items
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取车辆分类列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 获取车辆类型列表
pub async fn list_car_type(State(state): State<HttpAppState>) -> impl IntoResponse {
    let repo = CarTypeRepository::new(state.pool.clone());

    match repo.list().await {
        Ok(items) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": items
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取车辆类型列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 获取车辆颜色列表
pub async fn list_car_color(State(state): State<HttpAppState>) -> impl IntoResponse {
    let repo = CarColorRepository::new(state.pool.clone());

    match repo.list().await {
        Ok(items) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": items
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取车辆颜色列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 获取拖车原因类型列表
pub async fn list_causes_type(State(state): State<HttpAppState>) -> impl IntoResponse {
    let repo = DcTypeRepository::new(state.pool.clone());

    match repo.list().await {
        Ok(items) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": items
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取拖车原因类型列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 获取拖车原因列表
pub async fn list_causes(State(state): State<HttpAppState>) -> impl IntoResponse {
    let repo = DcCausesRepository::new(state.pool.clone());

    match repo.list().await {
        Ok(items) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": 1,
                "message": "success",
                "data": items
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取拖车原因列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 0,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// 健康检查
pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": 1,
            "message": "success"
        })),
    )
        .into_response()
}

// ============ 路由构建 ============

/// 创建车辆路由
pub fn create_car_router(state: HttpAppState) -> Router {
    Router::new()
        .route("/count", axum::routing::post(count_cars))
        .route("/list", axum::routing::post(list_cars))
        .route("/detain", axum::routing::post(detain_car))
        .route("/health", axum::routing::get(health))
        .with_state(state)
}

/// 创建配置选项路由
pub fn create_options_router(state: HttpAppState) -> Router {
    Router::new()
        .route("/car_class", axum::routing::get(list_car_class))
        .route("/car_type", axum::routing::get(list_car_type))
        .route("/car_color", axum::routing::get(list_car_color))
        .route("/causes_type", axum::routing::get(list_causes_type))
        .route("/causes", axum::routing::get(list_causes))
        .route("/health", axum::routing::get(health))
        .with_state(state)
}
