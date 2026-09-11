/**
 * @file user.ts
 * @description 用户管理 API
 * @date 2026-05-21
 * @description 统一使用带泛型的 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { UserQueryParams, UserCreateForm, UserUpdateForm, BatchUpdateRoleParams, BatchUpdateStatusParams, UserImportResult } from '@/types/user';
import type { ApiResponse } from '@/types/api';

// ============ 用户信息相关 ============

/**
 * @brief 获取当前用户信息
 * @returns { id, username, nickname, avatar, phone, email, gender, address, role, status, created_at, updated_at }
 */
export function getUserInfo() {
  return httpClient.get('/user/info');
}

/**
 * @brief 更新用户资料
 * @param data.nickname - 昵称（可选）
 * @param data.avatar - 头像URL（可选）
 * @param data.phone - 手机号（可选）
 * @param data.email - 邮箱（可选）
 */
export function updateUserInfo(data: {
  nickname?: string;
  avatar?: string;
  phone?: string;
  email?: string;
}) {
  return httpClient.put('/user/info', data);
}

/**
 * @brief 修改密码
 * @param data.old_password - 旧密码
 * @param data.new_password - 新密码
 */
export function changePassword(data: { old_password: string; new_password: string }) {
  return httpClient.put('/user/password', {
    old_password: data.old_password,
    new_password: data.new_password,
  });
}

/**
 * @brief 更新头像
 */
export function updateAvatar(avatar: string) {
  return httpClient.put('/user/avatar', { avatar });
}

// ============ 管理员接口 - 用户管理 ============

/**
 * @brief 获取用户列表（分页）
 */
export function listUsers(params?: UserQueryParams) {
  return httpClient.get('/admin/users', { params });
}

/**
 * @brief 获取单个用户详情
 */
export function getUser(id: number) {
  return httpClient.get(`/admin/users/${id}`);
}

/**
 * @brief 创建用户
 */
export function createUser(data: UserCreateForm) {
  return httpClient.post('/admin/users', data);
}

/**
 * @brief 更新用户
 */
export function updateUser(id: number, data: UserUpdateForm) {
  return httpClient.put(`/admin/users/${id}`, data);
}

/**
 * @brief 更新用户状态（启用/禁用/锁定）
 * @param status - 状态值: 0=禁用, 1=启用, 2=锁定
 * @param lockHours - 锁定时长（小时），仅 status=2 时有效
 */
export function updateUserStatus(id: number, status: number, lockHours?: number) {
  const data: Record<string, unknown> = { status };
  if (lockHours !== undefined) {
    data.lock_hours = lockHours;
  }
  return httpClient.put(`/admin/users/${id}/status`, data);
}

/**
 * @brief 更新用户角色
 * @param role - 角色: user, admin, vip
 */
export function updateUserRole(id: number, role: string) {
  return httpClient.put(`/admin/users/${id}/role`, { role });
}

/**
 * @brief 重置用户密码
 */
export function resetUserPassword(id: number, password?: string) {
  return httpClient.post(`/admin/users/${id}/reset-password`, {
    password: password || '',
  });
}

/**
 * @brief 删除用户
 */
export function deleteUser(id: number) {
  return httpClient.delete(`/admin/users/${id}`);
}

// ============ 批量用户操作 ============

/**
 * @brief 批量更新用户角色
 */
export function batchUpdateUserRole(params: BatchUpdateRoleParams) {
  return httpClient.put('/admin/users/batch-role', {
    user_ids: params.user_ids,
    role: params.role,
  });
}

/**
 * @brief 批量更新用户状态
 */
export function batchUpdateUserStatus(params: BatchUpdateStatusParams) {
  return httpClient.put('/admin/users/batch-status', {
    user_ids: params.user_ids,
    status: params.status,
  });
}

/**
 * @brief 批量删除用户
 */
export function batchDeleteUsers(user_ids: number[]) {
  return httpClient.post('/admin/users/batch-delete', {
    user_ids: user_ids,
  });
}

// ============ 用户导入导出 ============

/**
 * @brief 获取用户导入模板
 */
export function getUserImportTemplate() {
  return httpClient.get('/admin/users/import-template');
}

/**
 * @brief 导入用户
 */
export function importUsers(file: File) {
  const formData = new FormData();
  formData.append('file', file);
  return httpClient.post<ApiResponse<UserImportResult>>('/admin/users/import', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
}

/**
 * @brief 导出用户
 */
export function exportUsers(params?: {
  keyword?: string;
  status?: number;
  role?: string;
  format?: 'csv' | 'excel';
}) {
  return httpClient.get('/admin/users/export', {
    params,
    responseType: 'blob',
  });
}

/**
 * @brief 下载用户导入模板
 */
export function downloadUserImportTemplate() {
  return httpClient.get('/admin/users/import-template/download', {
    responseType: 'blob',
  });
}

// ============ 部门管理（委托给 department.ts） ============

// 从 department.ts 重新导出，避免破坏现有导入
export type { Department } from '@/types/user';
export type {
  DepartmentCreateParams,
  DepartmentUpdateParams,
  DepartmentQueryParams,
} from '@/types/user';

// 重新导出部门 API 函数
export {
  listDepartments,
  getDepartmentTree,
  createDepartment,
  updateDepartment,
  deleteDepartment,
} from './department';

// ============ 日志管理（委托给 log.ts） ============

// 重新导出登录日志函数
export { listLoginLogs } from './log';

// ============ 管理员接口 - 统计 ============

/**
 * @brief 获取系统统计信息
 */
export function getStatistics() {
  return httpClient.get('/admin/stats');
}

// ============ 类型导出 ============

/**
 * @brief 用户类型（从 types/user.ts 重新导出）
 */
export type { User, UserInfo as UserInfoType } from '@/types/user';

/**
 * @brief 角色类型（从 types/user.ts 重新导出）
 */
export type { Role } from '@/types/user';

/**
 * @brief 系统统计类型（从 types/user.ts 重新导出）
 */
export type { Statistics } from '@/types/user';

/**
 * @brief 登录日志类型（从 types/log.ts 重新导出）
 */
export type { LoginLog } from '@/types/log';

/**
 * @brief 用户状态枚举
 */
export enum UserStatus {
  DISABLED = 0, // 禁用
  ENABLED = 1, // 启用
  LOCKED = 2, // 锁定
}

/**
 * @brief 角色类型枚举
 */
export enum RoleType {
  USER = 'user',
  ADMIN = 'admin',
  VIP = 'vip',
}