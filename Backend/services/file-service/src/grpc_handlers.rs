//! gRPC Service Handlers for File Service
//!
//! 提供文件管理的 gRPC 接口

use std::net::SocketAddr;
use chrono::{DateTime, Utc};
use grpc_proto::file::file_service_server::FileServiceServer;
use std::sync::Arc;
use tonic::Status;

use crate::models::SysFile;
use crate::repository::FileRepository;

/// File 应用状态
#[derive(Clone)]
pub struct FileAppState {
    pub repository: FileRepository,
}

/// 文件信息 gRPC 响应结构
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub id: i64,
    pub file_name: String,
    pub original_name: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub storage_path: String,
    pub storage_type: String,
    pub url: Option<String>,
    pub md5: Option<String>,
    pub created_by: Option<i64>,
    pub tenant_id: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<SysFile> for FileInfo {
    fn from(file: SysFile) -> Self {
        Self {
            id: file.id,
            file_name: file.file_name,
            original_name: file.original_name,
            file_size: file.file_size,
            mime_type: file.mime_type,
            storage_path: file.storage_path,
            storage_type: file.storage_type,
            url: file.url,
            md5: file.md5,
            created_by: file.created_by,
            tenant_id: file.tenant_id,
            created_at: file.created_at,
        }
    }
}

// ============== 文件管理接口实现 ==============

/// 获取文件列表
pub async fn list_files(
    state: Arc<FileAppState>,
    page: i32,
    page_size: i32,
    category: Option<String>,
    keyword: Option<String>,
) -> Result<(Vec<FileInfo>, i64), Status> {
    let page = if page <= 0 { 1 } else { page as u32 };
    let page_size = if page_size <= 0 { 20 } else { page_size as u32 };

    let (files, total) = state
        .repository
        .find_list(
            page,
            page_size,
            category.as_deref(),
            keyword.as_deref(),
            None,
            None,
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

    let file_infos: Vec<FileInfo> = files.into_iter().map(FileInfo::from).collect();
    Ok((file_infos, total))
}

/// 获取文件详情
pub async fn get_file(state: Arc<FileAppState>, id: i64) -> Result<Option<FileInfo>, Status> {
    state
        .repository
        .find_by_id(id)
        .await
        .map(|opt| opt.map(FileInfo::from))
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 删除文件
pub async fn delete_file(state: Arc<FileAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .delete(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 批量删除文件
pub async fn batch_delete_files(state: Arc<FileAppState>, ids: Vec<i64>) -> Result<u64, Status> {
    state
        .repository
        .batch_delete(&ids)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 检查文件是否被使用
pub async fn check_file_in_use(state: Arc<FileAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .is_file_in_use(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

// ============== 文件分类接口 ==============

/// 获取文件分类列表
pub async fn list_categories() -> Result<Vec<String>, Status> {
    Ok(vec![
        "general".to_string(),
        "image".to_string(),
        "document".to_string(),
        "video".to_string(),
        "audio".to_string(),
        "avatar".to_string(),
        "attachment".to_string(),
    ])
}

/// 获取默认分类
pub async fn get_default_category() -> Result<String, Status> {
    Ok("general".to_string())
}

// ============== 导出服务实现 ==============

impl FileAppState {
    /// 创建新的应用状态
    #[must_use]
    pub const fn new(repository: FileRepository) -> Self {
        Self { repository }
    }

    /// 获取仓储引用
    #[must_use]
    pub const fn repository(&self) -> &FileRepository {
        &self.repository
    }
}

/// File gRPC 服务实现
#[derive(Clone)]
pub struct FileGrpcService {
    state: Arc<FileAppState>,
}

impl FileGrpcService {
    /// 创建新的 gRPC 服务
    #[must_use]
    pub const fn new(state: Arc<FileAppState>) -> Self {
        Self { state }
    }

    /// 获取状态引用
    #[must_use]
    pub const fn state(&self) -> &Arc<FileAppState> {
        &self.state
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for FileGrpcService {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}").into() })?;
        let server = FileServiceServer::new(FileGrpcService::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
