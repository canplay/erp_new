//! API密钥服务
//!
//! 提供API密钥的创建、验证和管理功能
//! 架构：仅通过 gRPC 对外提供服务，由 api-gateway 接入。

pub mod error;
pub mod grpc_handlers; // gRPC 业务逻辑（数据库持久化）
pub mod grpc_server;   // gRPC 服务 trait 实现
pub mod handlers;      // HTTP handler（即将废弃，保留编译兼容）
pub mod models;
pub mod renewal;
pub mod repository;    // 数据库仓储层

pub use error::{ApiKeyError, Result};
pub use grpc_handlers::{
    ApiKeyAppState, ApiKeyInfo, CreateKeyResponse, KeyStatsInfo, PaginatedApiKeysInfo,
    PaginatedUsageLogsInfo, ValidateKeyResponse,
};
pub use grpc_server::ApiKeyGrpcServer;

pub use repository::{ApiKey, ApiKeyUsageLog, PostgresApiKeyRepository};

pub use renewal::{
    ApiKey as RenewalApiKey, ApiKeyManager, KeyState, KeyStats, KeyType, RenewalConfig,
    RenewalRequest, RotationRequest, RotationResult,
};
