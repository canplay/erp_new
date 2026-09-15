//! 文件服务库
//!
//! 提供文件上传、下载、预览和管理功能

pub mod grpc_handlers; // gRPC 服务处理器
pub mod grpc_server;
pub mod handlers;
pub mod helpers;
pub mod models;
pub mod repository;
pub mod storage;
pub mod version; // 文件版本管理模块

pub use common::AppError;
pub use common::AppResult;

pub use grpc_handlers::{FileAppState, FileGrpcService, FileInfo};
pub use handlers::{AppState, create_router};
pub use models::*;
pub use repository::FileRepository;
pub use storage::{
    LocalStorage, LocalStorageConfig, StorageBackend, StorageError, generate_file_name,
};

// 导出版本管理类型
pub use version::{
    CompareConfig, DiffChange, DiffType, FileVersion, RollbackRequest, RollbackResult,
    VersionConfig, VersionDiff, VersionHistoryQuery, VersionManager, VersionState, VersionStats,
};
