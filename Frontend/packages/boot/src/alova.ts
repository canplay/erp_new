/**
 * @file alova.ts
 * @description 全局 alova HTTP 客户端配置
 */

import { createAlova } from 'alova';
import { axiosRequestAdapter } from '@alova/adapter-axios';
import type { AxiosResponse } from 'axios';

const TOKEN_KEY = 'admin_token';

export function getAccessToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setAccessToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function isAuthenticated(): boolean {
  return !!getAccessToken();
}

export function logout(): void {
  localStorage.removeItem(TOKEN_KEY);
  window.location.href = '/auth/login';
}

type ErrorNotifier = (msg: string) => void;
let errorNotifier: ErrorNotifier = (msg) => console.error('[API]', msg);

export function setErrorNotifier(fn: ErrorNotifier): void {
  errorNotifier = fn;
}

let alovaInstance: ReturnType<typeof createAlova> | null = null;

export async function setupAlova(): Promise<ReturnType<typeof createAlova>> {
  if (alovaInstance) return alovaInstance;

  const baseURL = import.meta.env.VITE_API_BASE_URL || '/api';

  const instance = createAlova({
    baseURL,
    timeout: 30000,
    requestAdapter: axiosRequestAdapter(),
    beforeRequest(method) {
      const token = getAccessToken();
      if (token) {
        method.config.headers = method.config.headers || {};
        (method.config.headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
      }
    },
    responded: {
      onSuccess: async (response: AxiosResponse) => {
        const data = response.data;
        if (data && typeof data === 'object' && 'success' in data) {
          if (!data.success) {
            const msg = data.error || data.message || '请求失败';
            errorNotifier(msg);
            throw new Error(msg);
          }
          return data.data;
        }
        return data;
      },
      onError: (error) => {
        const status = error?.response?.status;
        if (status === 401) {
          logout();
          return;
        }
        const msg = error?.response?.data?.error || error?.message || '网络错误';
        errorNotifier(msg);
        throw error;
      },
    },
  });

  alovaInstance = instance;
  return instance;
}

export function getAlova(): ReturnType<typeof createAlova> {
  if (!alovaInstance) {
    throw new Error('Alova not initialized. Call setupAlova() first.');
  }
  return alovaInstance;
}
