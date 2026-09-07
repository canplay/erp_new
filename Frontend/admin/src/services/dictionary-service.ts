/**
 * @file dictionary-service.ts
 * @brief 字典服务（dictionary-service）
 * @date 2026-06-18
 * @description 对接 Backend/Rust dictionary-service
 */

import { httpClient } from '@/utils/alova';
import type { HttpResponse } from '@/utils/alova';

// ============ 类型定义 ============

/**
 * @brief 字典类型
 */
export interface DictionaryType {
  id: number;
  name: string;
  description?: string;
  status: number;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 字典数据
 */
export interface DictionaryData {
  id: number;
  type_id: number;
  type_name: string;
  label: string;
  value: string;
  sort: number;
  status: number;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 字典类型查询参数
 */
export interface DictionaryTypeQueryParams {
  page?: number;
  page_size?: number;
  keyword?: string;
  status?: number;
}

/**
 * @brief 字典数据查询参数
 */
export interface DictionaryDataQueryParams {
  type_id?: number;
  type_name?: string;
  status?: number;
  is_default?: boolean;
  page?: number;
  page_size?: number;
}

/**
 * @brief 字典类型创建参数
 */
export interface DictionaryTypeCreateParams {
  name: string;
  description?: string;
  status?: number;
}

/**
 * @brief 字典数据创建参数
 */
export interface DictionaryDataCreateParams {
  type_id: number;
  label: string;
  value: string;
  sort?: number;
  status?: number;
  is_default?: boolean;
}

// ============ 字典类型 API ============

/**
 * @brief 获取字典类型列表
 * @description GET /api/dictionary/types
 */
export function getDictionaryTypes(
  params?: DictionaryTypeQueryParams
): Promise<HttpResponse<DictionaryType[]>> {
  return httpClient.get<DictionaryType[]>('/dictionary/types', { params })
}

/**
 * @brief 获取所有字典类型（不分页）
 * @description GET /api/dictionary/types/all
 */
export function getAllDictionaryTypes(): Promise<HttpResponse<DictionaryType[]>> {
  return httpClient.get<DictionaryType[]>('/dictionary/types/all')
}

/**
 * @brief 获取单个字典类型
 * @description GET /api/dictionary/types/:id
 */
export function getDictionaryType(
  id: number
): Promise<HttpResponse<DictionaryType>> {
  return httpClient.get<DictionaryType>(`/dictionary/types/${id}`)
}

/**
 * @brief 创建字典类型
 * @description POST /api/dictionary/types
 */
export function createDictionaryType(
  data: DictionaryTypeCreateParams
): Promise<HttpResponse<DictionaryType>> {
  return httpClient.post<DictionaryType>('/dictionary/types', data)
}

/**
 * @brief 更新字典类型
 * @description PUT /api/dictionary/types/:id
 */
export function updateDictionaryType(
  id: number,
  data: Partial<DictionaryTypeCreateParams>
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/dictionary/types/${id}`, data)
}

/**
 * @brief 删除字典类型
 * @description DELETE /api/dictionary/types/:id
 */
export function deleteDictionaryType(id: number): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(`/dictionary/types/${id}`)
}

/**
 * @brief 批量删除字典类型
 * @description DELETE /api/dictionary/types/batch
 */
export function batchDeleteDictionaryTypes(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>('/dictionary/types/batch', { data: { ids } })
}

// ============ 字典数据 API ============

/**
 * @brief 获取字典数据列表
 * @description GET /api/dictionary/data
 */
export function getDictionaryData(
  params?: DictionaryDataQueryParams
): Promise<HttpResponse<DictionaryData[]>> {
  return httpClient.get<DictionaryData[]>('/dictionary/data', { params })
}

/**
 * @brief 获取指定类型的字典数据
 * @description GET /api/dictionary/data/by-type/:typeName
 */
export function getDictionaryDataByType(
  typeName: string
): Promise<HttpResponse<DictionaryData[]>> {
  return httpClient.get<DictionaryData[]>(`/dictionary/data/by-type/${typeName}`)
}

/**
 * @brief 获取单个字典数据
 * @description GET /api/dictionary/data/:id
 */
export function getDictionaryDataDetail(
  id: number
): Promise<HttpResponse<DictionaryData>> {
  return httpClient.get<DictionaryData>(`/dictionary/data/${id}`)
}

/**
 * @brief 创建字典数据
 * @description POST /api/dictionary/data
 */
export function createDictionaryData(
  data: DictionaryDataCreateParams
): Promise<HttpResponse<DictionaryData>> {
  return httpClient.post<DictionaryData>('/dictionary/data', data)
}

/**
 * @brief 更新字典数据
 * @description PUT /api/dictionary/data/:id
 */
export function updateDictionaryData(
  id: number,
  data: Partial<DictionaryDataCreateParams>
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/dictionary/data/${id}`, data)
}

/**
 * @brief 删除字典数据
 * @description DELETE /api/dictionary/data/:id
 */
export function deleteDictionaryData(id: number): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(`/dictionary/data/${id}`)
}

/**
 * @brief 批量删除字典数据
 * @description DELETE /api/dictionary/data/batch
 */
export function batchDeleteDictionaryData(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>('/dictionary/data/batch', { data: { ids } })
}

/**
 * @brief 批量更新字典数据排序
 * @description PUT /api/dictionary/data/reorder
 */
export function reorderDictionaryData(
  orders: Array<{ id: number; sort: number }>
): Promise<HttpResponse<void>> {
  return httpClient.put<void>('/dictionary/data/reorder', { orders })
}

// ============ 字典缓存 API ============

/**
 * @brief 刷新字典缓存
 * @description POST /api/dictionary/cache/refresh
 */
export function refreshDictionaryCache(
  typeName?: string
): Promise<HttpResponse<void>> {
  const params = typeName ? { type: typeName } : {};
  return httpClient.post<void>('/dictionary/cache/refresh', undefined, { params })
}

/**
 * @brief 清除字典缓存
 * @description POST /api/dictionary/cache/clear
 */
export function clearDictionaryCache(): Promise<HttpResponse<void>> {
  return httpClient.post<void>('/dictionary/cache/clear')
}

/**
 * @brief 获取字典缓存状态
 * @description GET /api/dictionary/cache/status
 */
export function getDictionaryCacheStatus(): Promise<HttpResponse<{
  cache_size: number;
  last_refresh: string;
  is_cached: boolean;
}>> {
  return httpClient.get<{
    cache_size: number;
    last_refresh: string;
    is_cached: boolean;
  }>('/dictionary/cache/status');
}

// ============ 导出 ============

export const dictionaryService = {
  // 字典类型
  getDictionaryTypes,
  getAllDictionaryTypes,
  getDictionaryType,
  createDictionaryType,
  updateDictionaryType,
  deleteDictionaryType,
  batchDeleteDictionaryTypes,

  // 字典数据
  getDictionaryData,
  getDictionaryDataByType,
  getDictionaryDataDetail,
  createDictionaryData,
  updateDictionaryData,
  deleteDictionaryData,
  batchDeleteDictionaryData,
  reorderDictionaryData,

  // 字典缓存
  refreshDictionaryCache,
  clearDictionaryCache,
  getDictionaryCacheStatus,
};
