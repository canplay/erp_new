import { httpClient } from './index';

export const publishApi = {
  publish: (data: Record<string, unknown>) => httpClient.post<Record<string, unknown>>('/publish', data),
  listTasks: () => httpClient.get<Array<Record<string, unknown>>>('/publish/tasks'),
  listSchedules: () => httpClient.get<Array<Record<string, unknown>>>('/publish/schedules'),
  createSchedule: (data: Record<string, unknown>) => httpClient.post<Record<string, unknown>>('/publish/schedules', data),
  updateSchedule: (id: string, data: Record<string, unknown>) => httpClient.put<Record<string, unknown>>(`/publish/schedules/${id}`, data),
  deleteSchedule: (id: string) => httpClient.delete(`/publish/schedules/${id}`),
};
