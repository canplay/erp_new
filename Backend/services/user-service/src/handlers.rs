//! User Service gRPC Handlers
//!
//! 使用 Repository 模式进行数据库操作

use std::net::SocketAddr;
use grpc_proto::user::user_service_server::UserServiceServer;
use chrono::Utc;
use grpc_proto::user::*;
use grpc_proto::user::user_service_server::UserService;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::repository::{
    AnnouncementRepository, DepartmentRepository, RoleRepository, UserRepository,
};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub user_repository: UserRepository,
    pub role_repository: RoleRepository,
    pub department_repository: DepartmentRepository,
    pub announcement_repository: AnnouncementRepository,
}

impl AppState {
    /// 创建新的应用状态
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            user_repository: UserRepository::new(pool.clone()),
            role_repository: RoleRepository::new(pool.clone()),
            department_repository: DepartmentRepository::new(pool.clone()),
            announcement_repository: AnnouncementRepository::new(pool),
        }
    }
}

/// `UserService` 实现
#[derive(Clone)]
#[allow(clippy::too_many_arguments)]
pub struct UserServiceImpl {
    state: Arc<AppState>,
}

impl UserServiceImpl {
    /// 创建新的 `UserService` 实例
    #[must_use]
    pub const fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    include!("user_methods.rs" );
  include!("role_methods.rs" );
  include!("department_methods.rs" );
  include!("dictionary_methods.rs" );
  include!("announcement_methods.rs" );
  include!("system_config_methods.rs" );
  include!("permission_methods.rs" );
}

impl common::service_bootstrap::GrpcServiceBuilder for UserServiceImpl {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}" ).into() })?;
        let server = UserServiceServer::new(UserServiceImpl::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}" , e);
            }
        });
        Ok(handle)
    }
}
