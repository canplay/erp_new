//! Hik Service - 海康设备集成服务
//!
//! 提供海康设备的管理和控制接口

pub mod error;
pub mod grpc_server;
pub mod handlers;
pub mod models;
pub mod services;

use axum::{Router, routing::post};
use std::sync::Arc;

use crate::handlers::hik_service::{AppState, coupon, exec, signo_open};
use crate::services::{HikService, SignoService};

/// 创建 Hik Service 应用
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/hik/exec", post(exec))
        .route("/api/hik/coupon", post(coupon))
        .route("/api/hik/signo/open", post(signo_open))
        .with_state(state)
}

/// 创建服务状态
#[must_use]
pub fn create_state() -> AppState {
    AppState {
        hik_service: Arc::new(HikService::new()),
        signo_service: Arc::new(SignoService::new()),
    }
}
