//! User Service Library
//!
//! 提供用户服务的 gRPC + HTTP 实现

pub mod announcement_handlers;
pub mod announcement_methods;
pub mod department_handlers;
pub mod department_methods;
pub mod dictionary_methods;
pub mod handlers;
pub mod helpers;
pub mod http_handlers;
pub mod permission_methods;
pub mod repository;
pub mod role_handlers;
pub mod role_methods;
pub mod system_config_methods;
pub mod user_methods;

// 导出 Repository 类型
pub use repository::{
    AnnouncementRepository, AnnouncementRepositoryError, DepartmentRepository,
    DepartmentRepositoryError, RoleRepository, RoleRepositoryError, UserRepository,
    UserRepositoryError,
};

// 导出数据模型
pub use repository::{
    Announcement, AnnouncementListItem, DictionaryItem, DictionaryType, LoginLog,
    PaginatedAnnouncements, PaginatedDictionaryItems, PaginatedDictionaryTypes,
    PaginatedLoginLogs, SystemConfig,
};
pub use repository::{Department, DepartmentListItem, DepartmentTreeNode, PaginatedDepartments};
pub use repository::{PaginatedRoles, Permission, Role, RoleListItem, RoleTemplate};
pub use repository::{PaginatedUsers, UserDetail, UserListItem};

// 导出 HTTP Handler 状态
pub use http_handlers::HttpAppState;

// 导出 gRPC Handler
pub use handlers::UserServiceImpl;
