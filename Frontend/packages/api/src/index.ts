/**
 * @file index.ts
 * @description 共享 API 客户端和工具函数
 */

export { getAlova } from '@erp-new-frontend-monorepo/boot';

// ==================== 响应归一化工具 ====================

export interface PagedResult<T> {
  total: number;
  items: T[];
  page: number;
  size: number;
}

/** 归一化后端分页响应（兼容 totalCount/total/数组三种形态） */
export function normalize<T>(r: unknown, page = 1, size = 20): PagedResult<T> {
  const payload = r as { totalCount?: number; total?: number; items?: T[] } | null | undefined;
  return {
    total: payload?.totalCount ?? (Array.isArray(r) ? (r as T[]).length : (payload?.total ?? 0)),
    items: Array.isArray(r) ? (r as T[]) : (payload?.items ?? []),
    page,
    size,
  };
}

/** 兼容「数组 / { items }」两种返回形态 */
export function unwrapArray<T>(r: unknown): T[] {
  return Array.isArray(r) ? (r as T[]) : ((r as { items?: T[] } | null | undefined)?.items ?? []);
}

/** 创建 alova API 方法（用于快速定义 GET/POST 端点） */
export function api<T = unknown>(url: string, method: 'GET' | 'POST' | 'PUT' | 'DELETE' = 'GET', body?: unknown) {
  const alova = getAlova();
  const alovaMethod = method === 'GET'
    ? alova.Get<T>(url)
    : method === 'POST'
    ? alova.Post<T>(url, body)
    : method === 'PUT'
    ? alova.Put<T>(url, body)
    : alova.Delete<T>(url, body);
  return alovaMethod;
}

// ==================== 类型导出 ====================

export type { ApiResponse, PaginatedResponse, ApiError } from '@erp-new-frontend-monorepo/types';
