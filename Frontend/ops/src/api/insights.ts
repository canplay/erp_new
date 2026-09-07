import { httpClient } from './index';

export interface InsightReport {
  id?: string;
  insight_type: string;
  title?: string;
  analysis_body?: string;
  summary?: string;
  created_at?: string;
  [key: string]: unknown;
}

export const insightApi = {
  generate: (data: Record<string, unknown>) => httpClient.post<InsightReport>('/insights/generate', data),
  list: (accountId: string) => httpClient.get<InsightReport[]>(`/insights/${accountId}`),
  latest: (accountId: string) => httpClient.get<InsightReport>(`/insights/${accountId}/latest`),
};
