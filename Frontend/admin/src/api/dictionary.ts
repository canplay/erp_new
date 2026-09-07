/**
 * @file dictionary.ts
 * @description 数据字典 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

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
export function listDictionaryTypes(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  status?: number;
}) {
  return httpClient.get('/admin/dictionary/types', { params });
}

/**
 * @brief 获取单个字典类型
 */
export function getDictionaryType(id: number) {
  return httpClient.get(`/admin/dictionary/types/${id}`);
}

/**
 * @brief 创建字典类型
 */
export function createDictionaryType(data: {
  code: string;
  name: string;
  description?: string;
  sort?: number;
}) {
  return httpClient.post('/admin/dictionary/types', data);
}

/**
 * @brief 更新字典类型
 */
export function updateDictionaryType(
  id: number,
  data: {
    name?: string;
    description?: string;
    sort?: number;
    status?: number;
  }
) {
  return httpClient.put(`/admin/dictionary/types/${id}`, data);
}

/**
 * @brief 删除字典类型
 */
export function deleteDictionaryType(id: number) {
  return httpClient.delete(`/admin/dictionary/types/${id}`);
}

/**
 * @brief 批量删除字典类型
 */
export function batchDeleteDictionaryTypes(ids: number[]) {
  return httpClient.post('/admin/dictionary/types/batch-delete', { ids });
}

// ============ 字典项 API ============

/**
 * @brief 获取字典项列表
 */
export function listDictionaryItems(params?: {
  type_id?: number;
  keyword?: string;
  status?: number;
}) {
  return httpClient.get('/admin/dictionary/items', { params });
}

/**
 * @brief 获取字典类型下的所有字典项
 */
export function getDictionaryItemsByType(typeCode: string) {
  return httpClient.get(`/admin/dictionary/types/${typeCode}/items`);
}

/**
 * @brief 获取单个字典项
 */
export function getDictionaryItem(id: number) {
  return httpClient.get(`/admin/dictionary/items/${id}`);
}

/**
 * @brief 创建字典项
 */
export function createDictionaryItem(data: {
  type_id: number;
  label: string;
  value: string;
  sort?: number;
  status?: number;
  is_default?: boolean;
  remark?: string;
}) {
  return httpClient.post('/admin/dictionary/items', data);
}

/**
 * @brief 更新字典项
 */
export function updateDictionaryItem(
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
  return httpClient.put(`/admin/dictionary/items/${id}`, data);
}

/**
 * @brief 删除字典项
 */
export function deleteDictionaryItem(id: number) {
  return httpClient.delete(`/admin/dictionary/items/${id}`);
}

/**
 * @brief 批量删除字典项
 */
export function batchDeleteDictionaryItems(ids: number[]) {
  return httpClient.post('/admin/dictionary/items/batch-delete', { ids });
}

/**
 * @brief 批量创建字典项
 */
export function batchCreateDictionaryItems(type_id: number, items: Array<{
  label: string;
  value: string;
  sort?: number;
  is_default?: boolean;
  remark?: string;
}>) {
  return httpClient.post(`/admin/dictionary/types/${type_id}/items/batch`, { items });
}

/**
 * @brief 调整字典项顺序
 */
export function reorderDictionaryItems(type_id: number, itemIds: number[]) {
  return httpClient.put(`/admin/dictionary/types/${type_id}/items/reorder`, { itemIds });
}

/**
 * @brief 获取所有启用的字典（用于全局缓存）
 */
export function getAllEnabledDictionaries() {
  return httpClient.get('/admin/dictionary/all-enabled');
}