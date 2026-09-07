//! 反馈服务库
//!
//! 提供用户反馈管理的核心功能

pub mod grpc_handlers;
pub mod grpc_server;
pub mod handlers;
pub mod repository; // gRPC 服务处理器

pub use grpc_handlers::{FeedbackAppState, FeedbackGrpcService, FeedbackInfo};
pub use handlers::*;
pub use repository::*;
