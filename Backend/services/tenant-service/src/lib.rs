//! 租户管理服务库
//!
//! 提供多租户的 CRUD 管理、数据隔离策略和 gRPC 端点。

pub mod grpc_handlers;
pub mod handlers;
pub mod isolation;
pub mod lifecycle;
pub mod offboarding;
pub mod onboarding;
pub mod repository;

pub use handlers::*;

// 显式导出 repository 中的 TenantRepository（避免与 handlers 中的 TenantQueryParams 冲突）
pub use repository::TenantRepository;

pub use grpc_handlers::{TenantAppState, TenantGrpcService, TenantInfo};

// Re-export common error types for consistency with other services
pub use common::{AppError, AppResult};

// 导出版本管理类型
pub use isolation::{
    DataFilter, IsolationLevel, IsolationPolicy, ResourceUsage, Tenant, TenantContext,
    TenantIsolationManager, TenantState, TenantStats,
};

// 导出 onboarding/offboarding 类型
pub use onboarding::{
    OnboardingError, OnboardingRequest, OnboardingResponse, OnboardingResult, OnboardingService,
    OnboardingStatus, OnboardingStep,
};
pub use offboarding::{
    DataExportFormat, OffboardingError, OffboardingRequest, OffboardingResponse,
    OffboardingResult, OffboardingService, OffboardingStatus, OffboardingStep, TenantDataExport,
    UserExport,
};

// 导出生命周期管理类型
pub use lifecycle::{TenantLifecycleService, TenantState as TenantLifecycleState};
