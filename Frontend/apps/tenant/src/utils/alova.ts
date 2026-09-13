/**
 * @file alova.ts
 * @description 统一 alova 请求模块 — 提供 HTTP 客户端、响应转换、Token 刷新、CSRF 保护
 * @date 2026-08-15
 */

import { createAlova, type Method, type RequestBody } from 'alova';
import { axiosRequestAdapter } from '@alova/adapter-axios';
import { useAuthStore } from '@/stores/auth';
import { logger } from '@/utils/logger';
import { getStorageItem } from '@/utils/storage';
import { isMockMode, resolveMock } from '@/utils/mock';
import type { AlovaAxiosRequestConfig } from '@alova/adapter-axios';
import type { AxiosRequestHeaders, AxiosResponse, AxiosResponseHeaders } from 'axios';
import type { ApiResponse, PaginationParams, HttpResponse, RequestConfig } from '@/types/api';

// ==================== Token 刷新队列 ====================

let isRefreshing = false;
let refreshSubscribers: ((token: string) => void)[] = [];

async function subscribeTokenRefresh(callback: (token: string) => void): void {
  refreshSubscribers.push(callback);
}

async function onTokenRefreshed(newToken: string): void {
  refreshSubscribers.forEach((cb) => cb(newToken));
  refreshSubscribers = [];
}

async function onTokenRefreshFailed(error: unknown): void {
  refreshSubscribers.forEach((cb) => cb(''));
  refreshSubscribers = [];
  void error;
}

// ==================== 常量 ====================

/** API 基础地址 */
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api';

const CSRF_HEADER_NAME = 'X-CSRF-Token';
const CSRF_COOKIE_NAME = 'csrf_token';

const REQUEST_TIMEOUT = 30000;

// ==================== 类型定义 ====================

export type { ApiResponse, PaginationParams, HttpResponse, RequestConfig };

export interface PaginationResponse<T = unknown> {
  list: T[];
  total: number;
  page: number;
  page_size: number;
}

export type PaginatedResponse<T = unknown> = PaginationResponse<T>;

// ==================== CSRF ====================

async function getCsrfToken(): string | null {
  if (typeof document === 'undefined') return null;
  const cookies = document.cookie.split(';');
  for (const cookie of cookies) {
    const [name, value] = cookie.trim().split('=');
    if (name === CSRF_COOKIE_NAME) {
      return decodeURIComponent(value ?? '');
    }
  }
  return null;
}

// ==================== 响应转换 ====================

function transformResponse<T>(response: unknown): HttpResponse<T> {
  const defaultResponse: HttpResponse<T> = { code: 200, message: 'success', success: true, data: null as any };
  if (!response || typeof response !== 'object') {
    return defaultResponse;
  }

  let resp = response as Record<string, unknown>;
  if (!('code' in resp) && !('success' in resp) && 'data' in resp && resp.data && typeof resp.data === 'object') {
    const body = resp.data as Record<string, unknown>;
    if ('code' in body || 'data' in body) {
      resp = body;
    }
  }

  if ('data' in resp && resp.data && typeof resp.data === 'object') {
    const dataObj = resp.data as Record<string, unknown>;
    return {
      code: (resp.code ?? 200) as number,
      message: (resp.message ?? 'success') as string,
      success: (resp.success ?? true) as boolean,
      data: dataObj as T,
      list: dataObj.list as T[],
      total: (dataObj.total as number) ?? 0,
      ...dataObj,
    };
  }

  return {
    code: (resp.code ?? 200) as number,
    message: (resp.message ?? 'success') as string,
    success: (resp.success ?? true) as boolean,
    data: (resp.data ?? null) as T,
    ...resp,
  };
}

// ==================== Mock 适配器 ====================

/**
 * 构造请求适配器: 开发演示模式(VITE_USE_MOCK=true)下按 URL 命中 mock 数据,
 * 未命中或非 mock 模式时完全走真实 axios 请求链路。
 */
async function buildRequestAdapter(): ReturnType<typeof axiosRequestAdapter> {
  const realAdapter = axiosRequestAdapter();
  return (elements, method) => {
    if (isMockMode()) {
      const match = resolveMock({
        path: method.url,
        method: method.type,
        body: method.data,
        params: (method.config.params ?? {}) as Record<string, unknown>,
      });

      if (match) {
        const { status, body } = match;
        const axiosConfig: AxiosResponse['config'] = {
          baseURL: method.baseURL,
          url: method.url,
          data: method.data,
          ...method.config,
          headers: {} as AxiosRequestHeaders,
        };
        const mockResponse: AxiosResponse = {
          data: body,
          status,
          statusText: status >= 200 && status < 300 ? 'OK' : 'Error',
          headers: {},
          config: axiosConfig,
        };
        return {
          response: () => Promise.resolve(mockResponse),
          headers: () => Promise.resolve({} as AxiosResponseHeaders),
          abort: () => {
            /* mock: 无真实请求可中断 */
          },
        };
      }
    }
    return realAdapter(elements, method);
  };
}

// ==================== alova 实例 ====================

export const alovaInstance = createAlova<AlovaAxiosRequestConfig, AxiosResponse, AxiosResponseHeaders>({
  baseURL: API_BASE_URL,
  requestAdapter: buildRequestAdapter(),
  timeout: REQUEST_TIMEOUT,

  beforeRequest(method: Method) {
    const authStore = useAuthStore();
    const token = authStore.token;
    const config = method.config as { headers: Record<string, string>; method?: string; url?: string };

    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    const tenantId = getStorageItem<string>('myai_active_tenant_id', '');
    if (tenantId) {
      config.headers['X-Tenant-Id'] = tenantId;
    }

    const csrfToken = getCsrfToken();
    if (csrfToken) {
      const methodUpper = config.method?.toUpperCase();
      if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(methodUpper || '')) {
        config.headers[CSRF_HEADER_NAME] = csrfToken;
      }
    }

    logger.debug('【Alova 请求】', config.method?.toUpperCase(), config.url);
  },

  responded: {
    onSuccess: (response: unknown) => {
      return transformResponse(response);
    },

    onError: async (error: unknown, method: Method): Promise<void> => {
      const err = error as { response?: { status?: number }; config?: Record<string, unknown> };
      const status = err?.response?.status;

      if (status === 401) {
        const authStore = useAuthStore();
        const retryKey = '__retryCount';
        const retryCount = ((method.config as Record<string, unknown>)[retryKey] as number) || 0;
        if (retryCount >= 2) {
          logger.warn('【Token 刷新重试超限】已退出登录');
          authStore.logout();
          throw new Error('登录状态已失效，请重新登录');
        }
        (method.config as Record<string, unknown>)[retryKey] = retryCount + 1;

        if (isRefreshing) {
          return (new Promise((resolve, reject) => {
            subscribeTokenRefresh((newToken: string) => {
              if (!newToken) {
                reject(new Error('Token 刷新失败，请重新登录'));
                return;
              }
              const config = method.config as { headers: Record<string, string> };
              config.headers.Authorization = `Bearer ${newToken}`;
              (alovaInstance as unknown as { Send(method: Method): Promise<unknown> }).Send(method)
                .then((retryResponse: unknown) => resolve(retryResponse as void))
                .catch((retryError: unknown) => reject(retryError instanceof Error ? retryError : new Error(String(retryError))));
            });
          }) as unknown as void);
        }

        isRefreshing = true;
        try {
          const refreshed = await authStore.refreshAccessToken();
          if (refreshed && authStore.token) {
            onTokenRefreshed(authStore.token);
            const config = method.config as { headers: Record<string, string> };
            config.headers.Authorization = `Bearer ${authStore.token}`;
            const retryResult = await (alovaInstance as unknown as { Send(method: Method): Promise<unknown> }).Send(method);
            return retryResult as void;
          }
          onTokenRefreshFailed(new Error('Token 刷新失败'));
          logger.warn('【Token 刷新失败】已退出登录');
          authStore.logout();
          throw new Error('Token 刷新失败，请重新登录');
        } finally {
          isRefreshing = false;
        }
      }

      if (status === 403) {
        logger.warn('【权限不足】', method.config.url);
        throw new Error('权限不足，无法访问该资源');
      }

      if (status && status >= 500) {
        logger.error('【服务器错误】', status, method.config.url);
        throw new Error('服务器内部错误，请稍后重试');
      }

      throw error;
    },
  },
});

// ==================== 便捷请求方法 ====================

export async function get(url: string, params?: unknown, config?: unknown): Promise<HttpResponse<unknown>> {
  return alovaInstance.Get(url, { params: params as Record<string, unknown>, ...config as Record<string, unknown> });
}

export async function post(url: string, data?: RequestBody, config?: Record<string, unknown>): Promise<HttpResponse<unknown>> {
  return alovaInstance.Post(url, data, config);
}

export async function put(url: string, data?: RequestBody, config?: Record<string, unknown>): Promise<HttpResponse<unknown>> {
  return alovaInstance.Put(url, data, config);
}

export async function patch(url: string, data?: RequestBody, config?: Record<string, unknown>): Promise<HttpResponse<unknown>> {
  return alovaInstance.Patch(url, data, config);
}

export async function del(url: string, params?: unknown, config?: unknown): Promise<HttpResponse<unknown>> {
  return alovaInstance.Delete(url, { params, ...config as Record<string, unknown> });
}

export async function upload(url: string, file: File, fieldName = 'file', config?: unknown): Promise<HttpResponse<unknown>> {
  const formData = new FormData();
  formData.append(fieldName, file);
  return post(url, formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    ...(config as Record<string, unknown>),
  });
}

export async function download(url: string, params?: unknown, config?: unknown): Promise<Blob> {
  return alovaInstance.Get(url, {
    params: params as Record<string, unknown>,
    responseType: 'blob',
    ...(config as Record<string, unknown>),
  });
}

// ==================== 兼容旧代码 ====================

export const request = { get, post, put, patch, delete: del, upload, download };

// ==================== httpClient（API 文件使用的统一入口） ====================

export interface HttpClient {
  get<T = unknown>(url: string, paramsOrConfig?: RequestConfig | Record<string, unknown>, config?: RequestConfig): Promise<HttpResponse<T>>;
  post<T = unknown>(url: string, data?: RequestBody, config?: RequestConfig): Promise<HttpResponse<T>>;
  put<T = unknown>(url: string, data?: RequestBody, config?: RequestConfig): Promise<HttpResponse<T>>;
  patch<T = unknown>(url: string, data?: RequestBody, config?: RequestConfig): Promise<HttpResponse<T>>;
  delete<T = unknown>(url: string, paramsOrConfig?: RequestConfig | Record<string, unknown>, config?: RequestConfig): Promise<HttpResponse<T>>;
  upload<T = unknown>(url: string, file: File, fieldName?: string): Promise<HttpResponse<T>>;
}

async function normalizeParams(params?: unknown): Record<string, string | number | boolean> | undefined {
  if (!params || typeof params !== 'object') return undefined;
  return params as Record<string, string | number | boolean>;
}

async function normalizeConfig(config?: unknown): Record<string, unknown> | undefined {
  if (!config || typeof config !== 'object') return undefined;
  return config as Record<string, unknown>;
}

export const httpClient: HttpClient = {
  async get<T = unknown>(url: string, paramsOrConfig?: RequestConfig | Record<string, unknown>, config?: RequestConfig): Promise<HttpResponse<T>> {
    const params = paramsOrConfig && typeof paramsOrConfig === 'object' && 'params' in paramsOrConfig
      ? (paramsOrConfig).params
      : paramsOrConfig;
    const cfg = paramsOrConfig && typeof paramsOrConfig === 'object' && 'params' in paramsOrConfig
      ? config
      : config;
    try {
      return await alovaInstance.Get<HttpResponse<T>>(url, {
        params: normalizeParams(params) as Record<string, unknown>,
        ...normalizeConfig(cfg),
      });
    } catch (error) {
      logger.error('GET 请求失败:', url, error);
      throw error;
    }
  },

  async post<T = unknown>(url: string, data?: RequestBody, config?: RequestConfig): Promise<HttpResponse<T>> {
    try {
      return await alovaInstance.Post<HttpResponse<T>>(url, data, normalizeConfig(config));
    } catch (error) {
      logger.error('POST 请求失败:', url, error);
      throw error;
    }
  },

  async put<T = unknown>(url: string, data?: RequestBody, config?: RequestConfig): Promise<HttpResponse<T>> {
    try {
      return await alovaInstance.Put<HttpResponse<T>>(url, data, normalizeConfig(config));
    } catch (error) {
      logger.error('PUT 请求失败:', url, error);
      throw error;
    }
  },

  async patch<T = unknown>(url: string, data?: RequestBody, config?: RequestConfig): Promise<HttpResponse<T>> {
    try {
      return await alovaInstance.Patch<HttpResponse<T>>(url, data, normalizeConfig(config));
    } catch (error) {
      logger.error('PATCH 请求失败:', url, error);
      throw error;
    }
  },

  async delete<T = unknown>(url: string, paramsOrConfig?: RequestConfig | Record<string, unknown>, config?: RequestConfig): Promise<HttpResponse<T>> {
    const params = paramsOrConfig && typeof paramsOrConfig === 'object' && 'params' in paramsOrConfig
      ? (paramsOrConfig).params
      : paramsOrConfig;
    const cfg = paramsOrConfig && typeof paramsOrConfig === 'object' && 'params' in paramsOrConfig
      ? config
      : config;
    try {
      return await alovaInstance.Delete<HttpResponse<T>>(url, {
        params: normalizeParams(params) as Record<string, unknown>,
        ...normalizeConfig(cfg),
      });
    } catch (error) {
      logger.error('DELETE 请求失败:', url, error);
      throw error;
    }
  },

  async upload<T = unknown>(url: string, file: File, fieldName = 'file'): Promise<HttpResponse<T>> {
    const formData = new FormData();
    formData.append(fieldName, file);
    try {
      return await alovaInstance.Post<HttpResponse<T>>(url, formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      });
    } catch (error) {
      logger.error('UPLOAD 请求失败:', url, error);
      throw error;
    }
  },
};

// ==================== 默认导出 ====================

export default alovaInstance;
