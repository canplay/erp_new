//! XLT 信路通停车管理服务库
//!
//! 提供停车场的车辆入场/出场管理、停车费计算、MQTT 设备通信及道闸控制功能。
//! 设备状态缓存于 Redis，命令通过 EMQX HTTP API 桥接至 MQTT 设备。
//!
//! # 路由
//!
//! | 路径 | 方法 | 说明 |
//! |------|------|------|
//! | `/api/xlt/parking/entry` | POST | 车辆入场 |
//! | `/api/xlt/parking/exit` | POST | 车辆出场 |
//! | `/api/xlt/parking/vehicle/{park_code}/{plate_no}` | GET | 查询在场车辆 |
//! | `/api/xlt/parking/billing` | POST | 计算停车费用 |
//! | `/api/xlt/parking/records` | GET | 车辆进出记录列表 |
//! | `/api/xlt/mqtt/callback` | POST | 接收设备 MQTT 回调 |
//! | `/api/xlt/device/list` | GET | 获取设备列表 |
//! | `/api/xlt/device/open` | POST | 开闸 |
//! | `/api/xlt/device/close` | POST | 关闸 |
//! | `/health` | GET | 健康检查 |

pub use common::AppError;
pub use common::AppResult;

pub mod billing;
pub mod device_manager;
pub mod grpc_server;
pub mod helpers;
pub mod mqtt_gateway;
pub mod parking;
pub mod xlt;

pub use xlt::AppState;

use axum::{Router, routing::get, routing::post};
use std::sync::Arc;

use crate::billing::BillingService;
use crate::device_manager::DeviceManager;
use crate::mqtt_gateway::MqttGateway;
use crate::parking::ParkingService;
use crate::xlt::AppState;
use crate::models::MqttConfig;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/xlt/parking/entry", post(xlt::vehicle_entry))
        .route("/api/xlt/parking/exit", post(xlt::vehicle_exit))
        .route(
            "/api/xlt/parking/vehicle/{park_code}/{plate_no}",
            get(xlt::get_parking_vehicle),
        )
        .route("/api/xlt/parking/billing", post(xlt::calc_billing))
        .route("/api/xlt/parking/records", get(xlt::list_records))
        .route("/api/xlt/mqtt/callback", post(xlt::mqtt_callback))
        .route("/api/xlt/device/list", get(xlt::list_devices))
        .route("/api/xlt/device/open", post(xlt::open_barrier))
        .route("/api/xlt/device/close", post(xlt::close_barrier))
        .route("/health", get(xlt::health))
        .with_state(state)
}

#[must_use]
pub fn create_state() -> AppState {
    let parking_service = Arc::new(ParkingService::new());
    let billing_service = Arc::new(BillingService);
    let mqtt_config = MqttConfig::from_env();
    let mqtt_gateway = Arc::new(MqttGateway::new(mqtt_config));
    let device_manager = Arc::new(DeviceManager::new());

    AppState {
        parking_service,
        billing_service,
        mqtt_gateway,
        device_manager,
    }
}
