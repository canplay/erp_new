/**
 * @file alova.ts
 * @description 统一 HTTP 客户端 — 基于 alova + axios-adapter
 * @date 2026-07-06
 */

import { createAlova } from 'alova';
import { axiosRequestAdapter } from '@alova/adapter-axios';
import { type AlovaAxiosRequestConfig } from '@alova/adapter-axios';
import { type AxiosResponse, type AxiosResponseHeaders } from 'axios';

// ==================== 常量 ====================

const API_BASE_URL = '/api/v1/social-ops';
const REQUEST_TIMEOUT = 30_000;

// ==================== 类型 ====================

/** 后端标准响应格式 */
export interface ApiResponse<T = unknown> {
  code?: number;
  message?: string;
  data?: T;
  success?: boolean;
}

export interface PaginationParams {
  page?: number;
  pageSize?: number;
}

export interface PaginatedResponse<T = unknown> {
  list: T[];
  total: number;
  page?: number;
  pageSize?: number;
}

// ==================== alova 实例 ====================

export const alovaInstance = createAlova<AlovaAxiosRequestConfig, AxiosResponse, AxiosResponseHeaders>({
  baseURL: API_BASE_URL,
  requestAdapter: axiosRequestAdapter(),
  timeout: REQUEST_TIMEOUT,

  beforeRequest() {
    // 可在此注入 Token（若 ops 也需要登录态）
  },

  responded: {
    onSuccess: (response: unknown) => {
      // 后端格式: { code, message, data: T } → 直接返回 data 字段
      const resp = response as Record<string, unknown> | null;
      if (resp && typeof resp === 'object' && 'data' in resp) {
        return resp.data;
      }
      return response;
    },
    onError: (_error: unknown) => {
      const err = _error as { response?: { status?: number }; message?: string; code?: string };
      const status = err?.response?.status;

      // 网络错误（无响应）
      if (err?.code === 'ERR_NETWORK' || err?.message?.includes('Network Error')) {
        if (import.meta.env.DEV) console.error('[网络错误] 无法连接到服务器:', API_BASE_URL);
        throw new Error('网络连接失败，请检查网络');
      }

      // 超时
      if (err?.code === 'ECONNABORTED' || err?.message?.includes('timeout')) {
        if (import.meta.env.DEV) console.error('[请求超时]', API_BASE_URL);
        throw new Error('请求超时，请稍后重试');
      }

      if (status === 401) {
        if (import.meta.env.DEV) console.warn('[未授权] 登录已过期');
        throw new Error('未登录或登录已过期');
      }
      if (status === 403) {
        if (import.meta.env.DEV) console.warn('[权限不足]');
        throw new Error('权限不足');
      }
      if (status && status >= 500) {
        if (import.meta.env.DEV) console.error('[服务器错误]', status);
        throw new Error('服务器内部错误，请稍后重试');
      }

      if (import.meta.env.DEV) console.error('[请求失败]', err?.message ?? '未知错误');
      throw new Error(err?.message ?? '请求失败');
    },
  },
});

// ==================== HTTP 客户端 ====================

export interface HttpClient {
  get<T = unknown>(url: string, params?: Record<string, unknown>): Promise<T>;
  post<T = unknown>(url: string, data?: unknown): Promise<T>;
  put<T = unknown>(url: string, data?: unknown): Promise<T>;
  delete<T = unknown>(url: string): Promise<T>;
}

export const httpClient: HttpClient = {
  get<T>(url: string, params?: Record<string, unknown>): Promise<T> {
    return alovaInstance.Get<T>(url, params ? { params } : undefined) as Promise<T>;
  },
  post<T>(url: string, data?: unknown): Promise<T> {
    return alovaInstance.Post<T>(url, data as Record<string, unknown>) as Promise<T>;
  },
  put<T>(url: string, data?: unknown): Promise<T> {
    return alovaInstance.Put<T>(url, data as Record<string, unknown>) as Promise<T>;
  },
  delete<T>(url: string): Promise<T> {
    return alovaInstance.Delete<T>(url) as Promise<T>;
  },
};

export default alovaInstance;
