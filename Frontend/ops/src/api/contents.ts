import { httpClient } from './index';

export interface ContentItem {
  id: string;
  title: string;
  content_type: string;
  status: string;
  source_url: string;
  created_at: string;
  body?: string;
}

export const contentApi = {
  list: (params?: Record<string, unknown>) => httpClient.get<ContentItem[]>('/contents', params),
  get: (id: string) => httpClient.get<ContentItem>(`/contents/${id}`),
  create: (data: Partial<ContentItem>) => httpClient.post<ContentItem>('/contents/manual', data),
  update: (id: string, data: Partial<ContentItem>) => httpClient.put<ContentItem>(`/contents/${id}`, data),
  delete: (id: string) => httpClient.delete(`/contents/${id}`),
};
