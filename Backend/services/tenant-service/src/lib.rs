//! 租户管理服务库
//!
//! 提供多租户的 CRUD 管理、数据隔离策略和 gRPC 端点。

pub mod grpc_handlers;
pub mod handlers;
pub mod isolation;
pub mod repository;

pub use handlers::*;

// 显式导出 repository 中的 TenantRepository（避免与 handlers 中的 TenantQueryParams 冲突）
pub use repository::TenantRepository;

pub use grpc_handlers::{TenantAppState, TenantGrpcService, TenantInfo};

// 导出版本管理类型
pub use isolation::{
    DataFilter, IsolationLevel, IsolationPolicy, ResourceUsage, Tenant, TenantContext,
    TenantIsolationManager, TenantState, TenantStats,
};
