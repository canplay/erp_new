/**
 * @file department.ts
 * @brief 部门管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse, PaginationParams, PaginationResponse } from '@/utils/alova';
import type { User } from './user';
import { handleApiError } from '@/utils/apiErrorHandler';

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
export async function listDepartments(params?: ListDepartmentsParams) {
  try {
    return await httpClient.get('/admin/departments', { params });
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 获取部门详情
 */
export async function getDepartment(id: number) {
  try {
    return await httpClient.get(`/admin/departments/${id}`);
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 创建部门
 */
export async function createDepartment(data: CreateDepartmentRequest) {
  try {
    return await httpClient.post('/admin/departments', data);
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 更新部门
 */
export async function updateDepartment(id: number, data: UpdateDepartmentRequest) {
  try {
    return await httpClient.put(`/admin/departments/${id}`, data);
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 删除部门
 */
export async function deleteDepartment(id: number) {
  try {
    return await httpClient.delete(`/admin/departments/${id}`);
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 获取部门下的用户列表
 */
export async function getDepartmentUsers(id: number, params?: PaginationParams) {
  try {
    return await httpClient.get(
    `/admin/departments/${id}/users`,
    { params }  );
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 移动部门
 */
export async function moveDepartment(id: number, newParentId?: number) {
  try {
    return await httpClient.put(`/admin/departments/${id}/move`, { parent_id: newParentId });
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}

/**
 * @brief 获取部门树形结构
 */
export async function getDepartmentTree() {
  try {
    return await httpClient.get('/admin/departments/tree');
  } catch (error) {
    handleApiError(error, '部门管理');
    throw error;
  }
}