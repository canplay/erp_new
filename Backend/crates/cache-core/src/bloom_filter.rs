//! 布隆过滤器
//!
//! 用于防止缓存穿透，在查询缓存前快速判断 key 是否存在。
//! 适用于高并发场景下的缓存穿透防护。

use std::collections::HashSet;

/// 布隆过滤器配置
#[derive(Debug, Clone)]
pub struct BloomFilterConfig {
    /// 预期插入的元素数量
    pub expected_elements: usize,
    /// 误判率（越小越准确，范围 0.01-0.1）
    pub false_positive_rate: f64,
}

impl Default for BloomFilterConfig {
    fn default() -> Self {
        Self {
            expected_elements: 1_000_000,
            false_positive_rate: 0.01,
        }
    }
}

/// 布隆过滤器
///
/// 使用多个哈希函数判断元素是否存在于集合中。
/// - 存在：元素可能在集合中（有少量误判）
/// - 不存在：元素一定不在集合中
pub struct BloomFilter {
    bits: Vec<bool>,
    bit_count: usize,
    hash_count: usize,
    config: BloomFilterConfig,
}

impl BloomFilter {
    /// 创建新的布隆过滤器
    pub fn new(config: BloomFilterConfig) -> Self {
        let bit_count = Self::optimal_bit_count(config.expected_elements, config.false_positive_rate);
        let hash_count = Self::optimal_hash_count(bit_count, config.expected_elements);

        BloomFilter {
            bits: vec![false; bit_count as usize],
            bit_count,
            hash_count,
            config,
        }
    }

    /// 计算最优 bit 数组大小
    fn optimal_bit_count(expected: usize, error_rate: f64) -> usize {
        (expected as f64 * (-error_rate.ln()) / (2_f64.ln().powi(2))).ceil() as usize
    }

    /// 计算最优哈希函数数量
    fn optimal_hash_count(bit_count: usize, expected: usize) -> usize {
        ((bit_count as f64 / expected as f64) * 2_f64.ln()).ceil() as usize
    }

    /// 计算 key 的多个哈希值
    fn hash(&self, key: &str) -> Vec<usize> {
        let mut hashes = Vec::with_capacity(self.hash_count);
        for i in 0..self.hash_count {
            let h = self.fnv_hash(key, i as u64);
            hashes.push((h % self.bit_count as u64) as usize);
        }
        hashes
    }

    /// FNV-1a 哈希函数
    fn fnv_hash(&self, key: &str, offset: u64) -> u64 {
        let offset = offset.wrapping_mul(0x100);
        let mut hash = 14695981039346656037u64.wrapping_add(offset);
        for byte in key.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001B3);
        }
        hash
    }

    /// 检查 key 是否存在（可能有误判）
    pub fn contains(&self, key: &str) -> bool {
        let hashes = self.hash(key);
        hashes.iter().all(|&i| self.bits[i])
    }

    /// 添加 key
    pub fn add(&mut self, key: &str) {
        let hashes = self.hash(key);
        for &i in &hashes {
            self.bits[i] = true;
        }
    }

    /// 批量添加 keys
    pub fn add_batch(&mut self, keys: &[&str]) {
        for key in keys {
            self.add(key);
        }
    }

    /// 获取当前 bit 数组大小
    pub fn bit_count(&self) -> usize {
        self.bit_count
    }

    /// 估算当前元素数量
    pub fn estimate_count(&self) -> usize {
        let n = self.bit_count as f64;
        let k = self.hash_count as f64;
        let m = self.bits.len() as f64;
        let filled = self.bits.iter().filter(|&&b| b).count() as f64;
        (n * (-k * filled / m).ln()).ceil() as usize
    }
}

/// 缓存穿透防护器
///
/// 在查询缓存前，先检查布隆过滤器：
/// - 如果 key 不在布隆过滤器中，直接返回缓存未命中（避免查询数据库）
/// - 如果 key 在布隆过滤器中，正常查询缓存
///
/// 对于缓存未命中的 key，可以选择：
/// 1. 不添加到布隆过滤器（允许后续穿透）
/// 2. 添加到布隆过滤器（防止重复穿透，但可能有误判）
pub struct CachePenetrationGuard {
    bloom: BloomFilter,
    /// 缓存未命中时是否添加到布隆过滤器
    add_on_miss: bool,
    /// 缓存未命中的 key 集合（用于短期缓存）
    recent_misses: HashSet<String>,
    /// 未命中 key 的过期时间（秒）
    miss_ttl_secs: u64,
}

impl CachePenetrationGuard {
    /// 创建新的缓存穿透防护器
    pub fn new(config: BloomFilterConfig) -> Self {
        Self {
            bloom: BloomFilter::new(config),
            add_on_miss: true,
            recent_misses: HashSet::new(),
            miss_ttl_secs: 60,
        }
    }

    /// 检查 key 是否可能存在（布隆过滤器）
    pub fn might_exist(&self, key: &str) -> bool {
        self.bloom.contains(key)
    }

    /// 检查 key 是否近期未命中（短期缓存）
    pub fn is_recently_missed(&self, key: &str) -> bool {
        self.recent_misses.contains(key)
    }

    /// 记录 key 未命中
    pub fn record_miss(&mut self, key: &str) {
        if self.add_on_miss {
            self.bloom.add(key);
        }
        self.recent_misses.insert(key.to_string());
    }

    /// 检查 key 是否应该查询缓存
    ///
    /// 返回：
    /// - Ok(true): 应该查询缓存
    /// - Ok(false): 近期未命中，跳过缓存查询
    /// - Err: key 不在布隆过滤器中，直接返回缓存未命中
    pub fn should_query(&self, key: &str) -> Result<bool, ()> {
        if self.is_recently_missed(key) {
            return Ok(false);
        }
        if self.might_exist(key) {
            Ok(true)
        } else {
            Err(())
        }
    }

    /// 清理过期的未命中记录
    pub fn cleanup(&mut self) {
        // 简化实现：定期清理
        if self.recent_misses.len() > 10000 {
            self.recent_misses.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let mut bf = BloomFilter::new(BloomFilterConfig {
            expected_elements: 100,
            false_positive_rate: 0.01,
        });

        assert!(!bf.contains("key1"));
        bf.add("key1");
        assert!(bf.contains("key1"));
        assert!(!bf.contains("key2"));
    }

    #[test]
    fn test_bloom_filter_batch() {
        let mut bf = BloomFilter::default();
        let keys: Vec<&str> = (0..100).map(|i| format!("key{}", i).leak()).collect();
        bf.add_batch(&keys);

        for key in &keys {
            assert!(bf.contains(key));
        }
    }

    #[test]
    fn test_penetration_guard() {
        let mut guard = CachePenetrationGuard::new(BloomFilterConfig::default());

        // 初始状态：key 可能存在
        assert_eq!(guard.should_query("unknown_key"), Ok(true));

        // 记录未命中
        guard.record_miss("unknown_key");
        assert_eq!(guard.should_query("unknown_key"), Ok(false));

        // key 在布隆过滤器中
        assert!(guard.might_exist("unknown_key"));
    }
}
