// 复用共享 identityApi（packages/api/src/identity.ts）
export { identityApi } from '@erp-new-frontend-monorepo/api';
import { api, normalize, identityApi } from '@erp-new-frontend-monorepo/api';

export interface GrantDto {
  id: string;
  actorUserId: string;
  actorName: string;
  impersonatedUserId: string;
  impersonatedUserName: string;
  reason: string;
  startAt: string;
  endAt?: string | null;
  status: string;
  createdAt: string;
}

// @viewpoint platform-only（模拟登录仅平台）
export const impersonationApi = {
  grants: (page = 1, size = 20) =>
    api
      .Get<GrantDto[]>('/api/v1/identity/impersonation/grants', { page, size })
      .then((r) => normalize<GrantDto>(r, page, size)),
  start: (userId: string) => identityApi.impersonate(userId),
  end: () => identityApi.endImpersonation(),
  revoke: (id: string) => identityApi.revokeImpersonation(id),
};
