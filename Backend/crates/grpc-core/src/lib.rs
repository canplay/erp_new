//! grpc-core - gRPC 核心库
//!
//! 提供 gRPC 客户端工厂、连接池、服务发现和追踪功能

mod client;
mod discovery;
mod pool;
#[cfg(feature = "telemetry" )]
pub mod tracing;

pub use client::*;
pub use discovery::*;
pub use pool::*;
