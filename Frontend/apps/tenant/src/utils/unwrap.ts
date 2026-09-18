/**
 * @file unwrap.ts
 * @description 类型安全的 API 响应解包工具 — 替代 `as unknown as` 双重断言
 * @date 2026-09-19
 */

/** 标准 API 响应结构 */
interface ApiResponse<T = unknown> {
  code?: number;
  message?: string;
  success?: boolean;
  data?: T;
}

/** 分页 API 响应结构 */
interface PaginatedApiResponse<T = unknown> {
  code?: number;
  message?: string;
  data?: { list?: T[]; total?: number; page?: number; page_size?: number };
}

/**
 * 类型安全地提取 ApiResponse 中的 data 字段
 * 替代 `(response as unknown as { data?: T }).data` 模式
 */
export function unwrapData<T>(response: unknown): T | undefined {
  if (!response || typeof response !== 'object') return undefined;
  const r = response as ApiResponse<T>;
  return r.data;
}

/**
 * 类型安全地提取分页响应中的 list 字段
 */
export function unwrapList<T>(response: unknown): T[] {
  if (!response || typeof response !== 'object') return [];
  const r = response as PaginatedApiResponse<T>;
  return r.data?.list ?? [];
}

/**
 * 类型安全地提取分页响应中的 total 字段
 */
export function unwrapTotal(response: unknown): number {
  if (!response || typeof response !== 'object') return 0;
  const r = response as PaginatedApiResponse;
  return r.data?.total ?? 0;
}

/**
 * 完整解包分页响应 { list, total, page, page_size }
 */
export function unwrapPaginated<T>(response: unknown): { list: T[]; total: number; page?: number; page_size?: number } {
  if (!response || typeof response !== 'object') return { list: [], total: 0 };
  const r = response as PaginatedApiResponse<T>;
  return {
    list: r.data?.list ?? [],
    total: r.data?.total ?? 0,
    page: r.data?.page,
    page_size: r.data?.page_size,
  };
}
