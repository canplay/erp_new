//! 设备管理路由 — 内存实现
//!
//! 设备数据仅在 api-gateway 内存中管理，不经过 gRPC 后端。

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State, Extension},
    routing::{get, put, delete},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::middleware::JwtClaims;
use crate::routes::helpers::*;
use crate::repository::LoginDevice as RepoLoginDevice;

// ==================== 数据类型 ====================

// 使用 repository 中的 LoginDevice 类型，避免重复定义

// ==================== 查询参数 ====================

#[derive(Debug, Deserialize)]
pub struct DeviceQuery {
    pub keyword: Option<String>,
    pub device_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_trusted: Option<bool>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

// ==================== Handler ====================

/// GET /api/devices — 当前用户设备列表
async fn list_my_devices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Query(q): Query<DeviceQuery>,
) -> Json<Value> {
    let devices = state.device_store.read().await;
    let all: Vec<RepoLoginDevice> = devices.devices().iter()
        .filter(|d| d.user_id == claims.sub)
        .cloned()
        .collect();
    drop(devices);

    let mut result = all;

    if let Some(ref dt) = q.device_type {
        result.retain(|d| d.device_type == *dt);
    }
    if let Some(active) = q.is_active {
        result.retain(|d| d.is_active == active);
    }
    if let Some(trusted) = q.is_trusted {
        result.retain(|d| d.is_trusted == trusted);
    }

    let total = result.len();
    let page = q.page.unwrap_or(1).max(1);
    let size = q.page_size.unwrap_or(20).max(1);
    let start = (page - 1) * size;
    let list: Vec<RepoLoginDevice> = result.into_iter().skip(start).take(size).collect();

    json_success(json!({"list": list, "total": total, "page": page, "page_size": size}))
}

/// GET /api/devices/{id} — 设备详情
async fn get_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let devices = state.device_store.read().await;
    match devices.devices().iter().find(|d| d.id == id && d.user_id == claims.sub) {
        Some(d) => json_success(json!(d)),
        None => json_error("设备不存在"),
    }
}

/// PUT /api/devices/{id}/trust — 设为可信
async fn trust_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let devices = state.device_store.write().await;
    if let Some(d) = devices.devices_mut().iter_mut().find(|d| d.id == id && d.user_id == claims.sub) {
        d.is_trusted = true;
        json_ok()
    } else {
        json_error("设备不存在")
    }
}

/// PUT /api/devices/{id}/untrust — 取消可信
async fn untrust_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let devices = state.device_store.write().await;
    if let Some(d) = devices.devices_mut().iter_mut().find(|d| d.id == id && d.user_id == claims.sub) {
        d.is_trusted = false;
        json_ok()
    } else {
        json_error("设备不存在")
    }
}

/// PUT /api/devices/{id}/kick — 踢下线
async fn kick_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let devices = state.device_store.write().await;
    if let Some(d) = devices.devices_mut().iter_mut().find(|d| d.id == id && d.user_id == claims.sub) {
        d.is_active = false;
        json_ok()
    } else {
        json_error("设备不存在")
    }
}

/// PUT /api/devices/kick-other — 踢掉当前用户的其它设备
async fn kick_other_devices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
) -> Json<Value> {
    let devices = state.device_store.write().await;
    let count = devices.devices_mut().iter_mut()
        .filter(|d| d.user_id == claims.sub && d.is_active)
        .map(|d| { d.is_active = false; })
        .count();
    json_success(json!({"kicked": count}))
}

/// DELETE /api/devices/{id} — 删除设备记录
async fn delete_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let devices = state.device_store.write().await;
    let len_before = devices.devices().len();
    devices.devices_mut().retain(|d| !(d.id == id && d.user_id == claims.sub));
    if devices.devices().len() < len_before {
        json_ok()
    } else {
        json_error("设备不存在")
    }
}

/// GET /api/admin/devices — 全部设备列表（管理员）
async fn list_all_devices(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DeviceQuery>,
) -> Json<Value> {
    let devices = state.device_store.read().await;
    let all: Vec<RepoLoginDevice> = devices.devices().clone();
    drop(devices);

    let mut result = all;

    if let Some(ref kw) = q.keyword {
        let kw_lower = kw.to_lowercase();
        result.retain(|d| {
            d.device_name.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower)
                || d.browser.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower)
                || d.os.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower)
                || d.ip_address.to_lowercase().contains(&kw_lower)
        });
    }
    if let Some(ref dt) = q.device_type {
        result.retain(|d| d.device_type == *dt);
    }

    let total = result.len();
    let page = q.page.unwrap_or(1).max(1);
    let size = q.page_size.unwrap_or(20).max(1);
    let start = (page - 1) * size;
    let list: Vec<RepoLoginDevice> = result.into_iter().skip(start).take(size).collect();

    json_success(json!({"list": list, "total": total, "page": page, "page_size": size}))
}

// ==================== 设备扩展 Handler ====================

async fn batch_delete_devices() -> AppResult<Json<Value>> {
    tracing::warn!("batch_delete_devices: 功能未实现");
    Err(AppError::NotImplemented("batch_delete_devices: 功能未实现".to_string()))
}

async fn device_statistics() -> Json<Value> {
    json_success(json!({"total": 0, "online": 0, "abnormal": 0}))
}

async fn device_overview() -> Json<Value> {
    json_success(json!({"total": 0, "today_new": 0, "active": 0}))
}

async fn device_abnormal() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

// ============ 用户设备查询/踢下线 ============

async fn list_user_devices(Path(_user_id): Path<i64>) -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}

async fn kick_all_user_devices(Path(_user_id): Path<i64>) -> Json<Value> { json_ok() }

// ==================== 路由定义 ====================

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/devices", get(list_my_devices))
        .route("/api/devices/my", get(list_my_devices))
        .route("/api/devices/kick-other", put(kick_other_devices))
        .route("/api/devices/batch", delete(batch_delete_devices))
        .route("/api/devices/{id}", get(get_device).delete(delete_device))
        .route("/api/devices/{id}/trust", put(trust_device))
        .route("/api/devices/{id}/untrust", put(untrust_device))
        .route("/api/devices/{id}/kick", put(kick_device))
        .route("/api/admin/devices", get(list_all_devices))
        .route("/api/admin/devices/statistics", get(device_statistics))
        .route("/api/admin/devices/overview", get(device_overview))
        .route("/api/admin/devices/abnormal", get(device_abnormal))
        .route("/api/admin/devices/user/{user_id}", get(list_user_devices))
        .route("/api/admin/devices/user/{user_id}/kick-all", put(kick_all_user_devices))
}
