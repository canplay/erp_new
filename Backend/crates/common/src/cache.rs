//! 缓存初始化工具
//!
//! 提供统一的缓存创建接口，服务可使用此模块快速初始化多级缓存。

use crate::config::RedisConfig;
use cache_core::{MultiLevelCache, MultiLevelCacheConfig};

/// 创建多级缓存实例
///
/// L1: 本地 LRU 缓存（热点数据）
/// L2: Redis 分布式缓存
///
/// # 参数
/// - `redis_config`: Redis 配置
/// - `l1_capacity`: L1 本地缓存容量（条目数）
/// - `l2_ttl_seconds`: L2 Redis 缓存过期时间（秒）
///
/// # 示例
/// ```ignore
/// use common::cache::create_multi_level_cache;
/// use common::config::RedisConfig;
///
/// let config = RedisConfig::default();
/// let cache = create_multi_level_cache(&config, 1000, 3600).await?;
/// ```
pub async fn create_multi_level_cache(
    redis_config: &RedisConfig,
    l1_capacity: usize,
    l2_ttl_seconds: u64,
) -> anyhow::Result<MultiLevelCache> {
    let config = MultiLevelCacheConfig {
        l1_capacity,
        l2_ttl_seconds,
        l1_enabled: true,
        l2_enabled: true,
    };
    MultiLevelCache::new(&redis_config.url, config).await
}

/// 创建仅 L1 本地缓存（用于无 Redis 的场景）
#[must_use]
pub fn create_l1_cache(capacity: usize) -> MultiLevelCache {
    MultiLevelCache::l1_only(capacity)
}
