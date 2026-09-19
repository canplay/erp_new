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

// ==================== alova API 方法工厂 ====================

function createApiMethod<T = unknown>(url: string, method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH', body?: unknown) {
  const alova = getAlova();
  switch (method) {
    case 'GET':
      return alova.Get<T>(url, { params: body as Record<string, unknown> });
    case 'POST':
      return alova.Post<T>(url, body);
    case 'PUT':
      return alova.Put<T>(url, body);
    case 'PATCH':
      return alova.Patch<T>(url, body);
    case 'DELETE':
      return alova.Delete<T>(url, body);
    default:
      return alova.Get<T>(url);
  }
}

interface ApiFunction {
  <T = unknown>(url: string, method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH', body?: unknown): Promise<T>;
  Get<T = unknown>(url: string, params?: Record<string, unknown>): Promise<T>;
  Post<T = unknown>(url: string, body?: unknown): Promise<T>;
  Put<T = unknown>(url: string, body?: unknown): Promise<T>;
  Patch<T = unknown>(url: string, body?: unknown): Promise<T>;
  Delete<T = unknown>(url: string, params?: Record<string, unknown>): Promise<T>;
}

export const api: ApiFunction = Object.assign(
  function apiFn<T = unknown>(url: string, method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' = 'GET', body?: unknown) {
    return createApiMethod<T>(url, method, body);
  },
  {
    Get: <T = unknown>(url: string, params?: Record<string, unknown>) =>
      createApiMethod<T>(url, 'GET', params),
    Post: <T = unknown>(url: string, body?: unknown) =>
      createApiMethod<T>(url, 'POST', body),
    Put: <T = unknown>(url: string, body?: unknown) =>
      createApiMethod<T>(url, 'PUT', body),
    Patch: <T = unknown>(url: string, body?: unknown) =>
      createApiMethod<T>(url, 'PATCH', body),
    Delete: <T = unknown>(url: string, params?: Record<string, unknown>) =>
      createApiMethod<T>(url, 'DELETE', params),
  }
);

// ==================== Identity API ====================

export { identityApi, impersonationApi } from './identity';
export type { GrantDto, IdentityUser, IdentityRole, PermissionCatalogEntry } from './identity';

// ==================== Platform API ====================

export {
  tenantsApi,
  billingApi,
  sessionsApi,
  webhooksApi,
  healthApi,
} from './platform';
export type {
  TenantDto,
  TenantStatusDto,
  ProvisioningDto,
  ThemeDto,
  ThemeUpdateDto,
  BillingPlanDto,
  SubscriptionDto,
  InvoiceDto,
  InvoiceDetailDto,
  UsageDto,
  GenerateInvoiceDto,
  CreatePlanDto,
  UpdatePlanDto,
  CreateSubscriptionDto,
  SessionDto,
  WebhookSubscriptionDto,
  WebhookDeliveryDto,
  HealthEntry,
  HealthResult,
} from './platform';

// ==================== Catalog API ====================

export { catalogApi } from './catalog';
export type {
  BrandDto,
  CategoryDto,
  MoneyDto,
  ProductDto,
  CreateProductDto,
  UpdateProductDto,
} from './catalog';

// ==================== Tickets API ====================

export { ticketsApi } from './tickets';
export type {
  TicketDto,
  CommentDto,
  CreateTicketDto,
  UpdateTicketDto,
} from './tickets';

// ==================== Regulatory API ====================

export { regulatoryApi } from './regulatory';
export type {
  RegulatoryReceivePayload,
  RegulatoryFeedbackPayload,
  RegulatoryReceiveRecordItem,
  RegulatoryFeedbackRecordItem,
  RegulatoryExportRecordItem,
  RegulatoryExportRecordsResult,
  RegulatoryReceiveRecordsResponse,
} from './regulatory';

// ==================== Groups API ====================

export interface GroupItem {
  id: string;
  name: string;
  description?: string | null;
  isDefault: boolean;
  isSystemGroup: boolean;
  memberCount: number;
  roleIds?: string[] | null;
  roleNames?: string[] | null;
  createdAt: string;
}

export interface GroupMemberDto {
  id: string;
  userId: string;
  userName: string;
  email: string;
  fullName: string;
  roleNames: string[];
  joinedAt: string;
}

export interface CreateGroupResponse {
  id: string;
  name: string;
  description?: string | null;
  isDefault: boolean;
  isSystemGroup: boolean;
  memberCount: number;
  roleIds?: string[] | null;
  createdAt: string;
}

export interface DeleteResponse {
  success: boolean;
}

export interface AddMemberResponse {
  groupId: string;
  userId: string;
  joinedAt: string;
}

export interface RemoveMemberResponse {
  success: boolean;
}

export const groupsApi = {
  list: (page = 1, size = 20) =>
    api
      .Get<GroupItem[]>('/api/v1/identity/groups', { page, size })
      .then((r) => normalize<GroupItem>(r, page, size)),
  get: (id: string) => api.Get<GroupItem>(`/api/v1/identity/groups/${id}`),
  members: (groupId: string) =>
    api
      .Get<GroupMemberDto[]>(`/api/v1/identity/groups/${groupId}/members`)
      .then((r) => unwrapArray<GroupMemberDto>(r)),
  create: (data: { name: string; description?: string }) =>
    api.Post<CreateGroupResponse>('/api/v1/identity/groups', data),
  remove: (id: string) => api.Delete<DeleteResponse>(`/api/v1/identity/groups/${id}`),
  addMember: (groupId: string, userId: string) =>
    api.Post<AddMemberResponse>(`/api/v1/identity/groups/${groupId}/members`, { userId }),
  removeMember: (groupId: string, userId: string) =>
    api.Delete<RemoveMemberResponse>(`/api/v1/identity/groups/${groupId}/members/${userId}`),
};

// ==================== Extended APIs (currentUser, files, notifications, etc.) ====================

export {
  currentUserApi,
  forgotPasswordApi,
  filesApi,
  notificationsApi,
  auditsApi,
  analyticsApi,
  weeklyApi,
  monthlyApi,
  searchApi,
  knowledgeGraphApi,
  personnelApi,
  dailyApi,
  deviceApi,
  dangerApi,
} from './extended';

export type {
  CurrentUserDto,
  FileDto,
  NotificationDto,
  AuditSummaryDto,
  AuditDetailDto,
  AuditSummaryAggregateDto,
  ComplianceAssessmentDto,
  IssueAnalysisDto,
  WeeklyPlanDto,
  SearchResultDto,
  KnowledgeNodeDto,
  KnowledgeEdgeDto,
  PersonnelDto,
  DailyTaskDto,
  Device,
  DeviceInspectionDto,
  DeviceRepairDto,
  DeviceSparePartDto,
  HiddenDanger,
} from './extended';

// ==================== 类型导出 ====================

export type { ApiResponse, PaginatedResponse, ApiError } from '@erp-new-frontend-monorepo/types';
