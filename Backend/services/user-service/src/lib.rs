//! User Service Library
//!
//! 提供用户服务的 gRPC + HTTP 实现

pub(crate) mod announcement_http_handlers;
pub(crate) mod department_http_handlers;
pub mod handlers;
pub(crate) mod helpers;
pub(crate) mod http_handlers;
pub(crate) mod repository;
pub(crate) mod role_http_handlers;

// 导出 Repository 类型
pub(crate) use repository::{
    AnnouncementRepository, AnnouncementRepositoryError, DepartmentRepository,
    DepartmentRepositoryError, RoleRepository, RoleRepositoryError, UserRepository,
    UserRepositoryError,
};

// 导出数据模型
pub(crate) use repository::{
    Announcement, AnnouncementListItem, DictionaryItem, DictionaryType, LoginLog,
    PaginatedAnnouncements, PaginatedDictionaryItems, PaginatedDictionaryTypes,
    PaginatedLoginLogs, SystemConfig,
};
pub(crate) use repository::{Department, DepartmentListItem, DepartmentTreeNode, PaginatedDepartments};
pub(crate) use repository::{PaginatedRoles, Permission, Role, RoleListItem, RoleTemplate};
pub(crate) use repository::{PaginatedUsers, UserDetail, UserListItem};

// 导出 HTTP Handler 状态
pub(crate) use http_handlers::HttpAppState;

// 导出 gRPC Handler
pub(crate) use handlers::UserServiceImpl;
