//! CMS Service Library
//!
//! 内容管理服务库

pub mod grpc_handlers; // gRPC 服务处理器
pub mod grpc_server; // gRPC 服务实现（实现完整的 trait）
pub mod models;
pub mod repository;
pub mod user_service;

pub use grpc_handlers::CmsGrpcService;
pub use grpc_server::CmsGrpcServer;
pub use repository::CmsRepository;
pub use repository::article_repository::ArticleRepositoryError;
pub use repository::category_repository::CategoryRepositoryError;
pub use user_service::{CmsWithUserInfo, UserInfo, UserServiceClient};

/// CMS 应用状态
#[derive(Clone)]
pub struct CmsAppState {
    pub repository: CmsRepository,
}

impl CmsAppState {
    /// 创建新的应用状态
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            repository: CmsRepository::new(pool),
        }
    }
}
