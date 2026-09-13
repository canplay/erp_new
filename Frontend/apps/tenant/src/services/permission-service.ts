/**
 * @file permission-service.ts
 * @brief 权限系统增强服务
 * @date 2026-06-18
 * @description 对接 Backend/Rust 权限管理 API
 */

import { httpClient } from '@/utils/alova';
import type { HttpResponse } from '@/utils/alova';

// ============ 类型定义 ============

/**
 * @brief 数据权限配置
 */
export interface DataPermission {
  id: string;
  role_id: string;
  resource_type: string;
  data_scope: 'all' | 'department' | 'user' | 'custom';
  filter_expression?: string;
  allowed_department_ids?: number[];
  allowed_user_ids?: number[];
  priority: number;
  enabled: boolean;
}

/**
 * @brief 字段权限配置
 */
export interface FieldPermission {
  id: string;
  role_id: string;
  resource_type: string;
  field_name: string;
  permission: 'visible' | 'editable' | 'hidden' | 'masked';
  mask_pattern?: string;
}

/**
 * @brief 角色权限配置
 */
export interface RolePermissionConfig {
  role_name: string;
  function_permissions: string[];
  data_permissions: DataPermission[];
  field_permissions: FieldPermission[];
}

/**
 * @brief 权限矩阵配置
 */
export interface PermissionMatrixConfig {
  role_name: string;
  permissions: Array<{
    resource: string;
    actions: string[];
    data_scope: string;
    fields: string[];
  }>;
}

/**
 * @brief 权限变更日志
 */
export interface PermissionChangeLog {
  id: number;
  role_name: string;
  operator: string;
  change_type: 'add' | 'remove' | 'update';
  permission_type: 'function' | 'data' | 'field';
  resource: string;
  old_value?: string;
  new_value?: string;
  created_at: string;
}

// ============ 角色权限配置 API ============

/**
 * @brief 获取角色完整权限配置
 * @description GET /api/admin/roles/:code/permission-config
 */
export function getRolePermissionConfig(
  role_name: string
): Promise<HttpResponse<RolePermissionConfig>> {
  return httpClient.get<RolePermissionConfig>(
    `/admin/roles/${role_name}/permission-config`
  );
}

/**
 * @brief 更新角色完整权限配置
 * @description PUT /api/admin/roles/:code/permission-config
 */
export function updateRolePermissionConfig(
  role_name: string,
  config: RolePermissionConfig | PermissionMatrixConfig
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(
    `/admin/roles/${role_name}/permission-config`,
    config
  );
}

// ============ 数据权限 API ============

/**
 * @brief 获取角色数据权限配置
 * @description GET /api/admin/roles/:code/data-permissions
 */
export function getRoleDataPermissions(
  role_name: string
): Promise<HttpResponse<DataPermission[]>> {
  return httpClient.get<DataPermission[]>(
    `/admin/roles/${role_name}/data-permissions`
  );
}

/**
 * @brief 更新角色数据权限配置
 * @description PUT /api/admin/roles/:code/data-permissions
 */
export function updateRoleDataPermissions(
  role_name: string,
  dataPermissions: DataPermission[]
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(
    `/admin/roles/${role_name}/data-permissions`,
    { data_permissions: dataPermissions }
  );
}

// ============ 字段权限 API ============

/**
 * @brief 获取角色字段权限配置
 * @description GET /api/admin/roles/:code/field-permissions
 */
export function getRoleFieldPermissions(
  role_name: string
): Promise<HttpResponse<FieldPermission[]>> {
  return httpClient.get<FieldPermission[]>(
    `/admin/roles/${role_name}/field-permissions`
  );
}

/**
 * @brief 更新角色字段权限配置
 * @description PUT /api/admin/roles/:code/field-permissions
 */
export function updateRoleFieldPermissions(
  role_name: string,
  fieldPermissions: FieldPermission[]
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(
    `/admin/roles/${role_name}/field-permissions`,
    { field_permissions: fieldPermissions }
  );
}

// ============ 权限继承 API ============

/**
 * @brief 获取角色继承链
 * @description GET /api/admin/roles/:code/inherit
 */
export function getRoleInheritChain(
  role_name: string
): Promise<HttpResponse<Array<{ parent_role_id: string; child_role_id: string; inherit_data_permissions: boolean; inherit_field_permissions: boolean }>>> {
  return httpClient.get<Array<{ parent_role_id: string; child_role_id: string; inherit_data_permissions: boolean; inherit_field_permissions: boolean }>>(
    `/admin/roles/${role_name}/inherit`
  );
}

/**
 * @brief 设置角色继承
 * @description POST /api/admin/roles/:code/inherit
 */
export function setRoleInherit(
  role_name: string,
  inherit_from: string[]
): Promise<HttpResponse<void>> {
  return httpClient.post<void>(
    `/admin/roles/${role_name}/inherit`,
    { inherit_from: inherit_from }
  );
}

/**
 * @brief 移除角色继承
 * @description DELETE /api/admin/roles/:code/inherit
 */
export function removeRoleInherit(role_name: string): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(
    `/admin/roles/${role_name}/inherit`
  );
}

// ============ 批量权限分配 API ============

/**
 * @brief 批量为多个角色分配相同权限
 * @description POST /api/permissions/batch-assign
 */
export function batchAssignPermissions(
  role_names: string[],
  permissions: string[],
  mode: 'add' | 'set' | 'remove'
): Promise<HttpResponse<void>> {
  return httpClient.post<void>('/permissions/batch-assign', {
    roles: role_names,
    permissions,
    mode,
  });
}

/**
 * @brief 复制角色权限
 * @description POST /api/permissions/copy
 */
export function copyRolePermissions(
  sourceRole: string,
  targetRoles: string[],
  options?: {
    includeDataPermission?: boolean;
    includeFieldPermission?: boolean;
  }
): Promise<HttpResponse<void>> {
  return httpClient.post<void>('/permissions/copy', {
    source_role: sourceRole,
    target_roles: targetRoles,
    include_data_permission: options?.includeDataPermission ?? true,
    include_field_permission: options?.includeFieldPermission ?? true,
  }  );
}

// ============ 权限变更日志 API ============

/**
 * @brief 获取权限变更日志列表
 * @description GET /api/permission-change-logs
 */
export function listPermissionChangeLogs(
  params?: {
    page?: number;
    page_size?: number;
    role_name?: string;
    operator?: string;
    change_type?: 'add' | 'remove' | 'update';
    start_date?: string;
    end_date?: string;
  }
): Promise<HttpResponse<PermissionChangeLog[]>> {
  return httpClient.get<PermissionChangeLog[]>('/permission-change-logs', { params });
}

/**
 * @brief 获取权限变更详情
 * @description GET /api/permission-change-logs/:logId
 */
export function getPermissionChangeLogDetail(
  logId: number
): Promise<HttpResponse<PermissionChangeLog>> {
  return httpClient.get<PermissionChangeLog>(
    `/permission-change-logs/${logId}`
  );
}

/**
 * @brief 导出权限变更日志
 * @description GET /api/permission-change-logs/export
 */
export function exportPermissionChangeLogs(
  params?: {
    role_name?: string;
    start_date?: string;
    end_date?: string;
    format?: 'csv' | 'excel';
  }
): Promise<HttpResponse<Blob>> {
  return httpClient.get<Blob>('/permission-change-logs/export', {
    params,
    responseType: 'blob',
  });
}

// ============ 权限验证 API ============

/**
 * @brief 验证用户数据权限
 * @description POST /api/permissions/validate-data
 */
export function validateDataPermission(
  params: {
    permission: string;
    resource_type: string;
    resource_id: number | string;
  }
): Promise<HttpResponse<{ has_permission: boolean; details?: Record<string, unknown> }>> {
  return httpClient.post<{ has_permission: boolean; details?: Record<string, unknown> }>(
    '/permissions/validate-data',
    params
  );
}

/**
 * @brief 获取用户可访问的部门列表
 * @description GET /api/admin/roles/:code/accessible-departments
 */
export function getAccessibleDepartments(
  role_name: string
): Promise<HttpResponse<Array<{ id: number; name: string }>>> {
  return httpClient.get<Array<{ id: number; name: string }>>(
    `/admin/roles/${role_name}/accessible-departments`
  );
}

/**
 * @brief 获取用户可访问的租户列表
 * @description GET /api/admin/roles/:code/accessible-tenants
 */
export function getAccessibleTenants(
  role_name: string
): Promise<HttpResponse<Array<{ id: number; name: string }>>> {
  return httpClient.get<Array<{ id: number; name: string }>>(
    `/admin/roles/${role_name}/accessible-tenants`
  );
}

// ============ 权限缓存管理 API ============

/**
 * @brief 刷新角色权限缓存
 * @description POST /api/permissions/refresh-cache
 */
export function refreshPermissionCache(
  role_name?: string
): Promise<HttpResponse<void>> {
  const params = role_name ? { role: role_name } : {};
  return httpClient.post<void>('/permissions/refresh-cache', undefined, { params });
}

// ============ 导出 ============

export const permissionService = {
  // 角色权限配置
  getRolePermissionConfig,
  updateRolePermissionConfig,

  // 数据权限
  getRoleDataPermissions,
  updateRoleDataPermissions,

  // 字段权限
  getRoleFieldPermissions,
  updateRoleFieldPermissions,

  // 权限继承
  getRoleInheritChain,
  setRoleInherit,
  removeRoleInherit,

  // 批量权限分配
  batchAssignPermissions,
  copyRolePermissions,

  // 权限变更日志
  listPermissionChangeLogs,
  getPermissionChangeLogDetail,
  exportPermissionChangeLogs,

  // 权限验证
  validateDataPermission,
  getAccessibleDepartments,
  getAccessibleTenants,

  // 权限缓存
  refreshPermissionCache,
};
