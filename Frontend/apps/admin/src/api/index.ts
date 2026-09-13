// admin 业务 API 封装
// 共享 API 来自 @erp-new-frontend-monorepo/api；本文件保留 admin 平台视角独有 API
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
} from '@erp-new-frontend-monorepo/api';
export type {
  CurrentUserDto,
  IdentityUser,
  IdentityRole,
  PermissionCatalogEntry,
} from '@erp-new-frontend-monorepo/api';
export { getAlova } from '@erp-new-frontend-monorepo/boot/alova';
export {
  auditsApi,
  type AuditSummaryDto,
  type AuditDetailDto,
  type AuditSummaryAggregateDto,
} from '@erp-new-frontend-monorepo/api';

// admin 平台视角特有 API（按领域子模块）
export {
  regulatoryApi,
  type RegulatoryReceivePayload,
  type RegulatoryFeedbackPayload,
  type RegulatoryReceiveRecordItem,
  type RegulatoryFeedbackRecordItem,
  type RegulatoryExportRecordItem,
  type RegulatoryExportRecordsResult,
} from './regulatory';
export { impersonationApi } from './identity';
export {
  tenantsApi,
  billingApi,
  sessionsApi,
  webhooksApi,
  healthApi,
  type HealthResult,
  type HealthEntry,
} from './platform';
export {
  catalogApi,
  type BrandDto,
  type CategoryDto,
  type ProductDto,
  type CreateProductDto,
  type UpdateProductDto,
} from './catalog';
export {
  ticketsApi,
  type TicketDto,
  type CommentDto,
  type CreateTicketDto,
  type UpdateTicketDto,
} from './tickets';

// 复用共享 alova 封装（packages/api），不再重复定义 class M
import { api, normalize, unwrapArray } from '@erp-new-frontend-monorepo/api';

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

// admin 平台视角特有类型（原 index.ts 中定义，保持兼容）
export interface AdminNotification {
  id: string;
  type: string;
  title: string;
  body?: string | null;
  link?: string | null;
  source: string;
  metadataJson: string;
  readAtUtc?: string | null;
  createdAtUtc: string;
}

export interface AdminRole {
  id: string;
  name: string;
  description?: string;
  permissions?: string[];
}
