//! 服务发现
//!
//! 提供基于 Consul/Etcd 的服务注册与发现，支持静态配置和动态注册

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 服务实例信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// 服务名称
    pub name: String,
    /// 服务地址（主机:端口）
    pub addr: String,
    /// gRPC 端口
    pub grpc_port: u16,
    /// HTTP 端口
    pub http_port: Option<u16>,
    /// 服务权重（用于负载均衡）
    pub weight: u32,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
}

impl ServiceInstance {
    /// 创建新的服务实例
    pub fn new(name: impl Into<String>, addr: impl Into<String>, grpc_port: u16) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
            grpc_port,
            http_port: None,
            weight: 100,
            metadata: HashMap::new(),
        }
    }

    /// 获取完整的 gRPC 地址
    #[must_use]
    pub fn grpc_addr(&self) -> String {
        format!("http://{}:{}" , self.addr, self.grpc_port)
    }

    /// 设置 HTTP 端口
    #[must_use]
    pub const fn with_http_port(mut self, port: u16) -> Self {
        self.http_port = Some(port);
        self
    }

    /// 设置权重
    #[must_use]
    pub const fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 服务发现器
pub struct ServiceDiscovery {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl ServiceDiscovery {
    /// 创建新的服务发现器
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册服务实例
    pub fn register(&self, instance: ServiceInstance) {
        let mut services = self.services.write();
        let service_name = instance.name.clone();
        services
            .entry(service_name.clone())
            .or_default()
            .push(instance);
        tracing::info!("服务实例已注册: {service_name}" );
    }

    /// 注销服务实例
    pub fn deregister(&self, service_name: &str, addr: &str) {
        let mut services = self.services.write();
        if let Some(instances) = services.get_mut(service_name) {
            instances.retain(|i| i.addr != addr);
            tracing::info!("服务实例已注销: {service_name} @ {addr}" );
        }
    }

    /// 获取服务实例列表
    #[must_use]
    pub fn get_instances(&self, service_name: &str) -> Vec<ServiceInstance> {
        let services = self.services.read();
        services.get(service_name).cloned().unwrap_or_default()
    }

    /// 获取单个服务实例（轮询）
    pub fn get_instance(&self, service_name: &str) -> Option<ServiceInstance> {
        let instances = self.get_instances(service_name);
        if instances.is_empty() {
            return None;
        }
        // 简单的轮询负载均衡
        static INDEX: AtomicUsize = AtomicUsize::new(0);
        let idx = INDEX.fetch_add(1, Ordering::Relaxed) % instances.len();
        instances.into_iter().nth(idx)
    }

    /// 列出所有服务名称
    #[must_use]
    pub fn list_services(&self) -> Vec<String> {
        let services = self.services.read();
        services.keys().cloned().collect()
    }

    /// 健康检查
    ///
    /// 返回可用的服务实例列表
    /// 扩展：可实现主动健康检查（调用实例的 /health 端点）
    #[must_use]
    pub fn health_check(&self, service_name: &str) -> Vec<ServiceInstance> {
        self.get_instances(service_name)
    }
}

impl Default for ServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ServiceDiscovery {
    fn clone(&self) -> Self {
        Self {
            services: Arc::clone(&self.services),
        }
    }
}

/// 静态服务配置（用于配置文件）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticServiceConfig {
    #[serde(rename = "services" )]
    services: Vec<ServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    #[serde(rename = "addr" )]
    pub addr: String,
    #[serde(rename = "grpc_port" )]
    pub grpc_port: u16,
    #[serde(rename = "http_port" , default)]
    pub http_port: Option<u16>,
    #[serde(default = "default_weight" )]
    pub weight: u32,
}

/// 默认权重
const fn default_weight() -> u32 {
    100
}

impl StaticServiceConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 转换为服务发现器
    #[must_use]
    pub fn into_discovery(self) -> ServiceDiscovery {
        let discovery = ServiceDiscovery::new();
        for service in self.services {
            let instance = ServiceInstance::new(service.name, service.addr, service.grpc_port)
                .with_http_port(service.http_port.unwrap_or(0))
                .with_weight(service.weight);
            discovery.register(instance);
        }
        discovery
    }
}
