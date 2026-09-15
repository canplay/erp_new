pub use common::AppError;
pub use common::AppResult;
pub use handlers::{AppState, create_router};
pub use repository::AuditRepository;

pub mod grpc_handlers; // gRPC 服务处理器
pub mod handlers;
pub mod helpers;
pub mod models;
pub mod repository;
pub mod security; // 敏感操作告警模块

pub use grpc_handlers::{AuditAppState, AuditGrpcService, LoginLogInfo, OperationLogInfo};

// 导出安全告警类型
pub use security::{
    AlertCondition, AlertConfig, AlertEvent, AlertLevel, AlertManager, AlertRule, AlertStats,
    AlertType,
};
