/**
 * @file index.ts
 * @description 共享 API 客户端和工具函数
 */

export { getAlova } from '@erp-new-frontend-monorepo/boot/alova';

// ==================== 业务 API 导出 ====================

export {
  deviceApi,
  dailyApi,
  weeklyApi,
  monthlyApi,
  dangerApi,
  personnelApi,
  currentUserApi,
  identityApi,
  notificationsApi,
  forgotPasswordApi,
  filesApi,
} from '@erp-new-frontend-monorepo/api-domain';

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
