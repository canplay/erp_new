/**
 * @file api-key.ts
 * @description API Key 管理接口
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

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
export function listApiKeys(query: ApiKeyQuery) {
  return httpClient.get<ApiKeyPageResult>('/apikeys', { params: query });
}

/**
 * 获取 API Key 详情
 */
export function getApiKey(id: string) {
  return httpClient.get<ApiKey>(`/apikeys/${id}`);
}

/**
 * 创建 API Key
 */
export function createApiKey(data: CreateApiKeyRequest) {
  return httpClient.post<CreateApiKeyResponse>('/apikeys', data);
}

/**
 * 更新 API Key
 */
export function updateApiKey(id: string, data: UpdateApiKeyRequest) {
  return httpClient.put<ApiKey>(`/apikeys/${id}`, data);
}

/**
 * 删除 API Key
 */
export function deleteApiKey(id: string) {
  return httpClient.delete(`/apikeys/${id}`);
}

/**
 * 禁用 API Key
 */
export function disableApiKey(id: string) {
  return httpClient.post(`/apikeys/${id}/disable`);
}

/**
 * 启用 API Key
 */
export function enableApiKey(id: string) {
  return httpClient.post(`/apikeys/${id}/enable`);
}

/**
 * 获取 API Key 统计
 */
export function getApiKeyStats() {
  return httpClient.get<ApiKeyStats>('/apikeys/stats');
}

/**
 * 验证 API Key
 */
export function validateApiKey(key: string, ip_address?: string) {
  return httpClient.post<{ valid: boolean; error?: string; key_id?: string; permission_level?: string }>(
    '/apikeys/validate',
    { key, ip_address }
  );
}