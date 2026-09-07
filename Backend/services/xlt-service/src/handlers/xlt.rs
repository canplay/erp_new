//! XLT HTTP 路由处理器

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::Result;
use crate::models::{
    BillingRequest, MqttEnvelope, VehicleEvent, VehicleQuery,
};
use crate::services::{
    billing::BillingService,
    device_manager::DeviceManager,
    mqtt_gateway::MqttGateway,
    parking::ParkingService,
};

/// 应用共享状态
#[derive(Clone)]
pub struct AppState {
    pub parking_service: Arc<ParkingService>,
    pub billing_service: Arc<BillingService>,
    pub mqtt_gateway: Arc<MqttGateway>,
    pub device_manager: Arc<DeviceManager>,
}

/// 车辆入场
pub async fn vehicle_entry(
    State(state): State<AppState>,
    Json(event): Json<VehicleEvent>,
) -> Result<Json<serde_json::Value>> {
    let record = state.parking_service.handle_entry(&event).await?;
    Ok(Json(serde_json::json!(record)))
}

/// 车辆出场
pub async fn vehicle_exit(
    State(state): State<AppState>,
    Json(event): Json<VehicleEvent>,
) -> Result<Json<serde_json::Value>> {
    let record = state.parking_service.handle_exit(&event).await?;
    Ok(Json(serde_json::json!(record)))
}

/// 查询在场车辆
pub async fn get_parking_vehicle(
    State(state): State<AppState>,
    Path((park_code, plate_no)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let vehicle = state
        .parking_service
        .get_parking_vehicle(&park_code, &plate_no)
        .await?;
    Ok(Json(serde_json::json!(vehicle)))
}

/// 计算停车费用
pub async fn calc_billing(
    State(state): State<AppState>,
    Json(req): Json<BillingRequest>,
) -> Result<Json<serde_json::Value>> {
    let result = state.billing_service.calculate(&req).await?;
    Ok(Json(serde_json::json!(result)))
}

/// 车辆进出记录列表
pub async fn list_records(
    State(_state): State<AppState>,
    Query(query): Query<VehicleQuery>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "records": [],
        "total": 0,
        "page": query.page.unwrap_or(1),
        "page_size": query.page_size.unwrap_or(20),
    })))
}

/// 接收设备 MQTT 消息回调 (HTTP bridge)
#[derive(Debug, Deserialize)]
pub struct MqttCallbackPayload {
    pub topic: String,
    pub payload: String,
}

pub async fn mqtt_callback(
    State(state): State<AppState>,
    Json(msg): Json<MqttCallbackPayload>,
) -> Result<Json<serde_json::Value>> {
    let envelope: MqttEnvelope = serde_json::from_str(&msg.payload)
        .map_err(|e| crate::error::XltError::Internal(format!("解析MQTT消息失败: {e}")))?;

    tracing::info!(
        "MQTT回调: topic={}, command={}, sn={}",
        msg.topic,
        envelope.command,
        envelope.sn
    );

    // 处理特殊上行消息
    match envelope.command.as_str() {
        "Conn" => {
            if let Ok(conn) = serde_json::from_str::<crate::models::ConnData>(&envelope.data) {
                state.device_manager.register_device(&envelope.sn, "", &conn.dev_info, &envelope.version).await;
            }
        }
        "HeartBeat" => {
            state.device_manager.update_heartbeat(&envelope.sn).await;
        }
        "DeviceAlarm" => {
            if let Ok(alarm) = serde_json::from_str::<crate::models::DeviceAlarmData>(&envelope.data) {
                tracing::warn!(
                    "设备告警: sn={}, type={}, code={}, desc={}, time={}",
                    envelope.sn, alarm.alarm_type, alarm.alarm_code, alarm.alarm_desc, alarm.alarm_time
                );
            }
        }
        "VehicleDetection" => {
            if let Ok(det) = serde_json::from_str::<crate::models::VehicleDetectionData>(&envelope.data) {
                tracing::info!(
                    "车辆检测: sn={}, has_car={}, trigger={}, time={}",
                    envelope.sn, det.has_car, det.trigger_source, det.detect_time
                );
            }
        }
        _ => {}
    }

    state.device_manager.dispatch(envelope).await;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "success"
    })))
}

/// 获取设备列表
pub async fn list_devices(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let devices = state.device_manager.list_devices().await;
    Ok(Json(serde_json::json!(devices)))
}

/// 发送开闸命令
#[derive(Debug, Deserialize)]
pub struct BarrierCommand {
    pub sn: String,
    pub request_id: String,
}

pub async fn open_barrier(
    State(state): State<AppState>,
    Json(cmd): Json<BarrierCommand>,
) -> Result<Json<serde_json::Value>> {
    state.mqtt_gateway.send_open(&cmd.sn, &cmd.request_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// 发送关闸命令
pub async fn close_barrier(
    State(state): State<AppState>,
    Json(cmd): Json<BarrierCommand>,
) -> Result<Json<serde_json::Value>> {
    state.mqtt_gateway.send_close(&cmd.sn, &cmd.request_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// 健康检查
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "xlt-service"
    }))
}
