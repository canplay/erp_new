//! Common library for backend microservices
//!
//! 提供统一的错误处理、响应格式、配置和工具函数

pub mod app_state; // 应用状态管理
pub mod auth;
pub mod bootstrap;
pub mod cache;
pub mod config;
pub mod constants;
pub mod errors;
pub mod grpc_error; // gRPC 错误映射（tonic::Status ↔ AppError）
pub mod grpc_auth; // gRPC 服务间鉴权拦截器
pub mod health; // 健康检查端点
pub mod http; // HTTP 响应构建器
pub mod init;
pub mod metrics; // Prometheus metrics
#[cfg(feature = "metrics-exporter")]
pub mod metrics_exporter; // Prometheus metrics exporter
pub mod middleware;
#[cfg(feature = "telemetry")]
pub mod otel; // OpenTelemetry tracing
pub mod models;
pub mod repository;
pub mod response;
pub mod service_bootstrap;
pub mod service_runner;
pub mod shutdown; // 优雅关闭信号
pub mod utils; // SQL 标识符清理工具（防 SQL 注入）
pub mod validation; // 参数校验和密码工具 // 服务运行器（gRPC + HTTP 双服务）

// Re-export SQL 标识符清理工具（tenant 隔离层经顶层路径使用）
pub use utils::sanitize_identifier;

// Re-export 核心错误与响应类型（全后端最高频引用）
pub use errors::{AppError, AppResult};
pub use response::ApiResponse;

// Re-export HTTP 响应辅助函数（服务经 `common::ok_response` 等顶层路径使用）
pub use http::{
    bad_request_response, internal_error_response, not_found_response, ok_response,
    success_with_message_response, unauthorized_response,
};
