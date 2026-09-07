//! 文件模型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 文件存储类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Local,
    S3,
}

/// 文件记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SysFile {
    /// 文件ID
    pub id: i64,
    /// 文件名（存储用）
    pub file_name: String,
    /// 原始文件名
    pub original_name: String,
    /// 文件大小（字节）
    pub file_size: i64,
    /// MIME类型
    pub mime_type: Option<String>,
    /// 存储路径
    pub storage_path: String,
    /// 存储类型
    pub storage_type: String,
    /// 存储桶（可选）
    pub bucket: Option<String>,
    /// 访问URL
    pub url: Option<String>,
    /// 文件MD5
    pub md5: Option<String>,
    /// 创建者ID
    pub created_by: Option<i64>,
    /// 租户ID
    pub tenant_id: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
    /// 更新时间
    pub updated_at: Option<DateTime<Utc>>,
    /// 删除时间（软删除）
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 文件上传请求
#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadRequest {
    /// 文件名（可选，从 multipart 获取）
    #[serde(default)]
    pub file_name: Option<String>,
    /// 文件类别（用于分组）
    #[serde(default = "default_category")]
    pub category: String,
    /// 租户ID（可选）
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

fn default_category() -> String {
    "general".to_string()
}

/// 文件上传响应
#[derive(Debug, Clone, Serialize)]
pub struct FileUploadResponse {
    /// 文件ID
    pub id: i64,
    /// 文件名
    pub file_name: String,
    /// 原始文件名
    pub original_name: String,
    /// 文件大小
    pub file_size: i64,
    /// MIME类型
    pub mime_type: String,
    /// 访问URL
    pub url: String,
    /// 文件MD5
    pub md5: Option<String>,
}

/// 文件列表查询参数
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileListQuery {
    /// 页码
    #[serde(default = "default_page")]
    pub page: u32,
    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// 文件类别
    #[serde(default)]
    pub category: Option<String>,
    /// 文件名关键词
    #[serde(default)]
    pub keyword: Option<String>,
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

/// 文件列表响应
#[derive(Debug, Clone, Serialize)]
pub struct FileListResponse {
    /// 文件列表
    pub items: Vec<SysFile>,
    /// 总数
    pub total: i64,
    /// 当前页
    pub page: u32,
    /// 每页数量
    pub page_size: u32,
}

/// 文件详情响应
#[derive(Debug, Clone, Serialize)]
pub struct FileDetailResponse {
    /// 文件信息
    pub file: SysFile,
    /// 是否可删除
    pub can_delete: bool,
}

/// 删除文件请求
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteFileRequest {
    /// 文件ID
    pub id: i64,
}

/// 批量删除请求
#[derive(Debug, Clone, Deserialize)]
pub struct BatchDeleteRequest {
    /// 文件ID列表
    pub ids: Vec<i64>,
}
