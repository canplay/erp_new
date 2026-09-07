//! Auth Core - 认证核心模块
//!
//! 提供 JWT 认证和密码处理的核心功能

pub mod jwt;
pub mod middleware;
pub mod password;

// Re-exports
pub use jwt::{Claims, JwtService};
pub use middleware::{AuthError, AuthUser, AuthenticatedUser, JwtConfig, JwtValidator};
pub use password::PasswordService;
