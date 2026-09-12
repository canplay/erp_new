/**
 * 权限码常量定义
 * @file permissions.ts
 * @description 前端路由/按钮级权限码，与后端 permissions 表 code 字段对齐
 * @date 2026-08-07
 *
 * 修复记录 (fix-plan-20260806 P15):
 * 原 routes.ts 中 53 处硬编码 permissions: ['admin']，
 * 导致细粒度 RBAC 失效（非 admin 但拥有具体权限码的用户被拒绝）。
 * 现改为引用本文件常量；守卫逻辑 to.meta.permissions.some(p => hasPermission(p))
 * 不变，hasPermission 中 admin 角色恒 true 保证超管不受影响。
 *
 * 注意：后端 permissions 种子数据目前仅 21 个基础权限码。
 * 业务垂直域（ctp/lpr/xlt/tow/ebike）尚未在种子数据中定义权限码，
 * 暂以 ROLE_ADMIN 标记"仅超管可访问"，待后端补充权限码后逐一替换。
 * Currently using ROLE_ADMIN to mark "superadmin only", will replace with granular permissions after backend support.
 */

/** 仅 admin 角色可访问（业务垂直域等尚未定义权限码的资源） */
export const ROLE_ADMIN = ['admin'] as const;

// ==================== 用户管理 ====================
/** 用户列表页 */
export const PERM_USER_LIST = ['user:list', 'admin'] as const;
/** 用户详情页 */
export const PERM_USER_DETAIL = ['user:list', 'admin'] as const;
/** 重置密码按钮 */
export const PERM_USER_RESET_PASSWORD = ['user:reset_password', 'admin'] as const;
/** 批量操作按钮 */
export const PERM_USER_BATCH = ['user:batch', 'admin'] as const;

// ==================== 角色/权限管理 ====================
/** 角色列表页 */
export const PERM_ROLE_LIST = ['role:list', 'admin'] as const;
/** 角色权限配置页 */
export const PERM_ROLE_PERMISSIONS = ['role:permissions', 'admin'] as const;
/** 权限配置页 */
export const PERM_PERMISSION_MANAGE = ['role:permissions', 'admin'] as const;
/** 权限变更日志页 */
export const PERM_PERMISSION_LOG = ['role:permissions', 'admin'] as const;

// ==================== 部门管理 ====================
/** 部门列表页 */
export const PERM_DEPARTMENT_LIST = ['department:list', 'admin'] as const;

// ==================== 系统配置 ====================
/** 系统设置页 */
export const PERM_CONFIG_VIEW = ['config:view', 'admin'] as const;
/** 系统配置修改 */
export const PERM_CONFIG_UPDATE = ['config:update', 'admin'] as const;

// ==================== 公告管理 ====================
/** 公告列表页 */
export const PERM_ANNOUNCEMENT_LIST = ['announcement:list', 'admin'] as const;

// ==================== 日志审计 ====================
/** 登录日志页 */
export const PERM_LOG_LOGIN = ['log:view', 'admin'] as const;
/** 操作日志页 */
export const PERM_LOG_OPERATION = ['log:view', 'admin'] as const;
/** 敏感操作审计页 */
export const PERM_AUDIT_SENSITIVE = ['log:view', 'admin'] as const;
/** 登录设备页 */
export const PERM_DEVICE_LOGIN = ['log:view', 'admin'] as const;
/** IP白名单页 */
export const PERM_WHITELIST_IP = ['config:update', 'admin'] as const;

// ==================== 消息/通知 ====================
/** 消息通知页 */
export const PERM_MESSAGE_LIST = ['message:list', 'admin'] as const;
/** 通知模板页 */
export const PERM_NOTIFICATION_TEMPLATE = ['message:list', 'admin'] as const;

// ==================== 其他管理 ====================
/** 数据字典页 */
export const PERM_DICTIONARY_LIST = ['config:view', 'admin'] as const;
/** 文件管理页 */
export const PERM_FILE_MANAGE = ['file:manage', 'admin'] as const;
/** 系统监控页 */
export const PERM_MONITOR_VIEW = ['monitor:view', 'admin'] as const;
/** 数据统计页 */
export const PERM_DATA_DASHBOARD = ['data:view', 'admin'] as const;
/** 任务调度页 */
export const PERM_TASK_SCHEDULE = ['task:manage', 'admin'] as const;
/** API密钥页 */
export const PERM_API_KEY_MANAGE = ['api_key:manage', 'admin'] as const;
/** 租户管理页 */
export const PERM_TENANT_MANAGE = ['tenant:manage', 'admin'] as const;
/** 意见反馈页 */
export const PERM_FEEDBACK_LIST = ['feedback:list', 'admin'] as const;
/** 主题设置页 */
export const PERM_THEME_SETTINGS = ['config:view', 'admin'] as const;

// ==================== CMS 内容管理 ====================
/** 文章管理页 */
export const PERM_CMS_ARTICLE = ['cms:article', 'admin'] as const;
/** 分类管理页 */
export const PERM_CMS_CATEGORY = ['cms:category', 'admin'] as const;

// ==================== 工作流 ====================
/** 工作流管理 */
export const PERM_WORKFLOW = ['workflow:manage', 'admin'] as const;

// ==================== 业务垂直域（待后端补充权限码） ====================
/** 报表/数据源管理 */
export const PERM_REPORT_MANAGE = ROLE_ADMIN;
/** 数据源管理 */
export const PERM_DATA_SOURCE_MANAGE = ROLE_ADMIN;
/** 地锁设备管理 (ctp) */
export const PERM_CTP_DEVICE = ROLE_ADMIN;
/** 车牌识别 (lpr) */
export const PERM_LPR_MANAGE = ROLE_ADMIN;
/** 停车看板 (xlt) */
export const PERM_XLT_MANAGE = ROLE_ADMIN;
/** 拖车任务 (tow) */
export const PERM_TOW_MANAGE = ROLE_ADMIN;
/** 电动自行车监管 (ebike) */
export const PERM_EBIKE_MANAGE = ROLE_ADMIN;

/** 权限码对照表（后端 permissions 表 code 字段，schema.sql 2120+ 行） */
export const PERMISSION_CODES = {
  USER_LIST: 'user:list',
  USER_CREATE: 'user:create',
  USER_UPDATE: 'user:update',
  USER_DELETE: 'user:delete',
  USER_RESET_PASSWORD: 'user:reset_password',
  USER_BATCH: 'user:batch',
  ROLE_LIST: 'role:list',
  ROLE_CREATE: 'role:create',
  ROLE_UPDATE: 'role:update',
  ROLE_DELETE: 'role:delete',
  ROLE_PERMISSIONS: 'role:permissions',
  DEPARTMENT_LIST: 'department:list',
  DEPARTMENT_CREATE: 'department:create',
  DEPARTMENT_UPDATE: 'department:update',
  DEPARTMENT_DELETE: 'department:delete',
  CONFIG_VIEW: 'config:view',
  CONFIG_UPDATE: 'config:update',
  ANNOUNCEMENT_LIST: 'announcement:list',
  ANNOUNCEMENT_CREATE: 'announcement:create',
  ANNOUNCEMENT_UPDATE: 'announcement:update',
  ANNOUNCEMENT_DELETE: 'announcement:delete',
} as const;

export type PermissionCode = typeof PERMISSION_CODES[keyof typeof PERMISSION_CODES];
