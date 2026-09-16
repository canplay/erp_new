//! CTP（Centralized Things Platform）锁平台服务库
//!
//! 提供设备数据上报、远程锁控制（开/关/同步）、设备状态查询及列表检索功能。
//! 设备状态缓存于 Redis，控制命令通过 HTTP 转发至 CTP 第三方平台。
//!
//! ⚠️ 注意：
//! - 车牌识别回调相关功能已移至 `lpr-service`（protos/lpr.proto）
//! - 信路通 MQTT 协议相关功能由 `xlt-service` 处理
//!
//! # 路由
//!
//! | 路径 | 方法 | 说明 |
//! |------|------|------|
//! | `/api/ctp/report` | POST | 接收设备上报数据 |
//! | `/api/ctp/device/control` | POST | 发送锁控制命令 |
//! | `/api/ctp/device/{device_no}` | GET | 查询单个设备状态 |
//! | `/api/ctp/device/list` | GET | 设备列表（支持按车场分页） |
//! | `/health` | GET | 健康检查 |

pub use common::AppError;
pub use common::AppResult;

pub mod ctp;
pub mod grpc_server;
pub mod helpers;
pub mod models;
pub mod services;

pub use ctp::AppState;

use axum::{Router, routing::get, routing::post};
use std::sync::Arc;

use crate::ctp::AppState;
use crate::services::CtpDeviceService;

/// 创建 HTTP 应用
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/ctp/report", post(ctp::receive_device_data))
        .route("/api/ctp/device/control", post(ctp::control_lock))
        .route("/api/ctp/device/{device_no}", get(ctp::get_device))
        .route("/api/ctp/device/list", get(ctp::list_devices))
        .route("/health", get(ctp::health))
        .with_state(state)
}

/// 创建服务状态
#[must_use]
pub fn create_state() -> AppState {
    AppState {
        ctp_service: Arc::new(CtpDeviceService::new()),
    }
}
