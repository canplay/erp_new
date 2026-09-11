/**
 * @file department.ts
 * @brief 部门管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse, PaginationParams, PaginationResponse } from '@/utils/alova';
import type { User } from './user';

export interface Department {
  id: number;
  name: string;
  parent_id?: number;
  parent_name?: string;
  leader_id?: number;
  leader_name?: string;
  description?: string;
  sort_order?: number;
  user_count?: number;
  children?: Department[];
  created_at?: number;
  updated_at?: number;
}

export interface CreateDepartmentRequest {
  name: string;
  parent_id?: number;
  leader_id?: number;
  description?: string;
  sort_order?: number;
}

export interface UpdateDepartmentRequest {
  name?: string;
  parent_id?: number;
  leader_id?: number;
  description?: string;
  sort_order?: number;
}

export interface ListDepartmentsParams extends PaginationParams {
  keyword?: string;
  parent_id?: number;
}

/**
 * @brief 获取部门列表（树形）
 */
export function listDepartments(params?: ListDepartmentsParams) {
  return httpClient.get<ApiResponse<Department[]>>('/admin/departments', { params });
}

/**
 * @brief 获取部门详情
 */
export function getDepartment(id: number) {
  return httpClient.get<ApiResponse<Department>>(`/admin/departments/${id}`);
}

/**
 * @brief 创建部门
 */
export function createDepartment(data: CreateDepartmentRequest) {
  return httpClient.post<ApiResponse<Department>>('/admin/departments', data);
}

/**
 * @brief 更新部门
 */
export function updateDepartment(id: number, data: UpdateDepartmentRequest) {
  return httpClient.put<ApiResponse<void>>(`/admin/departments/${id}`, data);
}

/**
 * @brief 删除部门
 */
export function deleteDepartment(id: number) {
  return httpClient.delete<ApiResponse<void>>(`/admin/departments/${id}`);
}

/**
 * @brief 获取部门下的用户列表
 */
export function getDepartmentUsers(id: number, params?: PaginationParams) {
  return httpClient.get<ApiResponse<PaginationResponse<User>>>(
    `/admin/departments/${id}/users`,
    { params }
  );
}

/**
 * @brief 移动部门
 */
export function moveDepartment(id: number, newParentId?: number) {
  return httpClient.put<ApiResponse<void>>(`/admin/departments/${id}/move`, { parent_id: newParentId });
}

/**
 * @brief 获取部门树形结构
 */
export function getDepartmentTree() {
  return httpClient.get<ApiResponse<Department[]>>('/admin/departments/tree');
}