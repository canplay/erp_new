//! Circuit breaker configuration
//!
//! Holds the failure threshold, timeout, half-open settings,
//! and the manager instance that owns per-service breakers.

use std::sync::Arc;

use crate::CircuitBreakerConfig;
use crate::CircuitBreakerManager;

/// Circuit breaker configuration bundle.
///
/// Contains the raw threshold/timeout values and the
/// `CircuitBreakerManager` that tracks per-service breaker state.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfigBundle {
    /// Number of failures before opening the circuit.
    pub failure_threshold: u32,

    /// Seconds to wait before transitioning to half-open.
    pub timeout_secs: u64,

    /// Number of half-open requests allowed.
    pub half_open_requests: u32,

    /// The manager that owns per-service `CircuitBreaker` instances.
    pub manager: Arc<CircuitBreakerManager>,
}

impl CircuitBreakerConfigBundle {
    /// Build a `CircuitBreakerConfigBundle` from environment variables.
    pub fn from_env() -> Self {
        let failure_threshold: u32 = std::env::var("CIRCUIT_BREAKER_FAILURE_THRESHOLD" )
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);
        let timeout_secs: u64 = std::env::var("CIRCUIT_BREAKER_TIMEOUT_SECS" )
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);
        let half_open_requests: u32 = std::env::var("CIRCUIT_BREAKER_HALF_OPEN_REQUESTS" )
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let cb_config = CircuitBreakerConfig {
            failure_threshold: f64::from(failure_threshold),
            recovery_timeout_secs: timeout_secs,
            half_open_requests,
            window_size_secs: 60,
            min_requests: 5,
        };

        Self {
            failure_threshold,
            timeout_secs,
            half_open_requests,
            manager: Arc::new(CircuitBreakerManager::new(cb_config)),
        }
    }
}
