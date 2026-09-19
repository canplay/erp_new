//! 计费服务库
//!
//! 提供计费计划、订阅、发票和用量管理的 gRPC 接口。

mod handlers;
pub mod models;
mod repository;

pub use common::AppError as BillingError;
pub use common::AppResult;
pub use handlers::BillingAppState;
pub use models::{QuotaCheckResult, SubscriptionStateMachine, UsageSummary};
pub use repository::BillingRepository;
