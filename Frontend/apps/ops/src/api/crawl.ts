import { httpClient } from './index';

export interface CrawlSource {
  id: string;
  platform: string;
  source_name: string;
  is_active: boolean;
  [key: string]: unknown;
}

export const crawlApi = {
  listSources: () => httpClient.get<CrawlSource[]>('/crawl/sources'),
  createSource: (data: Record<string, unknown>) => httpClient.post<CrawlSource>('/crawl/sources', data),
  updateSource: (id: string, data: Record<string, unknown>) => httpClient.put<CrawlSource>(`/crawl/sources/${id}`, data),
  deleteSource: (id: string) => httpClient.delete(`/crawl/sources/${id}`),
  trigger: (id: string) => httpClient.post(`/crawl/sources/${id}/trigger`),
  history: (id: string) => httpClient.get<Array<Record<string, unknown>>>(`/crawl/sources/${id}/tasks`),
};
