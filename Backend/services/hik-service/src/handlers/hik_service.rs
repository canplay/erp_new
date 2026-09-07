// 海康服务 HTTP 处理器
// HTTP handlers for hik service

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::Result;
use crate::models::{CouponRequest, HikRequest};
use crate::services::{HikService, SignoService};

/// HTTP 应用状态
#[derive(Clone)]
pub struct AppState {
    pub hik_service: Arc<HikService>,
    pub signo_service: Arc<SignoService>,
}

/// 海康API执行
pub async fn exec(
    State(state): State<AppState>,
    Json(payload): Json<HikRequest>,
) -> Result<Json<serde_json::Value>> {
    let result = state.hik_service.exec(&payload).await?;
    Ok(Json(result))
}

/// 发送优惠券
pub async fn coupon(
    State(state): State<AppState>,
    Json(payload): Json<CouponRequest>,
) -> Result<Json<serde_json::Value>> {
    if payload.r#type == "car" {
        let result = state
            .hik_service
            .send_coupon(&payload.phone, payload.amount, payload.start, &payload.end)
            .await?;
        Ok(Json(result))
    } else {
        Ok(Json(serde_json::json!({
            "message": "success",
            "status": 0
        })))
    }
}

/// Signo开闸
#[derive(Debug, Deserialize)]
pub struct SignoRequest {
    pub place: String,
    pub name: String,
}

pub async fn signo_open(
    State(state): State<AppState>,
    Json(payload): Json<SignoRequest>,
) -> Result<Json<SignoResponse>> {
    let result = state
        .signo_service
        .open_gate(&payload.place, &payload.name)
        .await?;
    Ok(Json(SignoResponse {
        message: result,
        status: 1,
    }))
}

/// Signo响应
#[derive(Serialize)]
pub struct SignoResponse {
    pub message: String,
    pub status: i32,
}
