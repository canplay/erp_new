//! 服务发现配置

use serde::Deserialize;

/// 服务发现配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// 重连间隔（秒）
    pub reconnect_interval_secs: u64,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            reconnect_interval_secs: 30,
        }
    }
}

impl ServiceDiscoveryConfig {
    /// 从环境变量读取配置
    #[must_use]
    pub fn from_env() -> Self {
        let reconnect_interval_secs = std::env::var("SERVICE_DISCOVERY_RECONNECT_INTERVAL" )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        Self {
            reconnect_interval_secs,
        }
    }
}
