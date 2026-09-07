//! 日志采样模块
//!
//! 提供日志采样功能，减少高吞吐量场景下的日志量。
//! 支持固定采样率、动态采样率、按服务/级别采样等策略。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 采样策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingStrategy {
    /// 固定采样率（例如 1/10 表示每 10 条记录 1 条）
    Fixed(u32),
    /// 动态采样率（根据负载自动调整）
    Dynamic,
    /// 按服务采样（每个服务独立采样）
    ByService,
    /// 按级别采样（不同级别不同采样率）
    ByLevel,
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        SamplingStrategy::Fixed(10)
    }
}

/// 日志采样器
pub struct LogSampler {
    /// 采样策略
    strategy: SamplingStrategy,
    /// 固定采样率的分母（例如 10 表示 1/10）
    fixed_rate: u32,
    /// 当前计数
    counter: AtomicU64,
    /// 采样计数
    sampled_count: AtomicU64,
    /// 总计数
    total_count: AtomicU64,
    /// 按服务分组的计数
    service_counts: parking_lot::RwLock<HashMap<String, u64>>,
    /// 按级别分组的计数
    level_counts: parking_lot::RwLock<HashMap<String, u64>>,
    /// 动态采样率更新间隔
    dynamic_update_interval: Duration,
    /// 上次更新时间
    last_update: parking_lot::Mutex<Instant>,
}

impl LogSampler {
    /// 创建新的日志采样器
    pub fn new(strategy: SamplingStrategy) -> Self {
        Self {
            strategy,
            fixed_rate: 10,
            counter: AtomicU64::new(0),
            sampled_count: AtomicU64::new(0),
            total_count: AtomicU64::new(0),
            service_counts: parking_lot::RwLock::new(HashMap::new()),
            level_counts: parking_lot::RwLock::new(HashMap::new()),
            dynamic_update_interval: Duration::from_secs(60),
            last_update: parking_lot::Mutex::new(Instant::now()),
        }
    }

    /// 创建固定采样率的采样器
    pub fn with_fixed_rate(rate: u32) -> Self {
        Self {
            strategy: SamplingStrategy::Fixed(rate),
            fixed_rate: rate,
            ..Self::new(SamplingStrategy::Fixed(rate))
        }
    }

    /// 判断是否应该记录日志
    pub fn should_log(&mut self, service: &str, level: &str) -> bool {
        self.total_count.fetch_add(1, Ordering::Relaxed);

        match self.strategy {
            SamplingStrategy::Fixed(rate) => {
                let count = self.counter.fetch_add(1, Ordering::Relaxed);
                count % rate as u64 == 0
            }
            SamplingStrategy::Dynamic => {
                self.update_dynamic_rate();
                let count = self.counter.fetch_add(1, Ordering::Relaxed);
                let rate = self.get_dynamic_rate();
                count % rate as u64 == 0
            }
            SamplingStrategy::ByService => {
                let mut service_map = self.service_counts.write();
                let count = service_map.entry(service.to_string()).or_insert(0);
                *count += 1;
                *count % self.fixed_rate as u64 == 0
            }
            SamplingStrategy::ByLevel => {
                let mut level_map = self.level_counts.write();
                let count = level_map.entry(level.to_string()).or_insert(0);
                *count += 1;
                *count % self.fixed_rate as u64 == 0
            }
        }
    }

    /// 更新动态采样率
    fn update_dynamic_rate(&mut self) {
        let mut last_update = self.last_update.lock();
        if last_update.elapsed() >= self.dynamic_update_interval {
            // 简化实现：根据总计数动态调整采样率
            let total = self.total_count.load(Ordering::Relaxed);
            if total > 10000 {
                self.fixed_rate = 20; // 高负载，降低采样率
            } else if total > 5000 {
                self.fixed_rate = 15;
            } else {
                self.fixed_rate = 10;
            }
            *last_update = Instant::now();
        }
    }

    /// 获取动态采样率
    fn get_dynamic_rate(&self) -> u32 {
        self.fixed_rate
    }

    /// 获取采样统计
    pub fn get_stats(&self) -> LogSamplerStats {
        LogSamplerStats {
            total_count: self.total_count.load(Ordering::Relaxed),
            sampled_count: self.sampled_count.load(Ordering::Relaxed),
            fixed_rate: self.fixed_rate,
            strategy: self.strategy,
        }
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
        self.sampled_count.store(0, Ordering::Relaxed);
        self.total_count.store(0, Ordering::Relaxed);
    }
}

/// 日志采样统计
#[derive(Debug, Clone)]
pub struct LogSamplerStats {
    /// 总日志数
    pub total_count: u64,
    /// 采样日志数
    pub sampled_count: u64,
    /// 固定采样率
    pub fixed_rate: u32,
    /// 采样策略
    pub strategy: SamplingStrategy,
}

impl LogSamplerStats {
    /// 获取采样率（百分比）
    pub fn sampling_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 100.0;
        }
        (self.sampled_count as f64 / self.total_count as f64) * 100.0
    }

    /// 获取采样比（例如 1/10 表示 10%）
    pub fn sampling_ratio(&self) -> f64 {
        if self.total_count == 0 {
            return 1.0;
        }
        self.sampled_count as f64 / self.total_count as f64
    }
}

impl Default for LogSampler {
    fn default() -> Self {
        Self::new(SamplingStrategy::Fixed(10))
    }
}

/// 全局日志采样器实例
static GLOBAL_SAMPLER: std::sync::OnceLock<LogSampler> = std::sync::OnceLock::new();

/// 获取全局日志采样器
pub fn global_sampler() -> &'static LogSampler {
    GLOBAL_SAMPLER.get_or_init(LogSampler::default)
}

/// 设置全局日志采样器
pub fn set_global_sampler(sampler: LogSampler) {
    GLOBAL_SAMPLER.set(sampler).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_sampling() {
        let sampler = LogSampler::with_fixed_rate(5);
        let mut count = 0;

        for _ in 0..10 {
            if sampler.should_log("test", "info") {
                count += 1;
            }
        }

        // 固定采样率 5，10 条日志应该采样 2 条
        assert_eq!(count, 2);
    }

    #[test]
    fn test_sampling_stats() {
        let sampler = LogSampler::with_fixed_rate(10);
        for _ in 0..100 {
            sampler.should_log("test", "info");
        }

        let stats = sampler.get_stats();
        assert_eq!(stats.total_count, 100);
        assert_eq!(stats.fixed_rate, 10);
        assert!(stats.sampling_rate() > 0.0 && stats.sampling_rate() <= 100.0);
    }

    #[test]
    fn test_reset() {
        let sampler = LogSampler::with_fixed_rate(5);
        for _ in 0..50 {
            sampler.should_log("test", "info");
        }

        sampler.reset();
        let stats = sampler.get_stats();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.sampled_count, 0);
    }
}
