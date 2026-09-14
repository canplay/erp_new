//! CMS 富文本编辑器后端支持模块
//!
//! 实现图片上传、裁剪、压缩、CDN 同步功能
//!
//! @date 2026-05-16

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 媒体文件类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaType {
    /// 图片
    Image,
    /// 视频
    Video,
    /// 音频
    Audio,
    /// 文档
    Document,
    /// 其他
    Other,
}

impl MediaType {
    /// 从文件扩展名获取媒体类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" | "ico" => MediaType::Image,
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" => MediaType::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" => MediaType::Audio,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" => MediaType::Document,
            _ => MediaType::Other,
        }
    }
}

/// 媒体文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// 文件 ID
    pub id: String,
    /// 文件名
    pub filename: String,
    /// 原始文件名
    pub original_filename: String,
    /// 文件大小（字节）
    pub size: u64,
    /// MIME 类型
    pub mime_type: String,
    /// 媒体类型
    pub media_type: MediaType,
    /// 宽度（图片/视频）
    pub width: Option<u32>,
    /// 高度（图片/视频）
    pub height: Option<u32>,
    /// 持续时间（音频/视频，秒）
    pub duration: Option<f64>,
    /// 上传用户 ID
    pub user_id: i64,
    /// 所属内容 ID
    pub content_id: Option<i64>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: u64,
    /// 访问次数
    pub access_count: u64,
}

/// 图片处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProcessConfig {
    /// 目标宽度
    pub target_width: u32,
    /// 目标高度
    pub target_height: Option<u32>,
    /// 质量（1-100）
    pub quality: u8,
    /// 是否保持宽高比
    pub maintain_aspect_ratio: bool,
    /// 输出格式
    pub output_format: ImageFormat,
    /// 是否生成缩略图
    pub generate_thumbnail: bool,
    /// 缩略图宽度
    pub thumbnail_width: u32,
    /// 水印配置（可选）
    pub watermark: Option<WatermarkConfig>,
}

/// 水印配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkConfig {
    /// 水印类型
    pub watermark_type: WatermarkType,
    /// 水印内容（文字或图片路径）
    pub content: String,
    /// 位置
    pub position: WatermarkPosition,
    /// 透明度（0-100）
    pub opacity: u8,
    /// 字体大小（文字水印）
    pub font_size: Option<u32>,
    /// 边距
    pub margin: u32,
}

/// 水印类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WatermarkType {
    /// 文字水印
    Text,
    /// 图片水印
    Image,
}

/// 水印位置
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

/// 图片输出格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Gif,
}

/// CDN 同步状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CdnSyncStatus {
    /// 等待上传
    Pending,
    /// 上传中
    Uploading,
    /// 已同步
    Synced,
    /// 同步失败
    Failed,
}

/// CDN 同步记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnSyncRecord {
    /// 文件 ID
    pub file_id: String,
    /// CDN 存储路径
    pub cdn_path: String,
    /// CDN URL
    pub cdn_url: String,
    /// 同步状态
    pub status: CdnSyncStatus,
    /// 同步时间
    pub synced_at: Option<u64>,
    /// 同步重试次数
    pub retry_count: u32,
    /// 最后错误信息
    pub last_error: Option<String>,
}

/// 媒体管理器
pub struct MediaManager {
    /// 媒体存储
    media_store: Arc<RwLock<HashMap<String, MediaMetadata>>>,
    /// CDN 同步记录
    cdn_records: Arc<RwLock<HashMap<String, CdnSyncRecord>>>,
    /// 配置
    config: MediaConfig,
}

/// 媒体配置
#[derive(Debug, Clone)]
pub struct MediaConfig {
    /// 最大文件大小（字节）
    pub max_file_size: u64,
    /// 允许的文件扩展名
    pub allowed_extensions: Vec<String>,
    /// 默认图片质量
    pub default_quality: u8,
    /// 默认缩略图宽度
    pub default_thumbnail_width: u32,
    /// CDN 基础 URL
    pub cdn_base_url: String,
    /// 本地存储路径
    pub local_storage_path: String,
    /// 自动压缩阈值
    pub auto_compress_threshold: u64,
    /// 图片最大宽度
    pub max_image_width: u32,
    /// 图片最大高度
    pub max_image_height: u32,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_extensions: vec![
                "jpg".to_string(), "jpeg".to_string(), "png".to_string(),
                "gif".to_string(), "webp".to_string(), "bmp".to_string(),
                "mp4".to_string(), "webm".to_string(),
                "mp3".to_string(), "wav".to_string(),
                "pdf".to_string(),
            ],
            default_quality: 85,
            default_thumbnail_width: 200,
            cdn_base_url: "https://cdn.example.com".to_string(),
            local_storage_path: "/data/media".to_string(),
            auto_compress_threshold: 5 * 1024 * 1024, // 5MB
            max_image_width: 4096,
            max_image_height: 4096,
        }
    }
}

impl MediaManager {
    /// 创建新的媒体管理器
    pub fn new(config: MediaConfig) -> Self {
        Self {
            media_store: Arc::new(RwLock::new(HashMap::new())),
            cdn_records: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 生成文件 ID
    pub fn generate_file_id(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| {
                tracing::error!("系统时间早于 UNIX epoch");
                std::time::Duration::from_secs(0)
            })
            .as_nanos();
        format!("media_{:x}", timestamp)
    }

    /// 验证文件扩展名
    pub fn validate_extension(&self, filename: &str) -> Result<String, MediaError> {
        let ext = filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();

        if ext.is_empty() {
            return Err(MediaError::InvalidFilename);
        }

        if !self.config.allowed_extensions.contains(&ext) {
            return Err(MediaError::UnsupportedFormat);
        }

        Ok(ext)
    }

    /// 验证文件大小
    pub fn validate_size(&self, size: u64) -> Result<(), MediaError> {
        if size > self.config.max_file_size {
            return Err(MediaError::FileTooLarge);
        }
        Ok(())
    }

    /// 注册媒体文件
    pub fn register_media(&self, metadata: MediaMetadata) {
        let mut store = self.media_store.write();
        store.insert(metadata.id.clone(), metadata.clone());

        // 初始化 CDN 同步记录
        let mut cdn_records = self.cdn_records.write();
        cdn_records.insert(metadata.id.clone(), CdnSyncRecord {
            file_id: metadata.id.clone(),
            cdn_path: format!("{}/{}/{}.{}",
                self.config.cdn_base_url,
                metadata.media_type.to_string(),
                metadata.id,
                metadata.filename.rsplit('.').next().unwrap_or("")
            ),
            cdn_url: format!("{}/{}/{}.{}",
                self.config.cdn_base_url,
                metadata.media_type.to_string(),
                metadata.id,
                metadata.filename.rsplit('.').next().unwrap_or("")
            ),
            status: CdnSyncStatus::Pending,
            synced_at: None,
            retry_count: 0,
            last_error: None,
        });

        tracing::info!("注册媒体文件: id={}, filename={}", metadata.id, metadata.filename);
    }

    /// 获取媒体元数据
    pub fn get_metadata(&self, file_id: &str) -> Option<MediaMetadata> {
        let store = self.media_store.read();
        store.get(file_id).cloned()
    }

    /// 更新媒体元数据
    pub fn update_metadata(&self, file_id: &str, updates: MediaUpdates) -> Result<(), MediaError> {
        let mut store = self.media_store.write();

        if let Some(media) = store.get_mut(file_id) {
            if let Some(tags) = updates.tags {
                media.tags = tags;
            }
            if let Some(content_id) = updates.content_id {
                media.content_id = Some(content_id);
            }
            tracing::info!("更新媒体元数据: id={}", file_id);
            Ok(())
        } else {
            Err(MediaError::MediaNotFound)
        }
    }

    /// 删除媒体文件
    pub fn delete_media(&self, file_id: &str) -> bool {
        let mut store = self.media_store.write();
        let mut cdn_records = self.cdn_records.write();

        let removed = store.remove(file_id).is_some();
        cdn_records.remove(file_id);

        if removed {
            tracing::info!("删除媒体文件: id={}", file_id);
        }
        removed
    }

    /// 记录媒体访问
    pub fn record_access(&self, file_id: &str) {
        let mut store = self.media_store.write();
        if let Some(media) = store.get_mut(file_id) {
            media.access_count += 1;
        }
    }

    /// 按内容 ID 获取媒体
    pub fn get_media_by_content(&self, content_id: i64) -> Vec<MediaMetadata> {
        let store = self.media_store.read();
        store
            .values()
            .filter(|m| m.content_id == Some(content_id))
            .cloned()
            .collect()
    }

    /// 按标签获取媒体
    pub fn get_media_by_tag(&self, tag: &str) -> Vec<MediaMetadata> {
        let store = self.media_store.read();
        store
            .values()
            .filter(|m| m.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// 配置图片处理
    pub fn create_image_process_config(
        &self,
        target_width: u32,
        target_height: Option<u32>,
        maintain_aspect_ratio: bool,
        output_format: ImageFormat,
    ) -> ImageProcessConfig {
        ImageProcessConfig {
            target_width,
            target_height,
            quality: self.config.default_quality,
            maintain_aspect_ratio,
            output_format,
            generate_thumbnail: true,
            thumbnail_width: self.config.default_thumbnail_width,
            watermark: None,
        }
    }

    /// 配置文字水印
    pub fn add_text_watermark(
        &self,
        config: &mut ImageProcessConfig,
        text: &str,
        position: WatermarkPosition,
        opacity: u8,
    ) {
        config.watermark = Some(WatermarkConfig {
            watermark_type: WatermarkType::Text,
            content: text.to_string(),
            position,
            opacity,
            font_size: Some(16),
            margin: 10,
        });
    }

    /// 配置图片水印
    pub fn add_image_watermark(
        &self,
        config: &mut ImageProcessConfig,
        watermark_path: &str,
        position: WatermarkPosition,
        opacity: u8,
    ) {
        config.watermark = Some(WatermarkConfig {
            watermark_type: WatermarkType::Image,
            content: watermark_path.to_string(),
            position,
            opacity,
            font_size: None,
            margin: 10,
        });
    }

    /// 获取 CDN 同步状态
    pub fn get_cdn_status(&self, file_id: &str) -> Option<CdnSyncRecord> {
        let records = self.cdn_records.read();
        records.get(file_id).cloned()
    }

    /// 更新 CDN 同步状态
    pub fn update_cdn_status(
        &self,
        file_id: &str,
        status: CdnSyncStatus,
        error: Option<String>,
    ) -> Result<(), MediaError> {
        let mut records = self.cdn_records.write();

        if let Some(record) = records.get_mut(file_id) {
            record.status = status;
            if status == CdnSyncStatus::Synced {
                record.synced_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_else(|_| {
                            tracing::error!("系统时间早于 UNIX epoch");
                            std::time::Duration::from_secs(0)
                        })
                        .as_secs()
                );
            }
            if let Some(err) = error {
                record.last_error = Some(err);
                record.retry_count += 1;
            }
            tracing::info!("更新 CDN 同步状态: file_id={}, status={:?}", file_id, status);
            Ok(())
        } else {
            Err(MediaError::MediaNotFound)
        }
    }

    /// 获取待同步的媒体列表
    pub fn get_pending_sync(&self) -> Vec<String> {
        let records = self.cdn_records.read();
        records
            .iter()
            .filter(|(_, r)| r.status == CdnSyncStatus::Pending || r.status == CdnSyncStatus::Failed)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 获取媒体统计
    pub fn get_stats(&self) -> MediaStats {
        let store = self.media_store.read();
        let records = self.cdn_records.read();

        let total_size: u64 = store.values().map(|m| m.size).sum();
        let synced_count = records.values().filter(|r| r.status == CdnSyncStatus::Synced).count();

        let by_type: HashMap<String, usize> = store
            .values()
            .fold(HashMap::new(), |mut acc, m| {
                let key = format!("{:?}", m.media_type);
                *acc.entry(key).or_insert(0) += 1;
                acc
            });

        MediaStats {
            total_count: store.len(),
            total_size_bytes: total_size,
            synced_count,
            pending_count: records.len() - synced_count,
            by_type,
        }
    }

    /// 清理未使用的媒体
    pub fn cleanup_unused(&self, older_than_secs: u64) -> usize {
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| {
                tracing::error!("系统时间早于 UNIX epoch");
                std::time::Duration::from_secs(0)
            })
            .as_secs()
            .saturating_sub(older_than_secs);

        let mut store = self.media_store.write();
        let mut to_remove = Vec::new();

        for (id, media) in store.iter() {
            if media.access_count == 0 && media.created_at < cutoff {
                to_remove.push(id.clone());
            }
        }

        for id in &to_remove {
            store.remove(id);
        }

        tracing::info!("清理未使用媒体: count={}", to_remove.len());
        to_remove.len()
    }
}

/// 媒体更新
#[derive(Debug, Clone)]
pub struct MediaUpdates {
    pub tags: Option<Vec<String>>,
    pub content_id: Option<i64>,
}

/// 媒体统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStats {
    pub total_count: usize,
    pub total_size_bytes: u64,
    pub synced_count: usize,
    pub pending_count: usize,
    pub by_type: HashMap<String, usize>,
}

/// 媒体错误
#[derive(Debug, Clone, Copy)]
pub enum MediaError {
    /// 文件名无效
    InvalidFilename,
    /// 不支持的格式
    UnsupportedFormat,
    /// 文件过大
    FileTooLarge,
    /// 媒体不存在
    MediaNotFound,
    /// 上传失败
    UploadFailed,
    /// 处理失败
    ProcessingFailed,
    /// CDN 同步失败
    CdnSyncFailed,
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaError::InvalidFilename => write!(f, "无效的文件名"),
            MediaError::UnsupportedFormat => write!(f, "不支持的文件格式"),
            MediaError::FileTooLarge => write!(f, "文件过大"),
            MediaError::MediaNotFound => write!(f, "媒体不存在"),
            MediaError::UploadFailed => write!(f, "上传失败"),
            MediaError::ProcessingFailed => write!(f, "处理失败"),
            MediaError::CdnSyncFailed => write!(f, "CDN 同步失败"),
        }
    }
}

/// 生成 CDN URL
pub fn generate_cdn_url(base_url: &str, media_type: &str, file_id: &str, filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("");
    format!("{}/{}/{}.{}", base_url, media_type, file_id, ext)
}

/// 计算图片缩放尺寸
pub fn calculate_scaled_size(
    original_width: u32,
    original_height: u32,
    max_width: u32,
    max_height: Option<u32>,
    maintain_aspect_ratio: bool,
) -> (u32, u32) {
    let target_max_width = max_width;
    let target_max_height = max_height.unwrap_or(max_width);

    if !maintain_aspect_ratio {
        return (target_max_width, target_max_height);
    }

    let width_ratio = target_max_width as f64 / original_width as f64;
    let height_ratio = target_max_height as f64 / original_height as f64;
    let ratio = width_ratio.min(height_ratio);

    (
        (original_width as f64 * ratio) as u32,
        (original_height as f64 * ratio) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_extension() {
        assert_eq!(MediaType::from_extension("jpg"), MediaType::Image);
        assert_eq!(MediaType::from_extension("PNG"), MediaType::Image);
        assert_eq!(MediaType::from_extension("mp4"), MediaType::Video);
        assert_eq!(MediaType::from_extension("pdf"), MediaType::Document);
    }

    #[test]
    fn test_media_manager() {
        let manager = MediaManager::new(MediaConfig::default());

        // 验证扩展名
        assert!(manager.validate_extension("test.jpg").is_ok());
        assert!(manager.validate_extension("test.xyz").is_err());

        // 验证文件大小
        assert!(manager.validate_size(1024 * 1024).is_ok()); // 1MB
        assert!(manager.validate_size(100 * 1024 * 1024).is_err()); // 100MB

        // 注册媒体
        let metadata = MediaMetadata {
            id: "test123".to_string(),
            filename: "test.jpg".to_string(),
            original_filename: "original.jpg".to_string(),
            size: 1024,
            mime_type: "image/jpeg".to_string(),
            media_type: MediaType::Image,
            width: Some(1920),
            height: Some(1080),
            duration: None,
            user_id: 1,
            content_id: Some(100),
            tags: vec!["test".to_string()],
            created_at: 0,
            access_count: 0,
        };

        manager.register_media(metadata);

        // 获取元数据
        let retrieved = manager.get_metadata("test123").expect("metadata should exist");
        assert_eq!(retrieved.filename, "test.jpg");

        // 记录访问
        manager.record_access("test123");
        let updated = manager.get_metadata("test123").expect("metadata should exist");
        assert_eq!(updated.access_count, 1);

        // 获取统计
        let stats = manager.get_stats();
        assert_eq!(stats.total_count, 1);
    }

    #[test]
    fn test_calculate_scaled_size() {
        let (w, h) = calculate_scaled_size(1920, 1080, 800, Some(600), true);
        assert!(w <= 800);
        assert!(h <= 600);

        let (w, h) = calculate_scaled_size(1920, 1080, 800, Some(600), false);
        assert_eq!(w, 800);
        assert_eq!(h, 600);
    }

    #[test]
    fn test_generate_cdn_url() {
        let url = generate_cdn_url("https://cdn.example.com", "image", "media123", "test.jpg");
        assert!(url.starts_with("https://cdn.example.com/image/media123."));
    }
}