//! LRU 本地缓存实现
//!
//! 提供基于 LRU 策略的本地缓存，适用于热点数据缓存

use lru::LruCache;
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use std::num::NonZeroUsize;
use std::sync::Arc;

/// LRU 本地缓存
pub struct LruCacheStore<K, V> {
    cache: Arc<RwLock<LruCache<K, V>>>,
}

impl<K, V> LruCacheStore<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// 创建新的 LRU 缓存
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MAX));
        Self {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// 获取缓存值
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        cache.get(key).cloned()
    }

    /// 设置缓存值
    pub fn put(&self, key: K, value: V) {
        let mut cache = self.cache.write();
        cache.put(key, value);
    }

    /// 删除缓存值
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        cache.pop(key)
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// 获取缓存大小
    #[must_use]
    pub fn len(&self) -> usize {
        let cache = self.cache.read();
        cache.len()
    }

    /// 检查缓存是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read();
        cache.is_empty()
    }
}

impl<K, V> Clone for LruCacheStore<K, V> {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// 字符串类型的 LRU 缓存（用于简单的字符串缓存）
pub type StringLruCache = LruCacheStore<String, String>;

/// JSON 类型的 LRU 缓存
pub struct JsonLruCache {
    cache: Arc<RwLock<LruCache<String, String>>>,
}

impl JsonLruCache {
    /// 创建新的 JSON LRU 缓存
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MAX));
        Self {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// 获取缓存值并反序列化
    #[must_use]
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut cache = self.cache.write();
        cache.get(key).and_then(|v| serde_json::from_str(v).ok())
    }

    /// 设置缓存值（序列化后存储）
    pub fn put<T: Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let serialized = serde_json::to_string(value)?;
        let mut cache = self.cache.write();
        cache.put(key.to_string(), serialized);
        Ok(())
    }

    /// 删除缓存值
    #[must_use]
    pub fn remove(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write();
        cache.pop(key)
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// 获取缓存大小
    #[must_use]
    pub fn len(&self) -> usize {
        let cache = self.cache.read();
        cache.len()
    }

    /// 检查缓存是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read();
        cache.is_empty()
    }
}

impl Clone for JsonLruCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}
