/**
 * @file role.ts
 * @brief 角色管理 API
 * @date 2026-05-21
 * @description 统一使用带泛型的 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse, PaginationParams, PaginatedResponse } from '@/types/api';
import type { User } from '@/types/user';
import type { RoleDetail, RoleCreateForm, RoleUpdateForm } from '../types/role';
import type { RoleListResponse } from '../types/user';
import { handleApiError } from '@/utils/apiErrorHandler';

// 为兼容旧代码保留 Role 别名（推荐使用 RoleDetail）
export type Role = RoleDetail;

export interface ListRolesParams extends PaginationParams {
  keyword?: string;
  type?: string;
  status?: number;
}

// 导出类型（从 types/user.ts 导入）
export type { RoleListResponse } from '../types/user';

/**
 * @brief 获取角色列表
 */
export function listRoles(params?: ListRolesParams) {
  try {
    return await httpClient.get('/roles', { params });
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 获取角色详情
 */
export function getRole(name: string) {
  try {
    return await httpClient.get(`/roles/${name}`);
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 创建角色
 */
export function createRole(data: RoleCreateForm) {
  try {
    return await httpClient.post('/roles', data);
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 更新角色
 */
export function updateRole(name: string, data: RoleUpdateForm) {
  try {
    return await httpClient.put(`/roles/${name}`, data);
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 删除角色
 */
export function deleteRole(name: string) {
  try {
    return await httpClient.delete(`/roles/${name}`);
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 获取角色权限
 */
export function getRolePermissions(name: string) {
  try {
    return await httpClient.get(`/roles/${name}/permissions`);
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 设置角色权限
 */
export function setRolePermissions(name: string, permissions: string[]) {
  try {
    return await httpClient.put(`/roles/${name}/permissions`, { permissions });
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}

/**
 * @brief 获取角色下的用户列表
 */
export function getRoleUsers(name: string, params?: PaginationParams) {
  try {
    return await httpClient.get(
    `/roles/${name}/users`,
    { params }  );
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
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
  try {
    return await httpClient.post('/roles/copy-permissions', {
    source_role: sourceRole,
    target_roles: targetRoles,
    include_data_permission: options?.includeDataPermission ?? true,
    include_field_permission: options?.includeFieldPermission ?? true,  });
  } catch (error) {
    handleApiError(error, '角色管理');
    throw error;
  }
}