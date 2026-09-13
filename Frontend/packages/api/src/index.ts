/**
 * @file index.ts
 * @description 共享 API 客户端和工具函数
 *
 * 提供统一的 alova HTTP 客户端、响应归一化、类型导出
 * 各业务 API 子模块从 apps/admin/src/api/ 和 apps/tenant/src/api/ 导入
 */

// Re-export alova client factory from boot
export { getAlova, setupAlova, setErrorNotifier, isAuthenticated } from '@erp-new-frontend-monorepo/boot/alova';

// ==================== HTTP 客户端 ====================

interface AlovaClient {
  Get<T = unknown>(url: string, config?: unknown): Promise<T>;
  Post<T = unknown>(url: string, data?: unknown, config?: unknown): Promise<T>;
  Put<T = unknown>(url: string, data?: unknown, config?: unknown): Promise<T>;
  Patch<T = unknown>(url: string, data?: unknown, config?: unknown): Promise<T>;
  Delete<T = unknown>(url: string, config?: unknown): Promise<T>;
}

class M {
  private get a(): AlovaClient { return getAlova() as unknown as AlovaClient; }
  Get<T = unknown>(u: string, params?: Record<string, string | number | boolean>) {
    return this.a.Get<T>(u, params ? { params } : undefined);
  }
  Post<T = unknown>(u: string, d?: unknown) {
    return this.a.Post<T>(u, d);
  }
  Put<T = unknown>(u: string, d?: unknown) {
    return this.a.Put<T>(u, d);
  }
  Patch<T = unknown>(u: string, d?: unknown) {
    return this.a.Patch<T>(u, d);
  }
  Delete<T = unknown>(u: string) {
    return this.a.Delete<T>(u);
  }
}
export const api = new M();

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

// ==================== 共享类型 ====================

export interface CurrentUserDto {
  id: string;
  userName: string;
  email: string;
  fullName?: string;
  roleNames?: string[];
  tenantId?: string;
  tenant?: string;
}

export interface IdentityUser {
  id: string;
  userName: string;
  firstName?: string;
  lastName?: string;
  email: string;
  isActive: boolean;
  emailConfirmed?: boolean;
  phoneNumber?: string;
  imageUrl?: string;
  twoFactorEnabled?: boolean;
  tenantId?: string;
  tenant?: string;
}

export interface IdentityRole {
  id: string;
  name: string;
  description?: string;
  permissions?: string[];
}

export interface PermissionCatalogEntry {
  name: string;
  description: string;
  resource: string;
  action: string;
  isBasic: boolean;
  isRoot: boolean;
}

// ==================== 业务 API 子模块导出 ====================

// 从 admin 平台视角导出的业务 API（保持向后兼容）
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
  auditsApi,
  knowledgeGraphApi,
  searchApi,
  analyticsApi,
} from '@erp-new-frontend-monorepo/api-domain';

// 导出 API 请求/响应类型
export type {
  AuditSummaryDto,
  AuditDetailDto,
  AuditSummaryAggregateDto,
} from '@erp-new-frontend-monorepo/api-domain';
