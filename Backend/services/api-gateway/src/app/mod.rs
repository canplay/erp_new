//! Application state management - split by responsibility
//!
//! The original `AppState` had 15 fields mixing gRPC clients,
//! circuit breakers, repositories, and misc config.
//! Now split into focused modules:
//!
//! - `grpc` — gRPC client configuration
//! - `circuit_breaker` — circuit breaker settings
//! - `repositories` — all data stores
//! - `state` — the composed AppState

pub mod grpc;
pub mod circuit_breaker;
pub mod repositories;
pub mod state;
