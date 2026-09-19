// @viewpoint platform-only
// 复用共享 alova 封装（packages/api），不再重复定义 class M
import { api, normalize, unwrapArray } from '@erp-new-frontend-monorepo/api';

export interface TicketDto {
  id: string;
  title: string;
  description?: string | null;
  status: string;
  priority: string;
  assigneeId?: string | null;
  assigneeName?: string | null;
  reporterId: string;
  reporterName: string;
  createdAtUtc: string;
  updatedAtUtc?: string | null;
  closedAtUtc?: string | null;
  resolvedAtUtc?: string | null;
}

export interface CommentDto {
  id: string;
  ticketId?: string;
  authorUserId: string;
  body: string;
  createdAtUtc: string;
}

export interface CreateTicketDto {
  title: string;
  description?: string;
  priority?: string;
}

export interface UpdateTicketDto {
  title?: string;
  description?: string;
  priority?: string;
  status?: string;
  assigneeId?: string | null;
}

export const ticketsApi = {
  list: (params: { page?: number; pageSize?: number; status?: string } = {}) => {
    const query: Record<string, string | number | boolean> = {};
    if (params.page != null) query.pageNumber = params.page;
    if (params.pageSize != null) query.pageSize = params.pageSize;
    if (params.status != null) query.status = params.status;
    return api
      .Get<TicketDto[]>('/api/v1/tickets', query)
      .then((r) => normalize<TicketDto>(r, params.page ?? 1, params.pageSize ?? 20));
  },
  detail: (id: string) => api.Get<TicketDto>(`/api/v1/tickets/${id}`),
  comments: (id: string) =>
    api.Get<CommentDto[]>(`/api/v1/tickets/${id}/comments`).then((r) => unwrapArray<CommentDto>(r)),
  create: (data: CreateTicketDto) => api.Post('/api/v1/tickets', data),
  update: (id: string, data: UpdateTicketDto) => api.Put(`/api/v1/tickets/${id}`, data),
  assign: (id: string, userId: string) => api.Post(`/api/v1/tickets/${id}/assign`, { userId }),
  close: (id: string) => api.Post(`/api/v1/tickets/${id}/close`, {}),
  reopen: (id: string) => api.Post(`/api/v1/tickets/${id}/reopen`, {}),
  resolve: (id: string, note?: string) =>
    api.Post(`/api/v1/tickets/${id}/resolve`, { resolutionNote: note }),
  addComment: (id: string, body: string) => api.Post(`/api/v1/tickets/${id}/comments`, { body }),
  trash: () => api.Get<TicketDto[]>('/api/v1/tickets/trash').then((r) => unwrapArray<TicketDto>(r)),
  restore: (id: string) => api.Post(`/api/v1/tickets/${id}/restore`, {}),
  deleteTicket: (id: string) => api.Delete(`/api/v1/tickets/${id}`),
};
