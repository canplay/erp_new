//! CTP HTTP 路由处理器
//!
//! 提供设备数据上报、锁控制、设备状态查询和列表检索等端点。

use axum::{Json, extract::Query, extract::State};
use std::sync::Arc;

use common::AppResult;
use crate::helpers::{json_ctp_lock_response, json_device_upload_response, json_health, json_success_value};
use crate::models::{CmdType, DeviceDataUpload, DeviceQuery};
use crate::services::CtpDeviceService;

/// 应用共享状态
#[derive(Clone)]
pub struct AppState {
    pub ctp_service: Arc<CtpDeviceService>,
}

/// 接收设备上报数据并转发至 CTP 平台，同时缓存最新状态至 Redis
pub async fn receive_device_data(
    State(state): State<AppState>,
    Json(upload): Json<DeviceDataUpload>,
) -> AppResult<Json<serde_json::Value>> {
    let ctp_resp = state.ctp_service.upload_device_data(&upload).await?;
    state.ctp_service.handle_device_data_upload(&upload).await?;

    Ok(json_device_upload_response(ctp_resp.error_code, &ctp_resp.error_msg))
}

/// 发送锁控制命令（开锁/关锁/同步）
pub async fn control_lock(
    State(state): State<AppState>,
    Json(cmd): Json<crate::models::LockControlRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let ctp_resp = state
        .ctp_service
        .send_lock_command(&cmd.device_no, &cmd.cmd_type, cmd.data.as_deref())
        .await?;

    let action_str = match cmd.cmd_type {
        CmdType::Up => "up" ,
        CmdType::Down => "down" ,
        CmdType::Syn => "syn" ,
    };

    Ok(json_ctp_lock_response(ctp_resp.error_code == 0, &ctp_resp.error_msg, &cmd.device_no, action_str))
}

/// 查询单个设备状态（从 Redis 缓存获取）
pub async fn get_device(
    State(state): State<AppState>,
    axum::extract::Path(device_no): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let device = state.ctp_service.get_device_status(&device_no).await?;
    Ok(json_success_value(serde_json::to_value(device).unwrap_or_default()))
}

/// 分页获取设备列表（支持按车场代码过滤）
pub async fn list_devices(
    State(state): State<AppState>,
    Query(query): Query<DeviceQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let result = state
        .ctp_service
        .list_devices(query.park_code.as_deref(), page, page_size)
        .await?;
    Ok(Json(result))
}

/// 健康检查端点
pub async fn health() -> Json<serde_json::Value> {
    json_health("ctp-service" )
}
