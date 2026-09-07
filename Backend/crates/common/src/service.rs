//! 服务启动辅助模块
//!
//! 提供 gRPC 服务通用的启动和关闭逻辑

use tokio::signal;

/// 服务启动配置
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// 服务名称（用于日志和追踪）
    pub name: String,
    /// gRPC 监听地址
    pub grpc_addr: String,
    /// OTLP 端点（可选）
    pub otlp_endpoint: Option<String>,
}

/// 初始化服务追踪
///
/// 初始化 OpenTelemetry 并集成到 tracing 管线中。
/// 由 `OTEL_ENABLED` 和 `OTEL_EXPORTER_OTLP_ENDPOINT` 环境变量控制。
pub fn init_service_tracing(service_name: &str) {
    tracing::info!("初始化 {service_name} 服务追踪");
    crate::init::init_tracing(service_name);
}

/// 创建带追踪的 shutdown handler
///
/// 返回发送端和接收端，用于 graceful shutdown
#[must_use]
pub fn create_shutdown_handler() -> (
    tokio::sync::broadcast::Sender<()>,
    tokio::sync::broadcast::Receiver<()>,
) {
    tokio::sync::broadcast::channel(1)
}

/// 启动 shutdown 信号监听任务
///
/// 在后台启动一个任务，监听 Ctrl+C 信号并通过 `shutdown_tx` 广播
pub fn spawn_shutdown_listener(
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    service_name: &str,
) {
    let service_name = service_name.to_string();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("收到 Ctrl+C 信号，正在关闭 {service_name}...");
                let _ = shutdown_tx.send(());
            }
            Err(e) => {
                tracing::error!("监听信号失败: {e}");
            }
        }
    });
}

/// 解析 gRPC 地址
pub fn parse_grpc_addr(addr: &str) -> Result<std::net::SocketAddr, String> {
    addr.parse()
        .map_err(|e| format!("无效的 gRPC 地址 '{addr}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_debug() {
        let config = ServiceConfig {
            name: "test-service".into(),
            grpc_addr: "127.0.0.1:50051".into(),
            otlp_endpoint: Some("http://localhost:4317".into()),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("test-service"));
    }

    #[test]
    fn test_parse_grpc_addr_valid() {
        let result = parse_grpc_addr("127.0.0.1:50051");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_grpc_addr_invalid() {
        let result = parse_grpc_addr("invalid");
        assert!(result.is_err());
    }
}
