//! Repository 模块
//!
//! 数据访问层，提供统一的数据库操作接口

mod announcement;
mod department_repository;
mod role;
mod user_repository;

// 导出所有 Repository 和类型
pub(crate) use announcement::{
    AnnouncementRepository,
    CreateAnnouncementParams, CreateDictionaryItemParams, UpdateAnnouncementParams, UpdateDictionaryItemParams,
};
pub(crate) use department_repository::{
    DepartmentRepository, DepartmentRepositoryError,
    DepartmentTreeNode, UpdateDepartmentParams,
};
pub(crate) use role::{
    RoleRepository, RoleRepositoryError,
};
pub(crate) use user_repository::UserRepository;
