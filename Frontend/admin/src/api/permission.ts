/**
 * @file permission.ts
 * @description 权限管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type {
  DataPermission,
  FieldPermission,
  RolePermissionConfig,
  PermissionDefinition,
  PermissionMatrixConfig,
} from '@/types/permission';

// ============ 权限项管理 ============

/**
 * @brief 获取所有权限项列表
 */
export function listPermissions() {
  return httpClient.get('/admin/permissions');
}

/**
 * @brief 创建权限项
 */
export function createPermission(
  data: Omit<PermissionDefinition, 'key'> & { key?: string }
) {
  return httpClient.post('/admin/permissions', data);
}

/**
 * @brief 更新权限项
 */
export function updatePermission(
  key: string,
  data: Partial<Omit<PermissionDefinition, 'key'>>
) {
  return httpClient.put(`/admin/permissions/${encodeURIComponent(key)}`, data);
}

/**
 * @brief 删除权限项
 */
export function deletePermission(key: string) {
  return httpClient.delete(`/admin/permissions/${encodeURIComponent(key)}`);
}

/**
 * @brief 批量创建权限项
 */
export function batchCreatePermissions(
  permissions: Array<Omit<PermissionDefinition, 'key'>>
) {
  return httpClient.post('/admin/permissions/batch', { permissions });
}

// ============ 角色权限管理 ============

/**
 * @brief 获取角色完整权限配置（包含功能、数据、字段权限）
 */
export function getRolePermissionConfig(role_name: string) {
  return httpClient.get(`/admin/roles/${role_name}/permission-config`);
}

/**
 * @brief 更新角色完整权限配置
 * @description 接受两种格式：RolePermissionConfig 或 PermissionMatrixConfig
 */
export function updateRolePermissionConfig(
  role_name: string,
  config: RolePermissionConfig | PermissionMatrixConfig
) {
  return httpClient.put(`/admin/roles/${role_name}/permission-config`, config);
}

/**
 * @brief 获取角色数据权限配置
 */
export function getRoleDataPermissions(role_name: string) {
  return httpClient.get(`/admin/roles/${role_name}/data-permissions`);
}

/**
 * @brief 更新角色数据权限配置
 */
export function updateRoleDataPermissions(
  role_name: string,
  dataPermissions: DataPermission[]
) {
  return httpClient.put(`/admin/roles/${role_name}/data-permissions`, {
    data_permissions: dataPermissions,
  });
}

/**
 * @brief 获取角色字段权限配置
 */
export function getRoleFieldPermissions(role_name: string) {
  return httpClient.get(`/admin/roles/${role_name}/field-permissions`);
}

/**
 * @brief 更新角色字段权限配置
 */
export function updateRoleFieldPermissions(
  role_name: string,
  fieldPermissions: FieldPermission[]
) {
  return httpClient.put(`/admin/roles/${role_name}/field-permissions`, {
    field_permissions: fieldPermissions,
  });
}

// ============ 继承权限 ============

/**
 * @brief 获取角色继承链
 */
export function getRoleInheritChain(role_name: string) {
  return httpClient.get(`/admin/roles/${role_name}/inherit`);
}

/**
 * @brief 设置角色继承
 */
export function setRoleInherit(role_name: string, inherit_from: string[]) {
  return httpClient.post(`/admin/roles/${role_name}/inherit`, {
    inherit_from: inherit_from,
  });
}

/**
 * @brief 移除角色继承
 */
export function removeRoleInherit(role_name: string) {
  return httpClient.delete(`/admin/roles/${role_name}/inherit`);
}

// ============ 批量权限分配 ============

/**
 * @brief 批量为多个角色分配相同权限
 */
export function batchAssignPermissions(
  role_names: string[],
  permissions: string[],
  mode: 'add' | 'set' | 'remove'
) {
  return httpClient.post('/permissions/batch-assign', {
    roles: role_names,
    permissions,
    mode,
  });
}

/**
 * @brief 复制角色权限
 */
export function copyRolePermissions(
  sourceRole: string,
  targetRoles: string[],
  options?: {
    includeDataPermission?: boolean;
    includeFieldPermission?: boolean;
  }
) {
  return httpClient.post('/permissions/copy', {
    source_role: sourceRole,
    target_roles: targetRoles,
    include_data_permission: options?.includeDataPermission ?? true,
    include_field_permission: options?.includeFieldPermission ?? true,
  });
}

// ============ 权限变更日志 ============

/**
 * @brief 获取权限变更日志列表
 */
export function listPermissionChangeLogs(params?: {
  page?: number;
  page_size?: number;
  role_name?: string;
  operator?: string;
  change_type?: 'add' | 'remove' | 'update';
  start_date?: string;
  end_date?: string;
}) {
  return httpClient.get('/permission-change-logs', { params });
}

/**
 * @brief 获取权限变更详情
 */
export function getPermissionChangeLogDetail(logId: number) {
  return httpClient.get(`/permission-change-logs/${logId}`);
}

/**
 * @brief 导出权限变更日志
 */
export function exportPermissionChangeLogs(params?: {
  role_name?: string;
  start_date?: string;
  end_date?: string;
  format?: 'csv' | 'excel';
}) {
  return httpClient.get('/permission-change-logs/export', {
    params,
    responseType: 'blob',
  });
}

// ============ 权限验证 ============

/**
 * @brief 验证用户数据权限
 */
export function validateDataPermission(params: {
  permission: string;
  resource_type: string;
  resource_id: number | string;
}) {
  return httpClient.post('/permissions/validate-data', params);
}

/**
 * @brief 获取用户可访问的部门列表
 */
export function getAccessibleDepartments(role_name: string) {
  return httpClient.get(`/admin/roles/${role_name}/accessible-departments`);
}

/**
 * @brief 获取用户可访问的租户列表
 */
export function getAccessibleTenants(role_name: string) {
  return httpClient.get(`/admin/roles/${role_name}/accessible-tenants`);
}

// ============ 权限缓存管理 ============

/**
 * @brief 刷新角色权限缓存
 */
export function refreshPermissionCache(role_name?: string) {
  const params = role_name ? { role: role_name } : {};
  return httpClient.post('/permissions/refresh-cache', undefined, { params });
}

// ============ 权限预检 ============

/**
 * @brief 检查敏感权限操作是否需要二次确认
 */
export function checkSensitivePermission(permission: string) {
  return httpClient.post('/permissions/check-sensitive', { permission });
}