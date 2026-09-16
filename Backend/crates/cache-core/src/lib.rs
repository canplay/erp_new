//! cache-core - 缓存核心库
//!
//! 提供统一的缓存抽象，支持 Redis 远程缓存和本地 LRU 缓存

mod lru_cache;

#[cfg(feature = "redis-cache" )]
mod multi_level_cache;
#[cfg(feature = "redis-cache" )]
mod redis_cache;

// 导出公共类型
pub use lru_cache::{JsonLruCache, LruCacheStore, StringLruCache};
#[cfg(feature = "redis-cache" )]
pub use multi_level_cache::{CacheStats, MultiLevelCache, MultiLevelCacheConfig};
#[cfg(feature = "redis-cache" )]
pub use redis_cache::RedisCache;
