/**
 * @file permission.ts
 * @description 权限相关类型定义和常量
 * @date 2026-05-18
 */

import { useI18nT } from '@/composables/useI18nT';

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

/**
 * Build a PermissionOption with i18n-resolved name/description.
 * Uses i18nT so name and description are locale-aware at render time.
 */
function permDef(
  key: string,
  type: "function" | "data" | "field",
  category: "page" | "button" | "field" | "data",
  module: PermissionModule,
  opts: { sensitive?: boolean } = {}
): PermissionDefinition {
  const { i18nT } = useI18nT();
  return {
    key,
    name: i18nT(`common.perm.${key.replace(/:/g, '')}`, key),
    type,
    category,
    module,
    description: i18nT(`common.perm.${key.replace(/:/g, '')}Desc`, ''),
    sensitive: opts.sensitive ?? false,
  };
}

/**
 * Static permission metadata (keys, types, categories, modules).
 * Human-readable strings are resolved via i18n in `getPermissions()`.
 */
export const PERMISSION_META: Array<{
  key: string;
  type: "function" | "data" | "field";
  category: "page" | "button" | "field" | "data";
  module: PermissionModule;
  sensitive?: boolean;
}> = [
  { key: "system:user:list",     type: "function", category: "page",   module: PermissionModule.USER },
  { key: "system:user:create",  type: "function", category: "button", module: PermissionModule.USER },
  { key: "system:user:update",  type: "function", category: "button", module: PermissionModule.USER },
  { key: "system:user:delete",  type: "function", category: "button", module: PermissionModule.USER },
  { key: "system:user:export",  type: "function", category: "button", module: PermissionModule.USER },
  { key: "system:user:import",  type: "function", category: "button", module: PermissionModule.USER },
  { key: "system:role:list",    type: "function", category: "page",   module: PermissionModule.ROLE },
  { key: "system:role:create",  type: "function", category: "button", module: PermissionModule.ROLE },
  { key: "system:role:update",  type: "function", category: "button", module: PermissionModule.ROLE },
  { key: "system:role:delete",  type: "function", category: "button", module: PermissionModule.ROLE },
  { key: "system:permission:list",  type: "function", category: "page",   module: PermissionModule.PERMISSION },
  { key: "system:permission:assign", type: "function", category: "button", module: PermissionModule.PERMISSION },
];

/**
 * Build a translated PermissionOption list.
 * Call inside setup so that i18n locale is active.
 */
export function getPermissions(): PermissionDefinition[] {
  return PERMISSION_META.map((m) => permDef(m.key, m.type, m.category, m.module, m));
}

/**
 * Legacy export — empty by default, use getPermissions() for translated list.
 * Kept for import compatibility; consumers should migrate to getPermissions().
 */
export const PERMISSIONS: PermissionOption[] = PERMISSION_META.map((m) => ({
  key: m.key,
  name: m.key,
  type: m.type,
  category: m.category,
  module: m.module,
  sensitive: m.sensitive ?? false,
}));

export function getSensitivePermissions(): PermissionOption[] {
  return getPermissions().filter(p => p.sensitive);
}

/**
 * Build a ModuleOption list with English labels.
 * For translated labels, use getAllModules() inside setup where i18n is active.
 */
export function getAllModules(): ModuleOption[] {
  return [
    { value: PermissionModule.USER, label: 'User Management', icon: 'people' },
    { value: PermissionModule.ROLE, label: 'Role Management', icon: 'admin_panel_settings' },
    { value: PermissionModule.PERMISSION, label: 'Permission Management', icon: 'vpn_key' },
    { value: PermissionModule.LOGIN_LOG, label: 'Login Logs', icon: 'login' },
    { value: PermissionModule.OPERATION_LOG, label: 'Operation Logs', icon: 'history' },
    { value: PermissionModule.SYSTEM_CONFIG, label: 'System Config', icon: 'settings' },
    { value: PermissionModule.DICTIONARY, label: 'Dictionary Mgmt', icon: 'menu_book' },
    { value: PermissionModule.NOTIFICATION, label: 'Notification Mgmt', icon: 'notifications' },
    { value: PermissionModule.ANNOUNCEMENT, label: 'Announcement Mgmt', icon: 'campaign' },
    { value: PermissionModule.MONITOR, label: 'Monitoring', icon: 'monitor_heart' },
    { value: PermissionModule.FILE, label: 'File Management', icon: 'folder' },
    { value: PermissionModule.TASK, label: 'Task Management', icon: 'schedule' },
    { value: PermissionModule.IMPORT_EXPORT, label: 'Import/Export', icon: 'import_export' },
    { value: PermissionModule.ORGANIZATION, label: 'Organization Mgmt', icon: 'corporate_fare' },
    { value: PermissionModule.AUDIT, label: 'Audit Management', icon: 'fact_check' },
  ];
}

/**
 * Get translated module labels. Call inside setup.
 */
export function getAllModulesI18n(): ModuleOption[] {
  const { i18nT } = useI18nT();
  return [
    { value: PermissionModule.USER, label: i18nT('common.moduleUser', 'User Management'), icon: 'people' },
    { value: PermissionModule.ROLE, label: i18nT('common.moduleRole', 'Role Management'), icon: 'admin_panel_settings' },
    { value: PermissionModule.PERMISSION, label: i18nT('common.modulePermission', 'Permission Management'), icon: 'vpn_key' },
    { value: PermissionModule.LOGIN_LOG, label: i18nT('common.moduleLoginLog', 'Login Logs'), icon: 'login' },
    { value: PermissionModule.OPERATION_LOG, label: i18nT('common.moduleOperationLog', 'Operation Logs'), icon: 'history' },
    { value: PermissionModule.SYSTEM_CONFIG, label: i18nT('common.moduleSystemConfig', 'System Config'), icon: 'settings' },
    { value: PermissionModule.DICTIONARY, label: i18nT('common.moduleDictionary', 'Dictionary Mgmt'), icon: 'menu_book' },
    { value: PermissionModule.NOTIFICATION, label: i18nT('common.moduleNotification', 'Notification Mgmt'), icon: 'notifications' },
    { value: PermissionModule.ANNOUNCEMENT, label: i18nT('common.moduleAnnouncement', 'Announcement Mgmt'), icon: 'campaign' },
    { value: PermissionModule.MONITOR, label: i18nT('common.moduleMonitor', 'Monitoring'), icon: 'monitor_heart' },
    { value: PermissionModule.FILE, label: i18nT('common.moduleFile', 'File Management'), icon: 'folder' },
    { value: PermissionModule.TASK, label: i18nT('common.moduleTask', 'Task Management'), icon: 'schedule' },
    { value: PermissionModule.IMPORT_EXPORT, label: i18nT('common.moduleImportExport', 'Import/Export'), icon: 'import_export' },
    { value: PermissionModule.ORGANIZATION, label: i18nT('common.moduleOrganization', 'Organization Mgmt'), icon: 'corporate_fare' },
    { value: PermissionModule.AUDIT, label: i18nT('common.moduleAudit', 'Audit Management'), icon: 'fact_check' },
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
