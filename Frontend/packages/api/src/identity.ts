// 共享 identityApi（packages/api/src/identity.ts）
import { api, normalize, unwrapArray } from '@erp-new-frontend-monorepo/api';

export interface IdentityUser {
  id: string;
  userName: string;
  email: string;
  fullName?: string;
  isActive: boolean;
  createdAtUtc?: string;
}

export interface IdentityRole {
  id: string;
  name: string;
  description?: string;
  permissions?: string[];
}

export interface PermissionCatalogEntry {
  resource: string;
  action: string;
  description?: string;
}

export const identityApi = {
  users: (page = 1, size = 20) =>
    api
      .Get<IdentityUser[]>('/api/v1/identity/users', { page, size })
      .then((r) => normalize<IdentityUser>(r, page, size)),
  user: (id: string) => api.Get<IdentityUser>(`/api/v1/identity/users/${id}`),
  userRoles: (id: string) =>
    api.Get<IdentityRole[]>(`/api/v1/identity/users/${id}/roles`),
  toggleUserStatus: (id: string, isActive: boolean) =>
    api.Post(`/api/v1/identity/users/${id}/status`, { isActive }),
  assignUserRoles: (id: string, roleIds: string[]) =>
    api.Post(`/api/v1/identity/users/${id}/roles`, { roleIds }),
  userSessions: (id: string) =>
    api.Get(`/api/v1/identity/users/${id}/sessions`),
  revokeUserSession: (id: string, sessionId: string) =>
    api.Delete(`/api/v1/identity/users/${id}/sessions/${sessionId}`),
  revokeAllUserSessions: (id: string) =>
    api.Post(`/api/v1/identity/users/${id}/sessions/revoke-all`, {}),
  roles: (page = 1, size = 20) =>
    api
      .Get<IdentityRole[]>('/api/v1/identity/roles', { page, size })
      .then((r) => normalize<IdentityRole>(r, page, size)),
  role: (id: string) => api.Get<IdentityRole>(`/api/v1/identity/roles/${id}`),
  createRole: (data: { name: string; description?: string }) =>
    api.Post('/api/v1/identity/roles', data),
  deleteRole: (id: string) => api.Delete(`/api/v1/identity/roles/${id}`),
  permissionCatalog: () =>
    api.Get<PermissionCatalogEntry[]>('/api/v1/identity/permissions/catalog'),
  rolePermissions: (id: string) =>
    api.Get<string[]>(`/api/v1/identity/roles/${id}/permissions`),
  updateRolePermissions: (id: string, permissions: string[]) =>
    api.Put(`/api/v1/identity/roles/${id}/permissions`, { permissions }),
  impersonate: (userId: string) =>
    api.Post('/api/v1/identity/impersonation/start', { userId }),
  endImpersonation: () =>
    api.Post('/api/v1/identity/impersonation/end', {}),
  revokeImpersonation: (id: string) =>
    api.Delete(`/api/v1/identity/impersonation/${id}`),
};

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
