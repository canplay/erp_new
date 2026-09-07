//! 常量定义

/// 默认分页大小
pub const DEFAULT_PAGE_SIZE: i32 = 10;

/// 最大分页大小（防止查询过多数据）
pub const MAX_PAGE_SIZE: i32 = 100;

/// 最小分页页码
pub const MIN_PAGE: i32 = 1;

/// 默认密码（用于新用户创建）
pub const DEFAULT_PASSWORD: &str = "User@123456";

/// 密码最小长度
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// 用户名最小长度
pub const MIN_USERNAME_LENGTH: usize = 3;

/// 用户名最大长度
pub const MAX_USERNAME_LENGTH: usize = 50;

/// 允许的角色值
pub const VALID_ROLES: &[&str] = &["user", "admin", "vip"];

/// 允许的状态值
pub const VALID_STATUSES: &[i32] = &[0, 1, 2];

/// 状态: 禁用
pub const STATUS_DISABLED: i32 = 0;

/// 状态: 启用
pub const STATUS_ENABLED: i32 = 1;

/// 状态: 锁定
pub const STATUS_LOCKED: i32 = 2;

/// 角色: 普通用户
pub const ROLE_USER: &str = "user";

/// 角色: 管理员
pub const ROLE_ADMIN: &str = "admin";

/// 角色: VIP
pub const ROLE_VIP: &str = "vip";

/// 导出最大行数
pub const EXPORT_MAX_ROWS: i32 = 10000;

/// 批量操作最大并发数
pub const BATCH_CONCURRENCY: usize = 50;

/// 导入最大文件大小（字节）
pub const IMPORT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
