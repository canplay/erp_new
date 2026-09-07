//! 审计模型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 登录日志记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SysLoginLog {
    /// 日志ID
    pub id: i64,
    /// 用户ID
    pub user_id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// 登录地点
    pub login_location: Option<String>,
    /// 登录状态：1-成功，2-失败
    pub login_status: Option<i16>,
    /// 失败原因
    pub fail_reason: Option<String>,
    /// 登录方式：password, sms, oauth
    pub login_type: Option<String>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
}

/// 登录日志查询参数
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoginLogQuery {
    /// 页码
    #[serde(default = "default_page")]
    pub page: u32,
    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// 用户名
    #[serde(default)]
    pub username: Option<String>,
    /// 登录状态
    #[serde(default)]
    pub status: Option<i16>,
    /// 起始日期
    #[serde(default)]
    pub start_date: Option<String>,
    /// 结束日期
    #[serde(default)]
    pub end_date: Option<String>,
}

const fn default_page() -> u32 {
    1
}
const fn default_page_size() -> u32 {
    20
}

/// 登录日志列表响应
#[derive(Debug, Clone, Serialize)]
pub struct LoginLogResponse {
    pub items: Vec<SysLoginLog>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// 操作日志记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SysOperationLog {
    /// 日志ID
    pub id: i64,
    /// 用户ID
    pub user_id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 操作模块
    pub module: Option<String>,
    /// 业务类型
    pub business_type: Option<String>,
    /// 方法名
    pub method: Option<String>,
    /// 请求方法
    pub request_method: Option<String>,
    /// 请求URL
    pub request_url: Option<String>,
    /// 请求参数
    pub request_params: Option<String>,
    /// 请求体
    pub request_body: Option<String>,
    /// 响应数据
    pub response_data: Option<String>,
    /// 状态：1-成功，2-失败
    pub status: Option<i16>,
    /// 错误信息
    pub error_msg: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time: Option<i32>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
}

/// 操作日志查询参数
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OperationLogQuery {
    /// 页码
    #[serde(default = "default_page")]
    pub page: u32,
    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// 用户名
    #[serde(default)]
    pub username: Option<String>,
    /// 模块
    #[serde(default)]
    pub module: Option<String>,
    /// 业务类型
    #[serde(default)]
    pub business_type: Option<String>,
    /// 操作状态
    #[serde(default)]
    pub status: Option<i16>,
    /// 起始日期
    #[serde(default)]
    pub start_date: Option<String>,
    /// 结束日期
    #[serde(default)]
    pub end_date: Option<String>,
}

/// 操作日志列表响应
#[derive(Debug, Clone, Serialize)]
pub struct OperationLogResponse {
    pub items: Vec<SysOperationLog>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// 操作日志详情
#[derive(Debug, Clone, Serialize)]
pub struct OperationLogDetailResponse {
    pub log: SysOperationLog,
}

/// 批量删除请求
#[derive(Debug, Clone, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 登录统计
#[derive(Debug, Clone, Serialize)]
pub struct LoginStatistics {
    pub total_count: i64,
    pub success_count: i64,
    pub fail_count: i64,
    pub today_count: i64,
    pub today_success: i64,
    pub today_fail: i64,
}

/// 创建登录日志（内部使用）
#[derive(Debug, Clone)]
pub struct CreateLoginLog {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub login_location: Option<String>,
    pub login_status: i16,
    pub fail_reason: Option<String>,
    pub login_type: Option<String>,
}

// ============ API 调用日志 ============

/// API 调用日志记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiCallLog {
    /// 日志ID
    pub id: i64,
    /// 请求ID（用于链路追踪）
    pub request_id: Option<String>,
    /// HTTP方法
    pub method: Option<String>,
    /// 请求路径
    pub path: Option<String>,
    /// 查询参数
    pub query_params: Option<String>,
    /// 请求头
    pub headers: Option<String>,
    /// 请求体大小
    pub request_size: Option<i64>,
    /// 状态码
    pub status_code: Option<i32>,
    /// 响应时间（毫秒）
    pub response_time: Option<i32>,
    /// 响应大小
    pub response_size: Option<i64>,
    /// 客户端IP
    pub client_ip: Option<String>,
    /// 用户ID
    pub user_id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
}

/// API 调用日志查询参数
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApiCallLogQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// HTTP方法筛选
    #[serde(default)]
    pub method: Option<String>,
    /// 路径关键词筛选
    #[serde(default)]
    pub path_keyword: Option<String>,
    /// 状态码筛选（2=2xx, 4=4xx, 5=5xx）
    #[serde(default)]
    pub status_code: Option<i32>,
    /// 最小响应时间
    #[serde(default)]
    pub min_response_time: Option<i32>,
    /// 最大响应时间
    #[serde(default)]
    pub max_response_time: Option<i32>,
    /// 仅错误
    #[serde(default)]
    pub errors_only: Option<bool>,
    /// 起始日期
    #[serde(default)]
    pub start_date: Option<String>,
    /// 结束日期
    #[serde(default)]
    pub end_date: Option<String>,
}

/// API 调用日志列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ApiCallLogResponse {
    pub list: Vec<ApiCallLog>,
    pub total: i64,
}

/// API 调用统计
#[derive(Debug, Clone, Serialize)]
pub struct ApiCallStatistics {
    /// 总调用次数
    pub total_calls: i64,
    /// 成功次数
    pub success_calls: i64,
    /// 失败次数
    pub failed_calls: i64,
    /// 错误率（百分比）
    pub error_rate: f64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: i64,
    /// P50 响应时间
    pub p50_response_time: i64,
    /// P90 响应时间
    pub p90_response_time: i64,
    /// P95 响应时间
    pub p95_response_time: i64,
    /// P99 响应时间
    pub p99_response_time: i64,
    /// 最大响应时间
    pub max_response_time: i64,
    /// 最小响应时间
    pub min_response_time: i64,
    /// 总数据量
    pub total_data_size: i64,
    /// QPS
    pub qps: f64,
}

/// API 端点统计
#[derive(Debug, Clone, Serialize)]
pub struct ApiEndpointStatistics {
    /// 路径
    pub path: String,
    /// 方法
    pub method: String,
    /// 调用次数
    pub call_count: i64,
    /// 成功次数
    pub success_count: i64,
    /// 失败次数
    pub failed_count: i64,
    /// 平均响应时间
    pub avg_response_time: i64,
    /// P95 响应时间
    pub p95_response_time: i64,
    /// 错误率
    pub error_rate: f64,
    /// 分类
    pub category: String,
}

/// API 调用趋势数据点
#[derive(Debug, Clone, Serialize)]
pub struct ApiTrendPoint {
    /// 时间戳
    pub timestamp: i64,
    /// 调用次数
    pub call_count: i64,
    /// 错误次数
    pub error_count: i64,
    /// 平均响应时间
    pub avg_response_time: i64,
    /// P95 响应时间
    pub p95_response_time: i64,
}

/// 响应时间分布
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponseTimeDistribution {
    /// 分桶标签
    pub bucket: String,
    /// 最小值
    pub min: i32,
    /// 最大值
    pub max: i32,
    /// 计数
    pub count: i64,
    /// 百分比
    pub percentage: f64,
}

/// 统计时间范围查询
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApiStatisticsQuery {
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub granularity: Option<String>,
}

/// 创建API调用日志（内部使用）
#[derive(Debug, Clone)]
pub struct CreateApiCallLog {
    pub request_id: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub query_params: Option<String>,
    pub headers: Option<String>,
    pub request_size: Option<i64>,
    pub status_code: i32,
    pub response_time: i32,
    pub response_size: Option<i64>,
    pub client_ip: Option<String>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub error: Option<String>,
}

/// 创建操作日志（内部使用）
#[derive(Debug, Clone)]
pub struct CreateOperationLog {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub module: Option<String>,
    pub business_type: Option<String>,
    pub method: Option<String>,
    pub request_method: Option<String>,
    pub request_url: Option<String>,
    pub request_params: Option<String>,
    pub request_body: Option<String>,
    pub response_data: Option<String>,
    pub status: i16,
    pub error_msg: Option<String>,
    pub execution_time: i32,
    pub ip_address: Option<String>,
}
