/**
 * @file dictionary.ts
 * @description 数据字典 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ 类型定义 ============

/**
 * @brief 字典类型
 */
export interface DictionaryType {
  id: number;
  code: string;
  name: string;
  description?: string;
  sort: number;
  status: number;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 字典项
 */
export interface DictionaryItem {
  id: number;
  type_id: number;
  label: string;
  value: string;
  sort: number;
  status: number;
  is_default?: boolean;
  remark?: string;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 字典类型（含子项）
 */
export interface DictionaryTypeWithItems extends DictionaryType {
  items?: DictionaryItem[];
}

// ============ 字典类型 API ============

/**
 * @brief 获取字典类型列表
 */
export async function listDictionaryTypes(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  status?: number;
}) {
  try {
    return await httpClient.get('/admin/dictionary/types', { params });
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 获取单个字典类型
 */
export async function getDictionaryType(id: number) {
  try {
    return await httpClient.get(`/admin/dictionary/types/${id}`);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 创建字典类型
 */
export async function createDictionaryType(data: {
  code: string;
  name: string;
  description?: string;
  sort?: number;
}) {
  try {
    return await httpClient.post('/admin/dictionary/types', data);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 更新字典类型
 */
export async function updateDictionaryType(
  id: number,
  data: {
    name?: string;
    description?: string;
    sort?: number;
    status?: number;
  }
) {
  try {
    return await httpClient.put(`/admin/dictionary/types/${id}`, data);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 删除字典类型
 */
export async function deleteDictionaryType(id: number) {
  try {
    return await httpClient.delete(`/admin/dictionary/types/${id}`);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 批量删除字典类型
 */
export async function batchDeleteDictionaryTypes(ids: number[]) {
  try {
    return await httpClient.post('/admin/dictionary/types/batch-delete', { ids });
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

// ============ 字典项 API ============

/**
 * @brief 获取字典项列表
 */
export async function listDictionaryItems(params?: {
  type_id?: number;
  keyword?: string;
  status?: number;
}) {
  try {
    return await httpClient.get('/admin/dictionary/items', { params });
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 获取字典类型下的所有字典项
 */
export async function getDictionaryItemsByType(typeCode: string) {
  try {
    return await httpClient.get(`/admin/dictionary/types/${typeCode}/items`);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 获取单个字典项
 */
export async function getDictionaryItem(id: number) {
  try {
    return await httpClient.get(`/admin/dictionary/items/${id}`);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 创建字典项
 */
export async function createDictionaryItem(data: {
  type_id: number;
  label: string;
  value: string;
  sort?: number;
  status?: number;
  is_default?: boolean;
  remark?: string;
}) {
  try {
    return await httpClient.post('/admin/dictionary/items', data);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 更新字典项
 */
export async function updateDictionaryItem(
  id: number,
  data: {
    label?: string;
    value?: string;
    sort?: number;
    status?: number;
    is_default?: boolean;
    remark?: string;
  }
) {
  try {
    return await httpClient.put(`/admin/dictionary/items/${id}`, data);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 删除字典项
 */
export async function deleteDictionaryItem(id: number) {
  try {
    return await httpClient.delete(`/admin/dictionary/items/${id}`);
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 批量删除字典项
 */
export async function batchDeleteDictionaryItems(ids: number[]) {
  try {
    return await httpClient.post('/admin/dictionary/items/batch-delete', { ids });
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 批量创建字典项
 */
export async function batchCreateDictionaryItems(type_id: number, items: Array<{
  label: string;
  value: string;
  sort?: number;
  is_default?: boolean;
  remark?: string;
}>) {
  try {
    return await httpClient.post(`/admin/dictionary/types/${type_id}/items/batch`, { items });
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 调整字典项顺序
 */
export async function reorderDictionaryItems(type_id: number, itemIds: number[]) {
  try {
    return await httpClient.put(`/admin/dictionary/types/${type_id}/items/reorder`, { itemIds });
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}

/**
 * @brief 获取所有启用的字典（用于全局缓存）
 */
export async function getAllEnabledDictionaries() {
  try {
    return await httpClient.get('/admin/dictionary/all-enabled');
  } catch (error) {
    handleApiError(error, '字典管理');
    throw error;
  }
}