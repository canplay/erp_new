/**
 * @file api-key.ts
 * @description API Key 管理接口
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

/**
 * API Key 权限级别
 */
export type PermissionLevel = 'read' | 'write' | 'admin';

/**
 * API Key 状态
 */
export type KeyStatus = 'active' | 'inactive' | 'revoked';

/**
 * API Key 接口定义
 */
export interface ApiKey {
  id: string;
  name: string;
  description?: string;
  key_prefix: string;
  permission_level: PermissionLevel;
  status: KeyStatus;
  rate_limit?: number;
  allowed_ips?: string[];
  expires_at?: string;
  tenant_id?: string;
  last_used_at?: string;
  created_at: string;
  updated_at?: string;
}

/**
 * 创建 API Key 请求
 */
export interface CreateApiKeyRequest {
  name: string;
  description?: string;
  permission_level: PermissionLevel;
  rate_limit?: number;
  allowed_ips?: string[];
  expires_at?: string;
  tenant_id?: string;
}

/**
 * 更新 API Key 请求
 */
export interface UpdateApiKeyRequest {
  name?: string;
  description?: string;
  permission_level?: PermissionLevel;
  status?: KeyStatus;
  rate_limit?: number;
  allowed_ips?: string[];
  expires_at?: string;
}

/**
 * 查询参数
 */
export interface ApiKeyQuery extends Record<string, unknown> {
  page?: number;
  page_size?: number;
  status?: KeyStatus;
  keyword?: string;
}

/**
 * 分页响应
 */
export interface ApiKeyPageResult {
  records: ApiKey[];
  total: number;
  page: number;
  page_size: number;
}

/**
 * 创建 API Key 响应
 */
export interface CreateApiKeyResponse {
  id: string;
  key_id: string;
  key_secret: string;
  name: string;
  permission_level: string;
  expires_at?: string;
}

/**
 * 统计信息
 */
export interface ApiKeyStats {
  total_keys: number;
  active_keys: number;
  expired_keys: number;
  revoked_keys: number;
  total_requests: number;
  failed_requests: number;
}

/**
 * 获取 API Key 列表
 */
export async function listApiKeys(query: ApiKeyQuery) {
  try {
    return await httpClient.get('/apikeys', { params: query });
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 获取 API Key 详情
 */
export async function getApiKey(id: string) {
  try {
    return await httpClient.get(`/apikeys/${id}`);
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 创建 API Key
 */
export async function createApiKey(data: CreateApiKeyRequest) {
  try {
    return await httpClient.post('/apikeys', data);
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 更新 API Key
 */
export async function updateApiKey(id: string, data: UpdateApiKeyRequest) {
  try {
    return await httpClient.put(`/apikeys/${id}`, data);
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 删除 API Key
 */
export async function deleteApiKey(id: string) {
  try {
    return await httpClient.delete(`/apikeys/${id}`);
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 禁用 API Key
 */
export async function disableApiKey(id: string) {
  try {
    return await httpClient.post(`/apikeys/${id}/disable`);
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 启用 API Key
 */
export async function enableApiKey(id: string) {
  try {
    return await httpClient.post(`/apikeys/${id}/enable`);
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 获取 API Key 统计
 */
export async function getApiKeyStats() {
  try {
    return await httpClient.get('/apikeys/stats');
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}

/**
 * 验证 API Key
 */
export async function validateApiKey(key: string, ip_address?: string) {
  try {
    return await httpClient.post(
    '/apikeys/validate',
    { key, ip_address }  );
  } catch (error) {
    handleApiError(error, 'API密钥管理');
    throw error;
  }
}