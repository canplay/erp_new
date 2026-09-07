//! Common library for backend microservices
//!
//! 提供统一的错误处理、响应格式、配置和工具函数

pub mod app_state; // 应用状态管理
pub mod auth;
pub mod bootstrap;
pub mod config;
pub mod constants;
pub mod errors;
pub mod grpc_error; // gRPC 错误映射（tonic::Status ↔ AppError）
pub mod grpc_auth; // gRPC 服务间鉴权拦截器
pub use grpc_auth::grpc_auth_interceptor;
pub mod health; // 健康检查端点
pub mod http; // HTTP 响应构建器
pub mod init;
pub mod metrics; // Prometheus metrics
pub mod middleware;
#[cfg(feature = "telemetry")]
pub mod otel; // OpenTelemetry tracing
#[cfg(feature = "telemetry")]
pub use otel::shutdown_otel;
pub mod models;
pub mod repository;
pub mod response;
pub mod service;
pub mod service_bootstrap;
pub mod service_builder;
pub mod service_runner;
pub mod shutdown; // 优雅关闭信号
pub mod validation; // 参数校验和密码工具 // 服务运行器（gRPC + HTTP 双服务）

// Re-export repository types
pub use repository::{
    BaseRepository, CrudRepository, PageQuery, PageResult, RepositoryError, RepositoryResult,
};

// Re-export 导出常用工具函数（定义在 shutdown.rs 和 service_runner.rs 中）
pub use service_runner::{ServiceRunner, ServiceRunnerHandle};
pub use shutdown::{
    shutdown_signal, shutdown_signal_default, shutdown_signal_with_timeout, shutdown_with_name,
};

// Re-export commonly used types
pub use auth::{
    HEADER_USER_ID, HEADER_USER_NAME, HEADER_USER_ROLE, HEADER_USER_TOKEN, UserContext,
};
pub use errors::{AppError, AppResult, error_response};
pub use grpc_error::{GrpcAppErrorExt, GrpcResult, GrpcStatusExt, IntoTonicStatus};
pub use response::ApiResponse;

// Re-export constants
pub use constants::*;

// Re-export config types
pub use config::{
    AppConfig, DatabaseConfig, JwtConfig, PoolConfig, RedisConfig, ServiceConfig,
    ServiceDatabaseConfig, create_db_pool, create_service_db_pool, parse_port_from_env,
};

// Re-export middleware types and functions
pub use middleware::{
    IpRateLimiter, RateLimitConfig,
    cors_layer_with_config, cors_layer_with_env, cors_layer_with_origins,
};

// Re-export CSRF 保护相关
pub use middleware::{HEADER_CSRF_TOKEN, csrf_protection_middleware, generate_csrf_token};

// Re-export service builder (deprecated, use ServiceRunner instead)
#[allow(deprecated)]
pub use service_builder::ServiceBuilder2;

// Re-export HTTP 响应辅助函数
pub use http::{
    bad_request_response, batch_result_response, conflict_response, created_response,
    created_with_id_response, csv_download_response, forbidden_response, internal_error_response,
    list_response, no_content_response, not_found_response, ok_response, paginated_response,
    rate_limit_response, success_with_data_and_message, success_with_message_response,
    unauthorized_response, validate_pagination_params, cors_layer,
};

// Re-export 验证和密码工具
pub use validation::{
    generate_random_password, hash_password, validate_email, validate_id, validate_pagination,
    validate_password, validate_phone, validate_role, validate_status, validate_string_length,
    validate_username, verify_password,
};
