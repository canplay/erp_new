// 支付服务 HTTP 处理器
// HTTP handlers for pay service

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::error::PayError;
use crate::models::{PayCreateParams, PayOrder, PayQuery};
use crate::services::{
    ccb::{
        CcbConfig, CcbOrderParams, CcbOrderResponse, CcbQueryParams, CcbQueryResponse, CcbService,
    },
    ums::{UmsCloseParams, UmsConfig, UmsOrderParams, UmsQueryParams, UmsRefundParams, UmsService},
};

/// 统一响应结构
#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub data: T,
    pub message: String,
    #[serde(rename = "code")]
    pub status_code: i32,
}

impl<T> Response<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            data,
            message: message.to_string(),
            status_code: 200,
        }
    }

    #[must_use]
    pub fn error(message: &str) -> Response<()> {
        Response {
            data: (),
            message: message.to_string(),
            status_code: 500,
        }
    }
}

// ============ 基础支付接口 ============

/// 查询支付订单数量
/// GET /api/pay/count
pub async fn count(
    State(state): State<AppState>,
    Query(query): Query<PayQuery>,
) -> Result<Json<Response<i64>>, PayError> {
    let count = state.pay_service.count(&query).await?;
    Ok(Json(Response::success(count, "查询成功")))
}

/// 查询支付订单列表
/// GET /api/pay/list
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<PayQuery>,
) -> Result<Json<Response<Vec<PayOrder>>>, PayError> {
    let orders = state.pay_service.list(&query).await?;
    Ok(Json(Response::success(orders, "查询成功")))
}

/// 查询最近已支付订单信息
/// GET /`api/pay/latest/{user_id`}
pub async fn latest(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Response<Option<PayOrder>>>, PayError> {
    let order = state.pay_service.latest(&user_id).await?;
    Ok(Json(Response::success(order, "查询成功")))
}

/// 创建支付订单
/// POST /api/pay/create
pub async fn create_order(
    State(state): State<AppState>,
    Json(params): Json<PayCreateParams>,
) -> Result<Json<Response<PayOrder>>, PayError> {
    let order = state.pay_service.create_order(&params).await?;
    Ok(Json(Response::success(order, "创建成功")))
}

// ============ CCB 建行支付接口 ============

/// 创建 CCB 配置
fn create_ccb_config() -> CcbConfig {
    CcbConfig {
        merchantid: std::env::var("CCB_MERCHANT_ID").unwrap_or_default(),
        branchid: std::env::var("CCB_BRANCH_ID").unwrap_or_default(),
        posid: std::env::var("CCB_POS_ID").unwrap_or_default(),
        qupwd: std::env::var("CCB_QUPWD").unwrap_or_default(),
        pub_key: std::env::var("CCB_PUB_KEY").unwrap_or_default(),
    }
}

/// CCB 查询支付订单
/// GET /api/pay/ccb/query
pub async fn ccb_query(
    Query(params): Query<CcbQueryParams>,
) -> Result<Json<CcbQueryResponse>, PayError> {
    let service = CcbService::new(create_ccb_config());
    let result = service.query(&params).await?;
    Ok(Json(result))
}

/// CCB 创建支付订单
/// POST /api/pay/ccb/create
pub async fn ccb_create(
    Json(params): Json<CcbOrderParams>,
) -> Result<Json<CcbOrderResponse>, PayError> {
    let service = CcbService::new(create_ccb_config());
    let result = service.create_order(&params).await?;
    Ok(Json(result))
}

/// CCB 验证支付结果
/// GET /`api/pay/ccb/verify/{order_id`}
pub async fn ccb_verify(Path(order_id): Path<String>) -> Result<Json<Response<bool>>, PayError> {
    let service = CcbService::new(create_ccb_config());
    let result = service.verify_payment(&order_id).await?;
    Ok(Json(Response::success(result, "验证成功")))
}

/// CCB 退款参数
#[derive(Debug, Deserialize)]
pub struct CcbRefundParams {
    pub order_id: String,
    pub amount: i64,
}

/// CCB 退款
/// POST /api/pay/ccb/refund
pub async fn ccb_refund(
    Json(params): Json<CcbRefundParams>,
) -> Result<Json<Response<bool>>, PayError> {
    let service = CcbService::new(create_ccb_config());
    let result = service.refund(&params.order_id, params.amount).await?;
    Ok(Json(Response::success(result, "退款成功")))
}

// ============ UMS 银联支付接口 ============

/// 创建 UMS 配置
fn create_ums_config() -> UmsConfig {
    UmsConfig {
        appid: std::env::var("UMS_APP_ID").unwrap_or_default(),
        appkey: std::env::var("UMS_APP_KEY").unwrap_or_default(),
        mid: std::env::var("UMS_MID").unwrap_or_default(),
        tid: std::env::var("UMS_TID").unwrap_or_default(),
        ysjc: std::env::var("UMS_YSJC").ok(),
    }
}

/// 获取 UMS 访问令牌
async fn get_ums_token(service: &UmsService) -> Result<String, PayError> {
    service
        .get_access_token()
        .await
        .map_err(|e| PayError::InternalError(e.to_string()))
}

/// UMS 查询支付订单
/// GET /api/pay/ums/query
pub async fn ums_query(
    Query(params): Query<UmsQueryParams>,
) -> Result<Json<Response<serde_json::Value>>, PayError> {
    let pool = sqlx::PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
        .expect("failed to connect to database");
    let service = UmsService::new(create_ums_config(), pool);
    let token = get_ums_token(&service).await?;
    let result = service.query(&params, &token).await?;
    Ok(Json(Response::success(result, "查询成功")))
}

/// UMS 创建支付订单
/// POST /api/pay/ums/create
pub async fn ums_create(
    Json(params): Json<UmsOrderParams>,
) -> Result<Json<Response<serde_json::Value>>, PayError> {
    let pool = sqlx::PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
        .expect("failed to connect to database");
    let service = UmsService::new(create_ums_config(), pool);
    let token = get_ums_token(&service).await?;
    let result = service.create_order(&params, &token).await?;
    Ok(Json(Response::success(result, "创建成功")))
}

/// UMS 关闭支付订单
/// POST /api/pay/ums/close
pub async fn ums_close(
    Json(params): Json<UmsCloseParams>,
) -> Result<Json<Response<serde_json::Value>>, PayError> {
    let pool = sqlx::PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
        .expect("failed to connect to database");
    let service = UmsService::new(create_ums_config(), pool);
    let token = get_ums_token(&service).await?;
    let result = service.close(&params, &token).await?;
    Ok(Json(Response::success(result, "关闭成功")))
}

/// UMS 退款
/// POST /api/pay/ums/refund
pub async fn ums_refund(
    Json(params): Json<UmsRefundParams>,
) -> Result<Json<Response<serde_json::Value>>, PayError> {
    let pool = sqlx::PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap_or_default())
        .expect("failed to connect to database");
    let service = UmsService::new(create_ums_config(), pool);
    let token = get_ums_token(&service).await?;
    let result = service.refund(&params, &token).await?;
    Ok(Json(Response::success(result, "退款成功")))
}

/// UMS 获取订单信息
/// GET /api/pay/ums/info/{order}
pub async fn ums_info(
    State(state): State<AppState>,
    Path(order): Path<String>,
) -> Result<Json<Response<serde_json::Value>>, PayError> {
    let service = UmsService::new(create_ums_config(), state.pool.clone());
    let token = get_ums_token(&service).await?;
    let info = service.query_ums_info(&order, &token).await?;
    Ok(Json(Response::success(info, "查询成功")))
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn test_response_success() {
        let resp = Response::success(42, "成功");
        assert_eq!(resp.data, 42);
        assert_eq!(resp.message, "成功");
        assert_eq!(resp.status_code, 200);
    }

    #[test]
    fn test_response_error() {
        let resp = Response::<()>::error("失败");
        assert_eq!(resp.data, ());
        assert_eq!(resp.message, "失败");
        assert_eq!(resp.status_code, 500);
    }

    #[test]
    fn test_response_success_string() {
        let resp = Response::success("hello".to_string(), "ok");
        assert_eq!(resp.data, "hello");
        assert_eq!(resp.message, "ok");
    }

    #[test]
    fn test_response_serialization() {
        let resp = Response::success(100, "测试");
        let json = serde_json::to_string(&resp).expect("test assertion");
        assert!(json.contains("\"code\":200"));
        assert!(json.contains("\"message\":\"测试\""));
        assert!(json.contains("\"data\":100"));
    }

    #[test]
    fn test_response_error_serialization() {
        let resp = Response::<()>::error("错误");
        let json = serde_json::to_string(&resp).expect("test assertion");
        assert!(json.contains("\"code\":500"));
        assert!(json.contains("\"message\":\"错误\""));
    }
}
