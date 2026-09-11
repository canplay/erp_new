//! Application state management - split by responsibility
//!
//! The original `AppState` had 15 fields mixing gRPC clients,
//! circuit breakers, repositories, and misc config.
//! Now split into focused modules:
//!
//! - `grpc` — gRPC client configuration
//! - `circuit_breaker` — circuit breaker settings
//! - `repositories` — all data stores
//!
//! Note: `AppState` itself lives in `lib.rs` to avoid circular dependencies.

pub mod grpc;
pub mod circuit_breaker;
pub mod repositories;
pub mod state;

// Re-exports
pub use circuit_breaker::CircuitBreakerConfigBundle;
pub use grpc::GrpcConfig;
pub use repositories::Repositories;
