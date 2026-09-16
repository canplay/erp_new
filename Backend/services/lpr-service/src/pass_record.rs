//! 车牌识别通行记录模型

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 车牌识别通行记录（数据库行映射）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PassRecord {
    pub id: i64,
    pub plate_no: String,
    pub plate_color: String,
    pub plate_type: String,
    pub vehicle_type: String,
    pub device_id: String,
    pub device_name: String,
    pub park_code: String,
    pub lane_code: String,
    pub direction: String,
    pub pass_time: NaiveDateTime,
    pub image_url: String,
    pub confidence: f32,
    pub status: String,
    pub related_order_id: String,
    pub remark: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建通行记录的输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePassRecord {
    pub plate_no: String,
    pub plate_color: String,
    pub plate_type: String,
    pub vehicle_type: String,
    pub device_id: String,
    pub device_name: String,
    pub park_code: String,
    pub lane_code: String,
    pub direction: String,
    pub pass_time: NaiveDateTime,
    pub image_url: String,
    pub confidence: f32,
}

/// 车辆授权/白名单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleAuthorization {
    pub is_authorized: bool,
    pub auth_type: String,
    pub driver_name: String,
    pub driver_phone: String,
    pub valid_until: Option<String>,
}

/// Signo 开闸请求结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateControlResult {
    pub success: bool,
    pub message: String,
}

/// 通行处理结果汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassProcessResult {
    pub record_id: i64,
    pub plate_no: String,
    pub direction: String,
    pub is_authorized: bool,
    pub gate_opened: bool,
    pub auth_type: String,
    pub driver_name: String,
}

/// 通行统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassStats {
    pub total_pass: i64,
    pub total_entry: i64,
    pub total_exit: i64,
    pub high_confidence: i64,
    pub park_code: String,
}
