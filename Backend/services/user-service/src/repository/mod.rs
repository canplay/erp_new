//! Repository 模块
//!
//! 数据访问层，提供统一的数据库操作接口

mod announcement_repository;
mod department_repository;
mod role_repository;
mod user_repository;

// 导出所有 Repository 和类型
pub use announcement_repository::{
    Announcement, AnnouncementListItem, AnnouncementRepository, AnnouncementRepositoryError,
    DictionaryItem, DictionaryType, LoginLog, PaginatedAnnouncements, PaginatedDictionaryItems,
    PaginatedDictionaryTypes, PaginatedLoginLogs, SystemConfig,
};
pub use department_repository::{
    Department, DepartmentListItem, DepartmentRepository, DepartmentRepositoryError,
    DepartmentTreeNode, PaginatedDepartments,
};
pub use role_repository::{
    PaginatedRoles, Permission, Role, RoleListItem, RoleRepository, RoleRepositoryError,
    RoleTemplate,
};
pub use user_repository::{
    PaginatedUsers, UserDetail, UserListItem, UserRepository, UserRepositoryError,
};
