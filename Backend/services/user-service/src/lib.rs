//! User Service Library
//!
//! 提供用户服务的 gRPC 实现（业务经 api-gateway 以 gRPC 访问，
//! HTTP 端点仅保留健康检查，见 `main.rs` 中 `common::health::health_routes`）

pub mod handlers;
pub(crate) mod repository;
