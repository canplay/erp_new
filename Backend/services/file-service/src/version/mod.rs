//! 文件版本管理模块
//!
//! 实现文件版本历史、版本对比、版本回退功能
//! 支持多版本存储、增量存储、版本标签管理

mod helper;
mod manager;
mod tests;
mod types;

pub use types::{
    CompareConfig, DiffChange, DiffType, FileVersion, RollbackRequest, RollbackResult,
    VersionConfig, VersionDiff, VersionHistoryQuery, VersionState, VersionStats,
};
pub use manager::{CreateVersionParams, VersionManager};
