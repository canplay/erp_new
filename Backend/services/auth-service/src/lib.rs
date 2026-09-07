//! Auth Service Library
//!
//! 提供用户服务的 gRPC + HTTP 实现
//! 支持与 Casdoor 集成实现 SSO

pub mod casdoor;
pub mod handlers;
pub mod repository;

// 导出 Casdoor 相关类型
pub use casdoor::{
    CasdoorClient, CasdoorConfig, CasdoorError, CasdoorSession, CasdoorSessionManager,
    CasdoorTokenResponse, CasdoorUser,
};

pub use handlers::*;
pub use repository::*;
