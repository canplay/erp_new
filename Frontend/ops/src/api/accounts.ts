import { httpClient } from './index';

export interface SocialAccount {
  id: string;
  platform: string;
  account_name: string;
  account_id: string;
  is_active: boolean;
  created_at: string;
}

export const accountApi = {
  list: () => httpClient.get<SocialAccount[]>('/accounts'),
  get: (id: string) => httpClient.get<SocialAccount>(`/accounts/${id}`),
  create: (data: Partial<SocialAccount>) => httpClient.post<SocialAccount>('/accounts', data),
  update: (id: string, data: Partial<SocialAccount>) => httpClient.put<SocialAccount>(`/accounts/${id}`, data),
  delete: (id: string) => httpClient.delete(`/accounts/${id}`),
};
