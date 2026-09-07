/**
 * @file permission.ts
 * @description 权限相关类型定义和常量
 * @date 2026-05-18
 */

export enum PermissionModule {
  USER = "user",
  ROLE = "role",
  PERMISSION = "permission",
  LOGIN_LOG = "login_log",
  OPERATION_LOG = "operation_log",
  SYSTEM_CONFIG = "system_config",
  DICTIONARY = "dictionary",
  NOTIFICATION = "notification",
  ANNOUNCEMENT = "announcement",
  MONITOR = "monitor",
  FILE = "file",
  TASK = "task",
  IMPORT_EXPORT = "import_export",
  ORGANIZATION = "organization",
  AUDIT = "audit",
}

export enum PermissionScope {
  ALL = "all",
  DEPARTMENT = "department",
  DEPARTMENT_AND_CHILDREN = "department_and_children",
  SELF = "self",
  CUSTOM = "custom",
}

export interface PermissionValidity {
  is_permanent?: boolean;
  valid_from?: string;
  valid_until?: string;
  reminder_days?: number;
  has_validity?: boolean;
  start_time?: string;
  end_time?: string;
  is_recurring?: boolean;
  recurrence_rule?: string;
}

export interface DataPermission {
  permission: string;
  name?: string;
  entity_type?: string;
  scope?: PermissionScope;
  custom_scope?: number[];
  filterGroup?: {
    rules: Array<{ field: string; operator: string; value: unknown }>;
    logic: "and" | "or";
  };
}

export interface FieldPermissionItem {
  field: string;
  entity_type: string;
  canView: boolean;
  canEdit: boolean;
  description?: string;
  sensitive?: boolean;
}

export interface FieldPermission {
  permission?: string;
  name?: string;
  entity_type: string;
  field: string;
  canView: boolean;
  canEdit: boolean;
  allowed_fields?: string[];
  denied_fields?: string[];
  readOnly?: boolean;
  description?: string;
  sensitive?: boolean;
}

export interface PermissionDefinition {
  key: string;
  name: string;
  type?: "function" | "data" | "field";
  category?: "page" | "button" | "field" | "data";
  module?: string | PermissionModule;
  description?: string;
  sensitive?: boolean;
}

export interface RolePermissionConfig {
  role_name: string;
  permissions: string[];
  data_permissions?: DataPermission[];
  field_permissions?: FieldPermission[];
}

export interface PermissionOption {
  key: string;
  name: string;
  type?: "function" | "data" | "field";
  category?: "page" | "button" | "field" | "data";
  module?: string | PermissionModule;
  description?: string;
  sensitive?: boolean;
  value?: string;
  label?: string;
}

export interface ModuleOption {
  value: PermissionModule;
  label: string;
  icon?: string;
}

export interface PermissionGroup {
  module: PermissionModule;
  label: string;
  icon?: string;
  expanded?: boolean;
  permissions: PermissionOption[] | PermissionDefinition[];
}

export interface InheritPermissionInfo {
  role?: string;
  inherit_from?: string[];
  effective_permissions?: string[];
  role_name?: string;
  permissions?: string[];
  depth?: number;
}

export interface PermissionChangeLog {
  id: number;
  operator: string;
  changeType: "add" | "remove" | "update";
  role_name: string;
  permission: string;
  details?: string;
  created_at: string;
}

export interface PermissionDataScope {
  value: string;
  label: string;
  scope?: PermissionScope;
  custom_scope?: number[];
  module?: string | PermissionModule;
  description?: string;
}

export const PERMISSIONS: PermissionOption[] = [
  { key: "system:user:list", name: "查看用户", type: "function", category: "page", module: PermissionModule.USER, description: "查看用户列表和详情" },
  { key: "system:user:create", name: "创建用户", type: "function", category: "button", module: PermissionModule.USER, description: "创建新用户" },
  { key: "system:user:update", name: "更新用户", type: "function", category: "button", module: PermissionModule.USER, description: "修改用户信息" },
  { key: "system:user:delete", name: "删除用户", type: "function", category: "button", module: PermissionModule.USER, description: "删除用户" },
  { key: "system:user:export", name: "导出用户", type: "function", category: "button", module: PermissionModule.USER, description: "导出用户数据" },
  { key: "system:user:import", name: "导入用户", type: "function", category: "button", module: PermissionModule.USER, description: "导入用户数据" },
  { key: "system:role:list", name: "查看角色", type: "function", category: "page", module: PermissionModule.ROLE, description: "查看角色列表" },
  { key: "system:role:create", name: "创建角色", type: "function", category: "button", module: PermissionModule.ROLE, description: "创建新角色" },
  { key: "system:role:update", name: "更新角色", type: "function", category: "button", module: PermissionModule.ROLE, description: "修改角色信息" },
  { key: "system:role:delete", name: "删除角色", type: "function", category: "button", module: PermissionModule.ROLE, description: "删除角色" },
  { key: "system:permission:list", name: "查看权限", type: "function", category: "page", module: PermissionModule.PERMISSION, description: "查看权限列表" },
  { key: "system:permission:assign", name: "分配权限", type: "function", category: "button", module: PermissionModule.PERMISSION, description: "分配权限给角色" },
];

export function getSensitivePermissions(): PermissionOption[] {
  return PERMISSIONS.filter(p => p.sensitive);
}

export function getAllModules(): ModuleOption[] {
  return [
    { value: PermissionModule.USER, label: '用户管理', icon: 'people' },
    { value: PermissionModule.ROLE, label: '角色管理', icon: 'admin_panel_settings' },
    { value: PermissionModule.PERMISSION, label: '权限管理', icon: 'vpn_key' },
    { value: PermissionModule.LOGIN_LOG, label: '登录日志', icon: 'login' },
    { value: PermissionModule.OPERATION_LOG, label: '操作日志', icon: 'history' },
    { value: PermissionModule.SYSTEM_CONFIG, label: '系统配置', icon: 'settings' },
    { value: PermissionModule.DICTIONARY, label: '字典管理', icon: 'menu_book' },
    { value: PermissionModule.NOTIFICATION, label: '通知管理', icon: 'notifications' },
    { value: PermissionModule.ANNOUNCEMENT, label: '公告管理', icon: 'campaign' },
    { value: PermissionModule.MONITOR, label: '监控管理', icon: 'monitor_heart' },
    { value: PermissionModule.FILE, label: '文件管理', icon: 'folder' },
    { value: PermissionModule.TASK, label: '任务管理', icon: 'schedule' },
    { value: PermissionModule.IMPORT_EXPORT, label: '导入导出', icon: 'import_export' },
    { value: PermissionModule.ORGANIZATION, label: '组织管理', icon: 'corporate_fare' },
    { value: PermissionModule.AUDIT, label: '审计管理', icon: 'fact_check' },
  ];
}

/**
 * @brief 权限矩阵配置（用于 API 传输）
 */
export interface PermissionMatrixConfig {
  /** 功能权限列表 */
  function_permissions: string[];
  /** 数据权限列表 */
  data_permissions: DataPermission[];
  /** 字段权限列表 */
  field_permissions: FieldPermission[];
}
