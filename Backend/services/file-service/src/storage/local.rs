//! 本地文件存储
//!
//! 将文件存储在本地文件系统中

use super::{StorageBackend, StorageError};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 本地存储配置
#[derive(Debug, Clone)]
pub struct LocalStorageConfig {
    /// 存储根目录
    pub base_path: PathBuf,
    /// 访问基础URL
    pub base_url: String,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./uploads"),
            base_url: "/uploads".to_string(),
        }
    }
}

impl LocalStorageConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            base_path: std::env::var("UPLOAD_BASE_PATH").map_or_else(|_| PathBuf::from("./uploads"), PathBuf::from),
            base_url: std::env::var("UPLOAD_BASE_URL").unwrap_or_else(|_| "/uploads".to_string()),
        }
    }
}

/// 本地存储后端
#[derive(Debug, Clone)]
pub struct LocalStorage {
    config: LocalStorageConfig,
}

impl LocalStorage {
    /// 创建本地存储实例
    pub const fn new(config: LocalStorageConfig) -> Result<Self, StorageError> {
        Ok(Self { config })
    }

    /// 创建默认配置的存储实例
    pub fn default_storage() -> Result<Self, StorageError> {
        let config = LocalStorageConfig::from_env();
        Self::new(config)
    }

    /// 确保存储目录存在
    async fn ensure_dir(&self, path: &Path) -> Result<(), StorageError> {
        if !path.exists() {
            fs::create_dir_all(path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn upload(
        &self,
        key: &str,
        data: &[u8],
        _content_type: &str,
    ) -> Result<String, StorageError> {
        let path = self.config.base_path.join(key);

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            self.ensure_dir(parent).await?;
        }

        // 写入文件
        fs::write(&path, data).await?;

        // 返回相对路径作为存储键
        Ok(key.to_string())
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.config.base_path.join(key);

        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        fs::read(&path)
            .await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.config.base_path.join(key);

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        Ok(())
    }

    fn get_url(&self, key: &str) -> String {
        format!("{}/{}", self.config.base_url.trim_end_matches('/'), key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_storage_path() {
        let config = LocalStorageConfig {
            base_path: PathBuf::from("./uploads"),
            base_url: "/uploads".to_string(),
        };

        let storage = LocalStorage::new(config).expect("storage creation should not fail");
        let path = storage.get_url("abc123.jpg");
        assert_eq!(path, "/uploads/abc123.jpg");
    }
}
