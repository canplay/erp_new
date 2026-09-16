//! CTP 服务数据模型
//!
//! 定义设备上报数据、锁控制请求、设备状态等 CTP 协议核心数据结构。

use serde::{Deserialize, Serialize};

/// 数据类型枚举（CTP 协议 2.1 节）
///
/// | 值 | 含义 |
/// |---|------|
/// | 0 | 降板（落锁） |
/// | 1 | 升板（解锁） |
/// | 2 | 心跳 |
/// | 5 | 车辆入位 |
/// | 6 | 车辆出位 |
/// | 7 | 复位 |
/// | 8 | 逃费 |
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    /// 降板（落锁）
    Drop = 0,
    /// 升板（解锁）
    Rise = 1,
    /// 心跳
    Heartbeat = 2,
    /// 车辆入位
    VehicleEntry = 5,
    /// 车辆出位
    VehicleExit = 6,
    /// 复位
    Reset = 7,
    /// 逃费
    TheftEvasion = 8,
}

impl DataType {
    /// 从 i32 转换为 DataType，未知值返回 None
    #[must_use]
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(DataType::Drop),
            1 => Some(DataType::Rise),
            2 => Some(DataType::Heartbeat),
            5 => Some(DataType::VehicleEntry),
            6 => Some(DataType::VehicleExit),
            7 => Some(DataType::Reset),
            8 => Some(DataType::TheftEvasion),
            _ => None,
        }
    }

    /// 返回中文描述
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            DataType::Drop => "降板" ,
            DataType::Rise => "升板" ,
            DataType::Heartbeat => "心跳" ,
            DataType::VehicleEntry => "车辆入位" ,
            DataType::VehicleExit => "车辆出位" ,
            DataType::Reset => "复位" ,
            DataType::TheftEvasion => "逃费" ,
        }
    }
}

/// 错误码枚举（CTP 协议 1.5 节）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CtpErrorCode {
    /// 操作成功
    Success = 0,
    /// 请求验证失败
    AuthFailed = 10001,
    /// 请求设备未连接
    DeviceNotConnected = 10003,
    /// 请求参数字符串格式或内容非法
    InvalidParams = 10004,
    /// 请求任务处理超时
    Timeout = 10006,
    /// 请求设备无档案记录
    DeviceNotFound = 10020,
}

impl CtpErrorCode {
    /// 从 i32 转换为 `CtpErrorCode`
    #[must_use]
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => CtpErrorCode::Success,
            10001 => CtpErrorCode::AuthFailed,
            10003 => CtpErrorCode::DeviceNotConnected,
            10004 => CtpErrorCode::InvalidParams,
            10006 => CtpErrorCode::Timeout,
            10020 => CtpErrorCode::DeviceNotFound,
            _ => CtpErrorCode::Success,
        }
    }

    /// 返回中文描述
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            CtpErrorCode::Success => "操作成功" ,
            CtpErrorCode::AuthFailed => "请求验证失败" ,
            CtpErrorCode::DeviceNotConnected => "请求设备未连接" ,
            CtpErrorCode::InvalidParams => "请求参数字符串格式或内容非法" ,
            CtpErrorCode::Timeout => "请求任务处理超时" ,
            CtpErrorCode::DeviceNotFound => "请求设备无档案记录" ,
        }
    }
}

/// 设备上报的原始数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDataUpload {
    pub device_no: String,
    pub data_type: i32,
    pub voltage: Option<String>,
    pub status_one: Option<String>,
    pub status_two: Option<String>,
    pub data_time: Option<String>,
}

/// CTP 平台通用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpResponse {
    pub error_code: i32,
    pub error_msg: String,
}

/// 锁控制命令类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase" )]
pub enum CmdType {
    /// 开锁
    Up,
    /// 关锁
    Down,
    /// 同步状态
    Syn,
}

/// 锁控制请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockControlRequest {
    pub factory_id: String,
    pub device_no: String,
    pub cmd_type: CmdType,
    pub data: Option<String>,
}

/// 锁设备状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockDevice {
    pub id: String,
    pub device_no: String,
    pub factory_id: String,
    pub status: LockStatus,
    pub battery: Option<i32>,
    pub signal: Option<i32>,
    pub voltage: Option<String>,
    pub park_code: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 锁设备状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase" )]
pub enum LockStatus {
    /// 已锁定
    Locked,
    /// 已解锁
    Unlocked,
    /// 离线
    Offline,
    /// 故障
    Fault,
}

/// 设备查询参数
#[derive(Debug, Deserialize)]
pub struct DeviceQuery {
    pub park_code: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 设备列表分页响应
#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<LockDevice>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 命令执行结果
#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub device_no: String,
    pub action: String,
}

/// CTP 平台连接配置
#[derive(Debug, Clone)]
pub struct CtpConfig {
    pub api_base_url: String,
    pub factory_id: String,
    pub app_secret: String,
    pub redis_url: String,
}

impl CtpConfig {
    /// 从环境变量读取 CTP 配置
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            api_base_url: std::env::var("CTP_API_URL" )
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            factory_id: std::env::var("CTP_FACTORY_ID" )
                .unwrap_or_else(|_| "1".to_string()),
            app_secret: std::env::var("CTP_APP_SECRET" )
                .unwrap_or_else(|_| "A1B2C3D4E5F6".to_string()),
            redis_url: std::env::var("REDIS_URL" )
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        }
    }
}
