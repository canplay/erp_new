//! Pay Service - 支付服务
//!
//! 提供支付相关的功能，包括：
//! - 基础支付订单管理（查询、创建）
//! - CCB 建设银行支付集成
//! - UMS 银联支付集成

pub mod error;
pub mod grpc_server;
pub mod handlers;
pub mod models;
pub mod services;

// 重新导出常用类型
pub use handlers::AppState;
pub use services::pay::PayService;
pub use services::subscription::{SubscriptionService, SubscriptionResult, CreateSubscriptionParams, PlanDefinition};

use axum::{Router, routing::get, routing::post};

use crate::handlers::{
    ccb_create, ccb_query, ccb_refund, ccb_verify, count, create_order, latest, list, ums_close,
    ums_create, ums_info, ums_query, ums_refund,
};

/// 创建 Pay Service 应用（仅内部兼容，不再启动 HTTP）
pub fn create_app(state: AppState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(|| async { "OK" }))
        // ============ 基础支付接口 ============
        .route("/api/pay/count", get(count))
        .route("/api/pay/list", get(list))
        .route("/api/pay/latest/:user_id", get(latest))
        .route("/api/pay/create", post(create_order))
        // ============ CCB 建设银行支付接口 ============
        .route("/api/pay/ccb/query", get(ccb_query))
        .route("/api/pay/ccb/create", post(ccb_create))
        .route("/api/pay/ccb/verify/:order_id", get(ccb_verify))
        .route("/api/pay/ccb/refund", post(ccb_refund))
        // ============ UMS 银联支付接口 ============
        .route("/api/pay/ums/query", get(ums_query))
        .route("/api/pay/ums/create", post(ums_create))
        .route("/api/pay/ums/close", post(ums_close))
        .route("/api/pay/ums/refund", post(ums_refund))
        .route("/api/pay/ums/info/:order", get(ums_info))
        .with_state(state)
}
