import { httpClient } from './index';

export interface LLMProvider {
  id: string;
  provider_name: string;
  api_endpoint: string;
  model_name: string;
  is_active: boolean;
}

export const llmApi = {
  list: () => httpClient.get<LLMProvider[]>('/llm/providers'),
  add: (data: Partial<LLMProvider>) => httpClient.post<LLMProvider>('/llm/providers', data),
  update: (id: string, data: Partial<LLMProvider>) => httpClient.put<LLMProvider>(`/llm/providers/${id}`, data),
  delete: (id: string) => httpClient.delete(`/llm/providers/${id}`),
  test: (data: Partial<LLMProvider>) => httpClient.post<boolean>('/llm/providers/test', data),
};
