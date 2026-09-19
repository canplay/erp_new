//! Repository 模块
//!
//! 数据访问层，提供统一的数据库操作接口

mod announcement_repository;
mod department_repository;
mod role_repository;
mod user_repository;

// 导出所有 Repository 和类型
pub(crate) use announcement_repository::{
    Announcement, AnnouncementListItem, AnnouncementRepository, AnnouncementRepositoryError,
    CreateAnnouncementParams, CreateDictionaryItemParams, DictionaryItem, DictionaryType, LoginLog,
    PaginatedAnnouncements, PaginatedDictionaryItems, PaginatedDictionaryTypes, PaginatedLoginLogs,
    SystemConfig, UpdateAnnouncementParams, UpdateDictionaryItemParams,
};
pub(crate) use department_repository::{
    Department, DepartmentListItem, DepartmentRepository, DepartmentRepositoryError,
    DepartmentTreeNode, PaginatedDepartments, UpdateDepartmentParams,
};
pub(crate) use role_repository::{
    PaginatedRoles, Permission, Role, RoleListItem, RoleRepository, RoleRepositoryError,
    RoleTemplate,
};
pub(crate) use user_repository::{
    PaginatedUsers, UserDetail, UserListItem, UserRepository, UserRepositoryError,
};
