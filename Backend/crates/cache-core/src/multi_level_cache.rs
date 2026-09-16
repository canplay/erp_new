//! 多级缓存实现
//!
//! 提供 L1（本地 LRU）+ L2（Redis）的多级缓存策略

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

use crate::{JsonLruCache, RedisCache};

/// 多级缓存配置
#[derive(Debug, Clone)]
pub struct MultiLevelCacheConfig {
    /// L1 本地缓存容量
    pub l1_capacity: usize,
    /// L2 Redis 缓存 TTL（秒）
    pub l2_ttl_seconds: u64,
    /// 是否启用 L1 缓存
    pub l1_enabled: bool,
    /// 是否启用 L2 缓存
    pub l2_enabled: bool,
}

impl Default for MultiLevelCacheConfig {
    fn default() -> Self {
        Self {
            l1_capacity: 1000,
            l2_ttl_seconds: 3600,
            l1_enabled: true,
            l2_enabled: true,
        }
    }
}

/// 多级缓存管理器
///
/// 读写流程：
/// 1. 读取时：先查 L1 → 未命中则查 L2 → 未命中则返回 None
/// 2. 写入时：同时写入 L1 和 L2
pub struct MultiLevelCache {
    l1: Option<JsonLruCache>,
    l2: Option<Arc<RedisCache>>,
    config: MultiLevelCacheConfig,
}

impl MultiLevelCache {
    /// 创建多级缓存（同时启用 L1 和 L2）
    pub async fn new(redis_url: &str, config: MultiLevelCacheConfig) -> Result<Self> {
        let l2 = if config.l2_enabled {
            Some(Arc::new(RedisCache::new(redis_url).await?))
        } else {
            None
        };

        let l1 = if config.l1_enabled {
            Some(JsonLruCache::new(config.l1_capacity))
        } else {
            None
        };

        Ok(Self { l1, l2, config })
    }

    /// 创建仅 L1 缓存
    #[must_use]
    pub fn l1_only(capacity: usize) -> Self {
        Self {
            l1: Some(JsonLruCache::new(capacity)),
            l2: None,
            config: MultiLevelCacheConfig {
                l1_enabled: true,
                l2_enabled: false,
                ..Default::default()
            },
        }
    }

    /// 创建仅 L2 缓存
    pub async fn l2_only(redis_url: &str, ttl_seconds: u64) -> Result<Self> {
        let l2 = Some(Arc::new(RedisCache::new(redis_url).await?));

        Ok(Self {
            l1: None,
            l2,
            config: MultiLevelCacheConfig {
                l1_enabled: false,
                l2_enabled: true,
                l2_ttl_seconds: ttl_seconds,
                ..Default::default()
            },
        })
    }

    /// 获取缓存值
    ///
    /// 读取流程：L1 → L2 → None
    pub async fn get<T: DeserializeOwned + Serialize>(&self, key: &str) -> Option<T> {
        // 先查 L1
        if let Some(ref l1) = self.l1
            && let Some(value) = l1.get::<T>(key) {
                tracing::debug!("MultiLevelCache L1 hit: {key}" );
                return Some(value);
            }

        // L1 未命中，查 L2
        if let Some(ref l2) = self.l2
            && let Ok(Some(value)) = l2.get::<T>(key).await {
                tracing::debug!("MultiLevelCache L2 hit: {key}" );

                // 回填 L1
                if let Some(ref l1) = self.l1 {
                    let _ = l1.put(key, &value);
                }

                return Some(value);
            }

        tracing::debug!("MultiLevelCache miss: {key}" );
        None
    }

    /// 设置缓存值
    ///
    /// 写入流程：同时写入 L1 和 L2
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        // 写入 L1
        if let Some(ref l1) = self.l1 {
            l1.put(key, value)?;
        }

        // 写入 L2
        if let Some(ref l2) = self.l2 {
            l2.set(key, value, self.config.l2_ttl_seconds).await?;
        }

        tracing::debug!("MultiLevelCache set: {key}" );
        Ok(())
    }

    /// 删除缓存值
    pub async fn delete(&self, key: &str) -> Result<()> {
        // 删除 L1
        if let Some(ref l1) = self.l1 {
            let _ = l1.remove(key);
        }

        // 删除 L2
        if let Some(ref l2) = self.l2 {
            l2.delete(key).await?;
        }

        tracing::debug!("MultiLevelCache delete: {key}" );
        Ok(())
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        if let Some(ref l1) = self.l1 {
            l1.clear();
        }
    }

    /// 获取缓存统计信息
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            l1_size: self.l1.as_ref().map_or(0, super::lru_cache::JsonLruCache::len),
            l1_enabled: self.config.l1_enabled,
            l2_enabled: self.config.l2_enabled,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub l1_size: usize,
    pub l1_enabled: bool,
    pub l2_enabled: bool,
}

impl Clone for MultiLevelCache {
    fn clone(&self) -> Self {
        Self {
            l1: self.l1.clone(),
            l2: self.l2.clone(),
            config: self.config.clone(),
        }
    }
}
