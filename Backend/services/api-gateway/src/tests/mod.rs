//! API Gateway 单元测试
//!
//! Comprehensive route testing covering:
//! - 200 OK path
//! - 400 bad parameters
//! - 401 unauthorized
//! - gRPC unavailable degradation (503)

// ============================================================================
// 基础模块测试 (保留原有)
// ============================================================================


mod middleware_tests;
mod route_tests;
