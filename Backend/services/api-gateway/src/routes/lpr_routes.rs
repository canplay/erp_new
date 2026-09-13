//! LPR 服务路由 — 车牌识别回调 HTTP → gRPC 转发
//!
//! 接收 Vz 车牌识别相机的 HTTP POST 推送，
//! 通过 gRPC 转发至 lpr-service 处理。

use std::sync::Arc;
use axum::{Router, extract::State, routing::post, Json};
use serde::Deserialize;
use common::AppError;
#[allow(unused_imports)]

use crate::AppState;
use crate::routes::helpers::json_error;

/// 车牌识别回调请求体
#[derive(Debug, Deserialize)]
pub struct LprCallbackPayload {
    pub plate_no: Option<String>,
    pub plate_color: Option<String>,
    pub plate_type: Option<String>,
    pub vehicle_type: Option<String>,
    pub pass_time: Option<String>,
    pub image_url: Option<String>,
    pub device_id: Option<String>,
    pub park_code: Option<String>,
    pub direction: Option<String>,
    pub confidence: Option<f64>,
    pub device_name: Option<String>,
    pub lane_code: Option<String>,
}

/// 车牌识别回调处理器
async fn lpr_callback_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LprCallbackPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let clients = state.grpc_clients.read().await;
    let mut client = clients.lpr_client().await.map_err(|e| {
        tracing::error!("lpr-service 不可用: {e}");
        AppError::ServiceUnavailable("lpr-service 不可用".into())
    })?;

    let result = client
        .lpr_callback(
            payload.plate_no.unwrap_or_default(),
            payload.plate_color.unwrap_or_default(),
            payload.plate_type.unwrap_or_default(),
            payload.vehicle_type.unwrap_or_default(),
            payload.pass_time.unwrap_or_default(),
            payload.image_url.unwrap_or_default(),
            payload.device_id.unwrap_or_default(),
            payload.park_code.unwrap_or_default(),
            payload.direction.unwrap_or_default(),
            payload.confidence.unwrap_or(0.0),
            payload.device_name.unwrap_or_default(),
            payload.lane_code.unwrap_or_default(),
        )
        .await;

    match result {
        Ok(response) => Ok(Json(serde_json::json!({
            "success": response.success,
            "message": response.message,
            "code": response.code
        }))),
        Err(e) => {
            tracing::error!("gRPC LPR 回调转发失败: {e}");
            Err(AppError::ServiceUnavailable(format!("lpr-service 调用失败: {e}")))
        }
    }
}

// ==================== LPR 车辆授权 Handler ====================

#[derive(Debug, Deserialize)]
pub struct VehicleAuthPayload {
    pub plate_no: Option<String>,
    pub park_code: Option<String>,
    pub device_id: Option<String>,
}

async fn vehicle_auth_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleAuthPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let clients = state.grpc_clients.read().await;
    let mut client = clients.lpr_client().await.map_err(|e| {
        tracing::error!("lpr-service 不可用: {e}");
        AppError::ServiceUnavailable("lpr-service 不可用".into())
    })?;

    let result = client
        .get_vehicle_auth(
            &payload.plate_no.unwrap_or_default(),
            &payload.park_code.unwrap_or_default(),
        )
        .await;

    match result {
        Ok(response) => Ok(Json(serde_json::json!({
            "success": true,
            "code": 200,
            "data": {
                "is_authorized": response.is_authorized,
                "auth_type": response.auth_type,
                "driver_name": response.driver_name,
                "driver_phone": response.driver_phone,
                "valid_until": response.valid_until,
            }
        }))),
        Err(e) => {
            tracing::error!("gRPC LPR 车辆授权转发失败: {e}");
            Ok(json_error(&format!("lpr-service 调用失败: {e}")))
        }
    }
}

/// 路由定义
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/lpr/callback", post(lpr_callback_handler))
        .route("/api/lpr/callback", post(lpr_callback_handler))
        .route("/api/lpr/vehicle/auth", post(vehicle_auth_handler))
}
