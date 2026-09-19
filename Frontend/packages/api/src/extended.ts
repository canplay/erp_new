import { api, normalize } from '@erp-new-frontend-monorepo/api';

// ==================== Current User API ====================

export interface CurrentUserDto {
  id: string;
  userName: string;
  email: string;
  fullName?: string;
  roles?: string[];
  permissions?: string[];
}

export const currentUserApi = {
  me: () => api.Get<CurrentUserDto>('/api/v1/identity/users/me'),
  updateProfile: (data: { fullName?: string; email?: string }) =>
    api.Put('/api/v1/identity/users/me', data),
  changePassword: (data: { currentPassword: string; newPassword: string }) =>
    api.Post('/api/v1/identity/users/me/password', data),
};

// ==================== Forgot Password API ====================

export const forgotPasswordApi = {
  sendResetEmail: (email: string) =>
    api.Post('/api/v1/identity/forgot-password', { email }),
  resetPassword: (token: string, newPassword: string) =>
    api.Post('/api/v1/identity/reset-password', { token, newPassword }),
};

// ==================== Files API ====================

export interface FileDto {
  id: string;
  fileName: string;
  fileUrl: string;
  mimeType: string;
  size: number;
  createdAtUtc: string;
}

export const filesApi = {
  upload: (file: File, folder?: string) => {
    const fd = new FormData();
    fd.append('file', file);
    if (folder) fd.append('folder', folder);
    return api.Post<FileDto>('/api/v1/files/upload', fd);
  },
  download: (id: string) => api.Get(`/api/v1/files/${id}/download`),
  delete: (id: string) => api.Delete(`/api/v1/files/${id}`),
  list: (page = 1, size = 20) =>
    api
      .Get<FileDto[]>('/api/v1/files', { page, size })
      .then((r) => normalize<FileDto>(r, page, size)),
};

// ==================== Notifications API ====================

export interface NotificationDto {
  id: string;
  type: string;
  title: string;
  body?: string | null;
  link?: string | null;
  isRead: boolean;
  createdAtUtc: string;
}

export const notificationsApi = {
  list: (page = 1, size = 20) =>
    api
      .Get<NotificationDto[]>('/api/v1/notifications', { page, size })
      .then((r) => normalize<NotificationDto>(r, page, size)),
  unreadCount: () =>
    api.Get<number>('/api/v1/notifications/unread-count'),
  markRead: (id: string) =>
    api.Post(`/api/v1/notifications/${id}/read`, {}),
  markAllRead: () =>
    api.Post('/api/v1/notifications/read-all', {}),
  delete: (id: string) =>
    api.Delete(`/api/v1/notifications/${id}`),
};

// ==================== Audits API ====================

export interface AuditSummaryDto {
  id: string;
  action: string;
  resource: string;
  userId: string;
  userName: string;
  timestampUtc: string;
  ipAddress?: string;
  userAgent?: string;
  details?: string;
}

export interface AuditDetailDto extends AuditSummaryDto {
  requestBody?: string;
  responseBody?: string;
  changes?: Record<string, { old: unknown; new: unknown }>;
}

export interface AuditSummaryAggregateDto {
  totalActions: number;
  actionsByType: Record<string, number>;
  actionsByResource: Record<string, number>;
}

export const auditsApi = {
  list: <T = AuditSummaryDto>(page = 1, size = 20) =>
    api
      .Get<T[]>('/api/v1/audits', { page, size })
      .then((r) => normalize<T>(r, page, size)),
  detail: (id: string) =>
    api.Get<AuditDetailDto>(`/api/v1/audits/${id}`),
  aggregate: () =>
    api.Get<AuditSummaryAggregateDto>('/api/v1/audits/aggregate'),
  export: (params: { startDate?: string; endDate?: string; format?: 'csv' | 'json' }) =>
    api.Get('/api/v1/audits/export', params),
};

// ==================== Analytics API ====================

export interface ComplianceAssessmentDto {
  overallScore: number;
  categories: Array<{
    name: string;
    score: number;
    recommendations: string[];
  }>;
  lastAssessmentDate: string;
}

export interface IssueAnalysisDto {
  totalIssues: number;
  bySeverity: Record<string, number>;
  byStatus: Record<string, number>;
  topIssues: Array<{
    id: string;
    title: string;
    severity: string;
    status: string;
  }>;
}

export interface WeeklyPlanDto {
  week: string;
  goals: string[];
  completedGoals: string[];
  upcomingTasks: string[];
}

export const analyticsApi = {
  complianceAssessment: () =>
    api.Get<ComplianceAssessmentDto>('/api/v1/analytics/compliance-assessment'),
  issueAnalysis: () =>
    api.Get<IssueAnalysisDto>('/api/v1/analytics/issue-analysis'),
  overview: (params: { startDate?: string; endDate?: string } = {}) =>
    api.Get('/api/v1/analytics/overview', params),
};

export const weeklyApi = {
  list: (page = 1, size = 20) =>
    api
      .Get('/api/v1/weekly-checks', { page, size })
      .then((r) => normalize(r, page, size)),
  analysis: () =>
    api.Get<IssueAnalysisDto>('/api/v1/weekly-checks/analysis'),
  plan: () =>
    api.Get<WeeklyPlanDto>('/api/v1/weekly-checks/plan'),
  create: (data: Record<string, unknown>) =>
    api.Post('/api/v1/weekly-checks', data),
  update: (id: string, data: Record<string, unknown>) =>
    api.Put(`/api/v1/weekly-checks/${id}`, data),
  delete: (id: string) =>
    api.Delete(`/api/v1/weekly-checks/${id}`),
};

export const monthlyApi = {
  list: (page = 1, size = 20) =>
    api
      .Get('/api/v1/monthly-reports', { page, size })
      .then((r) => normalize(r, page, size)),
  get: (id: string) =>
    api.Get(`/api/v1/monthly-reports/${id}`),
  generate: (month: string) =>
    api.Post('/api/v1/monthly-reports/generate', { month }),
  delete: (id: string) =>
    api.Delete(`/api/v1/monthly-reports/${id}`),
};

// ==================== Search API ====================

export interface SearchResultDto {
  id: string;
  title: string;
  content: string;
  type: string;
  score: number;
  metadata?: Record<string, unknown>;
}

export const searchApi = {
  search: (params: { query: string; type?: string; page?: number; size?: number }) =>
    api
      .Get<SearchResultDto[]>('/api/v1/search', params)
      .then((r) => normalize<SearchResultDto>(r, params.page ?? 1, params.size ?? 20)),
  suggest: (query: string) =>
    api.Get<string[]>('/api/v1/search/suggest', { query }),
};

// ==================== Knowledge Graph API ====================

export interface KnowledgeNodeDto {
  uid: string;
  title: string;
  content: string;
  category: string;
  tags?: string[];
  createdAtUtc?: string;
  updatedAtUtc?: string;
}

export interface KnowledgeEdgeDto {
  sourceUid: string;
  targetUid: string;
  relation: string;
  weight?: number;
}

export const knowledgeGraphApi = {
  expand: (uid: string) =>
    api.Get<{ nodes: KnowledgeNodeDto[]; edges: KnowledgeEdgeDto[] }>(
      `/api/v1/knowledge-graph/nodes/${uid}/expand`
    ),
  node: (uid: string) =>
    api.Get<KnowledgeNodeDto>(`/api/v1/knowledge-graph/nodes/${uid}`),
  search: (query: string) =>
    api.Get<KnowledgeNodeDto[]>('/api/v1/knowledge-graph/search', { query }),
  createNode: (data: Partial<KnowledgeNodeDto>) =>
    api.Post<KnowledgeNodeDto>('/api/v1/knowledge-graph/nodes', data),
  updateNode: (uid: string, data: Partial<KnowledgeNodeDto>) =>
    api.Put<KnowledgeNodeDto>(`/api/v1/knowledge-graph/nodes/${uid}`, data),
  deleteNode: (uid: string) =>
    api.Delete(`/api/v1/knowledge-graph/nodes/${uid}`),
  related: (uid: string, depth = 1) =>
    api.Get<KnowledgeNodeDto[]>(`/api/v1/knowledge-graph/nodes/${uid}/related`, { depth }),
};

// ==================== Personnel API ====================

export interface PersonnelDto {
  id: string;
  name: string;
  email: string;
  department?: string;
  position?: string;
  isActive: boolean;
  createdAtUtc?: string;
}

export const personnelApi = {
  list: (params: { page?: number; size?: number; search?: string } = {}) =>
    api
      .Get<PersonnelDto[]>('/api/v1/personnel', params)
      .then((r) => normalize<PersonnelDto>(r, params.page ?? 1, params.size ?? 20)),
  get: (id: string) =>
    api.Get<PersonnelDto>(`/api/v1/personnel/${id}`),
  create: (data: Partial<PersonnelDto>) =>
    api.Post<PersonnelDto>('/api/v1/personnel', data),
  update: (id: string, data: Partial<PersonnelDto>) =>
    api.Put<PersonnelDto>(`/api/v1/personnel/${id}`, data),
  delete: (id: string) =>
    api.Delete(`/api/v1/personnel/${id}`),
};

// ==================== Daily Control API ====================

export interface DailyTaskDto {
  id: string;
  title: string;
  description?: string;
  status: string;
  priority: string;
  assignedTo?: string;
  dueDateUtc?: string;
  createdAtUtc?: string;
}

export const dailyApi = {
  tasks: (params: { page?: number; size?: number; status?: string } = {}) =>
    api
      .Get<DailyTaskDto[]>('/api/v1/daily-tasks', params)
      .then((r) => normalize<DailyTaskDto>(r, params.page ?? 1, params.size ?? 20)),
  get: (id: string) =>
    api.Get<DailyTaskDto>(`/api/v1/daily-tasks/${id}`),
  create: (data: Partial<DailyTaskDto>) =>
    api.Post<DailyTaskDto>('/api/v1/daily-tasks', data),
  update: (id: string, data: Partial<DailyTaskDto>) =>
    api.Put<DailyTaskDto>(`/api/v1/daily-tasks/${id}`, data),
  delete: (id: string) =>
    api.Delete(`/api/v1/daily-tasks/${id}`),
  complete: (id: string) =>
    api.Post(`/api/v1/daily-tasks/${id}/complete`, {}),
  reopen: (id: string) =>
    api.Post(`/api/v1/daily-tasks/${id}/reopen`, {}),
};

// ==================== Device API ====================

export interface Device {
  id: string;
  deviceNo: string;
  name: string;
  type: string;
  status: string;
  location?: string;
  lastActivityUtc?: string;
  createdAtUtc?: string;
}

export interface DeviceInspectionDto {
  id: string;
  deviceId: string;
  inspectorName: string;
  inspectionDateUtc: string;
  result: string;
  notes?: string;
}

export interface DeviceRepairDto {
  id: string;
  deviceId: string;
  description: string;
  status: string;
  cost?: number;
  completedAtUtc?: string;
}

export interface DeviceSparePartDto {
  id: string;
  deviceId: string;
  partName: string;
  quantity: number;
  replacedAtUtc?: string;
}

export const deviceApi = {
  list: (page = 1, size = 20) =>
    api
      .Get<Device[]>('/api/v1/devices', { page, size })
      .then((r) => normalize<Device>(r, page, size)),
  detail: (id: string) => api.Get<Device>(`/api/v1/devices/${id}`),
  create: (data: Partial<Device>) => api.Post<Device>('/api/v1/devices', data),
  update: (id: string, data: Partial<Device>) =>
    api.Put<Device>(`/api/v1/devices/${id}`, data),
  delete: (id: string) => api.Delete(`/api/v1/devices/${id}`),
  inspectionsPaged: (id: string, page = 1, size = 20) =>
    api
      .Get<DeviceInspectionDto[]>(`/api/v1/devices/${id}/inspections`, { page, size })
      .then((r) => normalize<DeviceInspectionDto>(r, page, size)),
  repairs: (id: string, page = 1, size = 20) =>
    api
      .Get<DeviceRepairDto[]>(`/api/v1/devices/${id}/repairs`, { page, size })
      .then((r) => normalize<DeviceRepairDto>(r, page, size)),
  spareParts: (id: string) =>
    api
      .Get<DeviceSparePartDto[]>(`/api/v1/devices/${id}/spare-parts`),
};

// ==================== Hidden Danger API ====================

export interface HiddenDanger {
  id: string;
  title: string;
  description?: string;
  severity: string;
  status: string;
  reportedBy: string;
  assignedTo?: string;
  dueDateUtc?: string;
  createdAtUtc?: string;
}

export const dangerApi = {
  list: (page = 1, size = 20) =>
    api
      .Get<HiddenDanger[]>('/api/v1/hidden-dangers', { page, size })
      .then((r) => normalize<HiddenDanger>(r, page, size)),
  detail: (id: string) => api.Get<HiddenDanger>(`/api/v1/hidden-dangers/${id}`),
  create: (data: Partial<HiddenDanger>) =>
    api.Post<HiddenDanger>('/api/v1/hidden-dangers', data),
  update: (id: string, data: Partial<HiddenDanger>) =>
    api.Put<HiddenDanger>(`/api/v1/hidden-dangers/${id}`, data),
  delete: (id: string) =>
    api.Delete(`/api/v1/hidden-dangers/${id}`),
  resolve: (id: string, notes?: string) =>
    api.Post(`/api/v1/hidden-dangers/${id}/resolve`, { notes }),
};
