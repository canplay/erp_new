import { httpClient } from './index';

export const rewriteApi = {
  createTask: (data: Record<string, unknown>) => httpClient.post<Record<string, unknown>>('/rewrite', data),
  listTasks: () => httpClient.get<Array<Record<string, unknown>>>('/rewrite/tasks'),
  getTask: (id: string) => httpClient.get<Record<string, unknown>>(`/rewrite/tasks/${id}`),
  updateVersion: (id: string, data: Record<string, unknown>) => httpClient.put<Record<string, unknown>>(`/rewrite/versions/${id}`, data),
};
