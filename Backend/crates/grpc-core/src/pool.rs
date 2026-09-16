//! gRPC 连接池
//!
//! 提供 gRPC 连接复用、健康检查和自动重连

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// 连接正常
    Healthy,
    /// 连接断开
    Disconnected,
    /// 连接中
    Connecting,
    /// 连接错误
    Error,
}

/// 单个 gRPC 连接信息
pub struct PooledConnection {
    channel: Channel,
    state: ConnectionState,
    created_at: std::time::Instant,
}

impl PooledConnection {
    /// 检查连接是否过期（超过指定时间）
    #[must_use]
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() > max_age
    }

    /// 获取通道
    #[must_use]
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// 获取状态
    #[must_use]
    pub const fn state(&self) -> ConnectionState {
        self.state
    }
}

/// gRPC 连接池
pub struct GrpcConnectionPool {
    connections: Arc<RwLock<HashMap<String, PooledConnection>>>,
    max_idle_time: Duration,
    max_pool_size: usize,
}

impl GrpcConnectionPool {
    /// 创建新的连接池
    #[must_use]
    pub fn new(max_idle_time_secs: u64, max_pool_size: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_idle_time: Duration::from_secs(max_idle_time_secs),
            max_pool_size,
        }
    }

    /// 获取或创建连接
    pub async fn get_connection(&self, addr: &str) -> Result<Channel> {
        // 先尝试从池中获取
        {
            let mut connections = self.connections.write();

            if let Some(conn) = connections.get(addr) {
                if conn.state == ConnectionState::Healthy && !conn.is_expired(self.max_idle_time) {
                    tracing::debug!("gRPC 连接池命中: {addr}" );
                    return Ok(conn.channel());
                }

                // 清理过期连接
                connections.remove(addr);
            }
        }

        // 创建新连接
        tracing::info!("创建新的 gRPC 连接: {addr}" );
        let channel = tonic::transport::Endpoint::from_shared(addr.to_string())?
            .timeout(Duration::from_secs(30))
            .connect()
            .await?;

        // 添加到池中
        {
            let mut connections = self.connections.write();

            // 清理超过最大池大小的连接
            while connections.len() >= self.max_pool_size {
                if let Some(oldest) = connections.keys().next().cloned() {
                    connections.remove(&oldest);
                    tracing::debug!("清理旧的 gRPC 连接: {oldest}" );
                }
            }

            connections.insert(
                addr.to_string(),
                PooledConnection {
                    channel: channel.clone(),
                    state: ConnectionState::Healthy,
                    created_at: std::time::Instant::now(),
                },
            );
        }

        Ok(channel)
    }

    /// 移除连接
    pub fn remove_connection(&self, addr: &str) {
        let mut connections = self.connections.write();
        connections.remove(addr);
        tracing::debug!("移除 gRPC 连接: {addr}" );
    }

    /// 清理所有连接
    pub fn clear(&self) {
        let mut connections = self.connections.write();
        connections.clear();
        tracing::info!("清理所有 gRPC 连接" );
    }

    /// 获取连接统计
    #[must_use]
    pub fn stats(&self) -> PoolStats {
        let connections = self.connections.read();
        PoolStats {
            total: connections.len(),
            healthy: connections
                .values()
                .filter(|c| c.state == ConnectionState::Healthy)
                .count(),
        }
    }
}

/// 连接池统计
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total: usize,
    pub healthy: usize,
}

impl Clone for GrpcConnectionPool {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            max_idle_time: self.max_idle_time,
            max_pool_size: self.max_pool_size,
        }
    }
}
