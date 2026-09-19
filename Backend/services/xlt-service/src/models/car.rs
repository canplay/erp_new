//! 车辆相关数据模型
//!
//! 车辆识别、车辆进出场事件、在场车辆、快照、车辆信息查询等。

use serde::{Deserialize, Serialize};

/// 车辆检测 (`VehicleDetection`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleDetectionData {
    #[serde(default)]
    pub has_car: i32,
    #[serde(default)]
    pub detect_time: String,
    #[serde(default)]
    pub trigger_source: i32,
}

/// 车辆识别结果 (Result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultData {
    #[serde(default)]
    pub car_no: String,
    #[serde(default)]
    pub car_noc: String,
    #[serde(default)]
    pub trigg_t: String,
    #[serde(default)]
    pub pic_n: String,
    #[serde(default)]
    pub pic_n_full: String,
    #[serde(default)]
    pub pic_min: String,
    #[serde(default)]
    pub pic_min_full: String,
    #[serde(default)]
    pub pic_n_file: String,
    #[serde(default)]
    pub pic_min_file: String,
    #[serde(default)]
    pub confid: i32,
    #[serde(default)]
    pub img_t: String,
    #[serde(default)]
    pub coordinate: String,
    #[serde(default)]
    pub memo: String,
    #[serde(default)]
    pub encrypted: String,
    #[serde(default)]
    pub open_gate: i32,
    #[serde(default)]
    pub valid_plate: String,
    #[serde(default)]
    pub guid: String,
    #[serde(default)]
    pub fw_confid: i32,
    #[serde(default)]
    pub car_type: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub body_color: String,
    #[serde(default)]
    pub is_car_exist: String,
    #[serde(default)]
    pub result_type: i32,
}

/// 实时检测状态 (Rtd)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtdData {
    pub has_car: i32,
}

/// 车辆保留/恢复 (`CarRetention`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarRetentionData {
    pub car_no: String,
    pub car_noc: String,
    pub trigg_t: String,
    pub encrypted: String,
    pub pic_n: String,
    pub pic_n_full: String,
    pub pic_n_file: String,
}

/// 离线结果 (`OfflineResult`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineResultData {
    #[serde(flatten)]
    pub result: ResultData,
}

/// `CarInfo` 查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarInfoData {
    #[serde(default)]
    pub query_id: String,
    #[serde(default)]
    pub plate: String,
    #[serde(default)]
    pub r#type: i32,
    #[serde(default)]
    pub time: String,
}

/// 快照图片 (`SnapshotPic` upload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotData {
    #[serde(default)]
    pub snapshot_id: String,
    #[serde(default)]
    pub pic_n: String,
    #[serde(default)]
    pub pic_n_full: String,
    #[serde(default)]
    pub pic_n_file: String,
}

/// 车辆信息操作 (`AddCarInfo` / `DeleteCarInfo`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarInfoItem {
    pub plate: String,
    pub r#type: i32,
    #[serde(default)]
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCarInfoData {
    pub car_info_list: Vec<CarInfoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCarInfoData {
    pub plate_list: Vec<String>,
}

/// 查询车辆信息 (`QueryCarInfo`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCarInfoData {
    pub query_id: String,
    pub plate: String,
}

/// 车辆进出场事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleEvent {
    pub plate_no: String,
    #[serde(default)]
    pub plate_color: String,
    pub park_code: String,
    #[serde(default)]
    pub lane_code: String,
    pub event_time: String,
    #[serde(default)]
    pub vehicle_type: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub amount: i64,
    #[serde(default)]
    pub pay_type: String,
}

/// 在场车辆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingVehicle {
    pub plate_no: String,
    pub plate_color: String,
    pub park_code: String,
    pub entry_time: String,
    pub lane_code: String,
    pub vehicle_type: String,
    pub duration_minutes: i64,
    pub amount: i64,
}

/// 车辆进出查询
#[derive(Debug, Deserialize)]
pub struct VehicleQuery {
    pub plate_no: Option<String>,
    pub park_code: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}
