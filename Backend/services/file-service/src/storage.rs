//! 文件存储模块
//!
//! 支持本地存储和 S3 兼容存储

mod local;

pub use local::{LocalStorage, LocalStorageConfig};

use async_trait::async_trait;
use std::path::Path;

/// 存储后端特征
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 上传文件
    async fn upload(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, StorageError>;

    /// 下载文件
    async fn download(&self, key: &str) -> Result<Vec<u8>, StorageError>;

    /// 删除文件
    async fn delete(&self, key: &str) -> Result<(), StorageError>;

    /// 获取文件访问URL
    fn get_url(&self, key: &str) -> String;
}

/// 存储错误
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("上传失败: {0}" )]
    UploadFailed(String),

    #[error("下载失败: {0}" )]
    DownloadFailed(String),

    #[error("删除失败: {0}" )]
    DeleteFailed(String),

    #[error("文件不存在: {0}" )]
    NotFound(String),

    #[error("存储配置错误: {0}" )]
    ConfigError(String),

    #[error("IO错误: {0}" )]
    IoError(#[from] std::io::Error),
}

/// 生成唯一文件名
#[must_use]
pub fn generate_file_name(original_name: &str, ext: Option<&str>) -> String {
    let uuid = uuid::Uuid::new_v4().to_string().replace('-', "" );
    let extension = ext.unwrap_or_else(|| {
        Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("" )
    });

    if extension.is_empty() {
        uuid
    } else {
        format!("{uuid}.{extension}" )
    }
}

/// 从 MIME 类型推断扩展名
#[must_use]
pub fn extension_from_mime(mime_type: &str) -> &str {
    match mime_type {
        "image/jpeg" => "jpg" ,
        "image/png" => "png" ,
        "image/gif" => "gif" ,
        "image/webp" => "webp" ,
        "image/svg+xml" => "svg" ,
        "application/pdf" => "pdf" ,
        "application/msword" => "doc" ,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx" ,
        "application/vnd.ms-excel" => "xls" ,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx" ,
        "text/plain" => "txt" ,
        "text/html" => "html" ,
        "text/css" => "css" ,
        "text/javascript" => "js" ,
        "application/json" => "json" ,
        "application/xml" => "xml" ,
        "application/zip" => "zip" ,
        "application/x-rar-compressed" => "rar" ,
        _ => "bin" ,
    }
}
