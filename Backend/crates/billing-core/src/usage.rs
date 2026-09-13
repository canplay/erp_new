//! 用量跟踪

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 用量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UsageType {
    /// API 调用次数
    ApiCalls,
    /// 文件上传次数
    FileUploads,
    /// 存储空间 (GB)
    StorageGb,
    /// 活跃用户数
    ActiveUsers,
    /// 消息发送量
    Messages,
    /// 语音通话分钟数
    VoiceMinutes,
    /// 视频通话分钟数
    VideoMinutes,
}

impl UsageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiCalls => "api_calls",
            Self::FileUploads => "file_uploads",
            Self::StorageGb => "storage_gb",
            Self::ActiveUsers => "active_users",
            Self::Messages => "messages",
            Self::VoiceMinutes => "voice_minutes",
            Self::VideoMinutes => "video_minutes",
        }
    }
}

/// 用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub usage_type: UsageType,
    pub quantity: f64,
    pub recorded_at: DateTime<Utc>,
}

/// 用量聚合查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAggregation {
    pub tenant_id: Uuid,
    pub usage_type: UsageType,
    pub period: AggregationPeriod,
    pub total: f64,
    pub average: f64,
    pub peak: f64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

/// 聚合周期
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationPeriod {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl AggregationPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
        }
    }
}
