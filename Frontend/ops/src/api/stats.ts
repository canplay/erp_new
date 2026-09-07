import { httpClient } from './index';

export const statsApi = {
  accountStats: (id: string) => httpClient.get<Record<string, unknown>>(`/stats/accounts/${id}`),
  contentStats: (id: string) => httpClient.get<Record<string, unknown>>(`/stats/contents/${id}`),
  overview: () => httpClient.get<Record<string, unknown>>('/stats/overview'),
};
