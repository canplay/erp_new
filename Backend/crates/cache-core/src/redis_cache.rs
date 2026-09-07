//! Redis 缓存实现
//!
//! 提供基于 Redis 的分布式缓存功能

use anyhow::Result;
use redis::{AsyncCommands, Client};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Redis 缓存连接池包装器
pub struct RedisCache {
    client: Arc<Client>,
    connection: Arc<RwLock<redis::aio::MultiplexedConnection>>,
}

impl RedisCache {
    /// 创建新的 Redis 缓存实例
    pub async fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)?;
        let connection = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            client: Arc::new(client),
            connection: Arc::new(RwLock::new(connection)),
        })
    }

    /// 获取缓存值
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.connection.write().await;
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(v) => {
                let deserialized: T = serde_json::from_str(&v)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// 设置缓存值（带 TTL）
    pub async fn set<T: Serialize>(&self, key: &str, value: T, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.connection.write().await;
        let serialized = serde_json::to_string(&value)?;
        let _: () = conn.set_ex(key, serialized, ttl_seconds).await?;
        Ok(())
    }

    /// 设置缓存值（无过期时间）
    pub async fn set_no_expire<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        let mut conn = self.connection.write().await;
        let serialized = serde_json::to_string(&value)?;
        conn.set::<_, _, ()>(key, serialized).await?;
        Ok(())
    }

    /// 删除缓存键
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.connection.write().await;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.connection.write().await;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// 设置 key 的过期时间
    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.connection.write().await;
        conn.expire::<_, ()>(key, ttl_seconds as i64).await?;
        Ok(())
    }

    /// 自增操作
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.connection.write().await;
        let value: i64 = conn.incr(key, 1).await?;
        Ok(value)
    }

    /// 批量获取
    pub async fn mget<T: DeserializeOwned>(&self, keys: Vec<&str>) -> Result<Vec<Option<T>>> {
        let mut conn = self.connection.write().await;
        let values: Vec<Option<String>> = conn.mget(keys).await?;

        let result: Vec<Option<T>> = values
            .into_iter()
            .map(|v| v.and_then(|s| serde_json::from_str(&s).ok()))
            .collect();

        Ok(result)
    }

    /// 批量设置
    pub async fn mset<T: Serialize>(&self, items: Vec<(&str, T, u64)>) -> Result<()> {
        let mut conn = self.connection.write().await;

        for (key, value, ttl) in items {
            let serialized = serde_json::to_string(&value)?;
            conn.set_ex::<_, _, ()>(key, serialized, ttl).await?;
        }

        Ok(())
    }
}

impl Clone for RedisCache {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            connection: Arc::clone(&self.connection),
        }
    }
}
