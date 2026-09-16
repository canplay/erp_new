//! XLT 服务数据模型
//!
//! 定义信路通 MQTT 协议消息类型、停车业务实体和配置结构。

use serde::{Deserialize, Serialize};

/// 设备告警 (`DeviceAlarm`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAlarmData {
    #[serde(default)]
    pub alarm_type: i32,
    #[serde(default)]
    pub alarm_code: i32,
    #[serde(default)]
    pub alarm_desc: String,
    #[serde(default)]
    pub alarm_time: String,
}

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

/// 显示屏配置项 (`SetLCDItems`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcdItemsData {
    pub cmd_name: String,
    pub template: i32,
    #[serde(default)]
    pub ad_id: i32,
    pub items: String,
    pub voice: String,
    pub action: i32,
}

/// 显示屏配置 (`DlPassRule` 内嵌)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassRuleData {
    #[serde(default)]
    pub tcar_timing: i32,
    #[serde(default)]
    pub wcar_timing: i32,
    #[serde(default)]
    pub bcar_timing: i32,
    #[serde(default)]
    pub tcar_act: String,
    #[serde(default)]
    pub wcar_act: String,
    #[serde(default)]
    pub bcar_act: String,
}

/// MQTT 消息信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttEnvelope {
    pub command: String,
    #[serde(default)]
    pub request_id: String,
    pub time: String,
    pub version: String,
    pub sn: String,
    #[serde(default)]
    pub data: String,
}

/// 设备连接信息 (Conn)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnData {
    #[serde(default)]
    pub dev_info: String,
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

/// IO 状态 (`IOStatus`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOStatusData {
    pub no: i32,
    pub state: i32,
}

/// 道闸状态 (`BarrierStatus`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarrierStatusData {
    #[serde(default)]
    pub gate_status: i32,
    #[serde(default)]
    pub command_source: i32,
    #[serde(default)]
    pub error_code: i32,
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

/// 更新完成通知 (`OverUpdate`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverUpdateData {
    #[serde(default)]
    pub passruler: i32,
    #[serde(default)]
    pub carinfo: i32,
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

/// OSS 图片配置 (Image download)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub r#type: i32,
    pub dir: String,
    pub endpoint: String,
    pub bucket_name: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    #[serde(default)]
    pub image_path: i32,
}

/// 使能更新 (`EnableUpdate` download)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableUpdateData {
    #[serde(default)]
    pub passruler: i32,
    #[serde(default)]
    pub carinfo: i32,
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

/// 串口配置 (`SerialConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfigData {
    pub channel: i32,
    #[serde(default)]
    pub baud_rate: i32,
    #[serde(default)]
    pub parity: i32,
    #[serde(default)]
    pub stop: i32,
    #[serde(default)]
    pub databits: i32,
    pub state: Option<i32>,
}

/// 串口数据 (`SerialData`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialDataItem {
    pub channel: i32,
    pub data: String,
    pub len: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialDataMsg {
    pub serial_data: Vec<SerialDataItem>,
}

/// 加密状态/设备
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionData {
    pub status: i32,
}

/// 加密设备 (`EncryptDevice` download)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptDeviceData {
    pub key: String,
    pub enable: i32,
}

/// 更换密钥 (`ChangeEncryptionKey`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeKeyData {
    pub current_key: String,
    pub new_key: String,
}

/// 设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    #[serde(default)]
    pub enable_reply: i32,
    #[serde(default)]
    pub enable_offline: i32,
    #[serde(default)]
    pub enable_iostatus0: i32,
    #[serde(default)]
    pub enable_iostatus1: i32,
    #[serde(default)]
    pub enable_result_video: i32,
}

/// 清除数据 (`ClearData`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearDataMsg {
    #[serde(default)]
    pub clear_car_info: i32,
    #[serde(default)]
    pub clear_result: i32,
    #[serde(default)]
    pub clear_log: i32,
}

/// 通用回复命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyData {
    pub err_code: i32,
    pub err_info: String,
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

/// 计费规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRule {
    pub park_code: String,
    pub free_minutes: i32,
    pub first_hour_fee: i64,
    pub hourly_fee: i64,
    pub daily_cap: i64,
    pub night_fee: i64,
    pub night_start: String,
    pub night_end: String,
}

#[derive(Debug, Deserialize)]
pub struct BillingRequest {
    pub plate_no: String,
    pub park_code: String,
    pub entry_time: String,
    pub exit_time: String,
}

#[derive(Debug, Serialize)]
pub struct BillingResult {
    pub plate_no: String,
    pub park_code: String,
    pub entry_time: String,
    pub exit_time: String,
    pub duration_minutes: i64,
    pub total_amount: i64,
    pub rule: String,
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

/// MQTT 配置
#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub http_port: u16,
    pub client_id: String,
    pub username: String,
    pub password: String,
}

impl MqttConfig {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("XLT_MQTT_HOST" )
                .unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("XLT_MQTT_PORT" )
                .unwrap_or_else(|_| "1883".to_string())
                .parse()
                .unwrap_or(1883),
            http_port: std::env::var("XLT_MQTT_HTTP_PORT" )
                .unwrap_or_else(|_| "18083".to_string())
                .parse()
                .unwrap_or(18083),
            client_id: std::env::var("XLT_MQTT_CLIENT_ID" )
                .unwrap_or_else(|_| "xlt-service".to_string()),
            username: std::env::var("XLT_MQTT_USERNAME" )
                .unwrap_or_else(|_| "admin".to_string()),
            password: std::env::var("XLT_MQTT_PASSWORD" )
                .unwrap_or_else(|_| "public".to_string()),
        }
    }
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// XLT 平台配置
#[derive(Debug, Clone)]
pub struct XltConfig {
    pub redis_url: String,
}

impl XltConfig {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL" )
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        }
    }
}
