/**
 * @file security-service.ts
 * @brief 安全服务（security-service）
 * @date 2026-06-18
 * @description 对接 Backend/Rust security-service
 */

import { httpClient } from '@/utils/alova';
import type { HttpResponse } from '@/utils/alova';

// ============ 类型定义 ============

/**
 * @brief IP 白名单规则
 */
export interface IpWhitelistRule {
  id: number;
  name: string;
  ip_type: 'ip' | 'cidr' | 'range';
  ip_start: string;
  ip_end?: string;
  mask?: string;
  target_type: 'all' | 'role' | 'user';
  target_ids?: number[];
  effect_start_time?: string;
  effect_end_time?: string;
  description?: string;
  priority: number;
  status: number;
  created_by?: number;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 敏感操作记录
 */
export interface SensitiveOperation {
  id: number;
  operation_type: string;
  operation_desc?: string;
  user_id: number;
  user_name: string;
  confirm_type: 'password' | 'sms' | 'email' | 'authenticator' | 'admin';
  confirm_status: 'pending' | 'success' | 'failed' | 'expired';
  confirm_data?: Record<string, unknown>;
  verify_code?: string;
  verify_expire_time?: string;
  target_resource?: string;
  target_resource_name?: string;
  ip_address?: string;
  operation_status: 'success' | 'failed';
  failure_reason?: string;
  created_at: string;
}

/**
 * @brief IP 白名单查询参数
 */
export interface IpWhitelistQueryParams {
  ip_type?: 'ip' | 'cidr' | 'range';
  target_type?: 'all' | 'role' | 'user';
  status?: number;
  keyword?: string;
  page?: number;
  page_size?: number;
}

/**
 * @brief IP 白名单创建参数
 */
export interface IpWhitelistCreateParams {
  name: string;
  ip_type: 'ip' | 'cidr' | 'range';
  ip_start: string;
  ip_end?: string;
  mask?: string;
  target_type: 'all' | 'role' | 'user';
  target_ids?: number[];
  effect_start_time?: string;
  effect_end_time?: string;
  description?: string;
  priority?: number;
  status?: number;
}

/**
 * @brief IP 白名单更新参数
 */
export interface IpWhitelistUpdateParams extends IpWhitelistCreateParams {
  id: number;
}

/**
 * @brief 敏感操作查询参数
 */
export interface SensitiveOperationQueryParams {
  operation_type?: string;
  user_id?: number;
  confirm_type?: 'password' | 'sms' | 'email' | 'authenticator' | 'admin';
  confirm_status?: 'pending' | 'success' | 'failed' | 'expired';
  operation_status?: 'success' | 'failed';
  start_date?: string;
  end_date?: string;
  page?: number;
  page_size?: number;
}

/**
 * @brief 发起敏感操作验证
 */
export interface InitiateVerifyParams {
  operation_type: string;
  operation_desc?: string;
  confirm_type: 'password' | 'sms' | 'email' | 'authenticator' | 'admin';
  target_resource?: string;
  target_resource_name?: string;
}

/**
 * @brief 提交验证
 */
export interface SubmitVerifyParams {
  verify_code: string;
}

// ============ IP 白名单 API ============

/**
 * @brief 获取 IP 白名单规则列表
 * @description GET /api/security/ip-whitelist
 */
export function getIpWhitelist(
  params?: IpWhitelistQueryParams
): Promise<HttpResponse<IpWhitelistRule[]>> {
  return httpClient.get<IpWhitelistRule[]>('/security/ip-whitelist', { params });
}

/**
 * @brief 获取 IP 白名单规则详情
 * @description GET /api/security/ip-whitelist/:id
 */
export function getIpWhitelistDetail(
  id: number
): Promise<HttpResponse<IpWhitelistRule>> {
  return httpClient.get<IpWhitelistRule>(`/security/ip-whitelist/${id}`);
}

/**
 * @brief 创建 IP 白名单规则
 * @description POST /api/security/ip-whitelist
 */
export function createIpWhitelist(
  data: IpWhitelistCreateParams
): Promise<HttpResponse<IpWhitelistRule>> {
  return httpClient.post<IpWhitelistRule>('/security/ip-whitelist', data);
}

/**
 * @brief 更新 IP 白名单规则
 * @description PUT /api/security/ip-whitelist/:id
 */
export function updateIpWhitelist(
  id: number,
  data: IpWhitelistUpdateParams
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/security/ip-whitelist/${id}`, data);
}

/**
 * @brief 删除 IP 白名单规则
 * @description DELETE /api/security/ip-whitelist/:id
 */
export function deleteIpWhitelist(id: number): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(`/security/ip-whitelist/${id}`);
}

/**
 * @brief 批量删除 IP 白名单规则
 * @description DELETE /api/security/ip-whitelist/batch
 */
export function batchDeleteIpWhitelist(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>('/security/ip-whitelist/batch', { data: { ids } });
}

/**
 * @brief 启用 IP 白名单规则
 * @description PUT /api/security/ip-whitelist/:id/enable
 */
export function enableIpWhitelist(id: number): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/security/ip-whitelist/${id}/enable`);
}

/**
 * @brief 禁用 IP 白名单规则
 * @description PUT /api/security/ip-whitelist/:id/disable
 */
export function disableIpWhitelist(id: number): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/security/ip-whitelist/${id}/disable`);
}

/**
 * @brief 批量启用规则
 * @description PUT /api/security/ip-whitelist/batch-enable
 */
export function batchEnableIpWhitelist(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.put<void>('/security/ip-whitelist/batch-enable', { ids });
}

/**
 * @brief 批量禁用规则
 * @description PUT /api/security/ip-whitelist/batch-disable
 */
export function batchDisableIpWhitelist(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.put<void>('/security/ip-whitelist/batch-disable', { ids });
}

/**
 * @brief 调整规则优先级
 * @description PUT /api/security/ip-whitelist/reorder
 */
export function reorderIpWhitelist(
  orders: Array<{ id: number; priority: number }>
): Promise<HttpResponse<void>> {
  return httpClient.put<void>('/security/ip-whitelist/reorder', { orders });
}

/**
 * @brief 验证 IP 是否在白名单内
 * @description GET /api/security/ip-whitelist/check
 */
export function checkIpWhitelist(
  ip: string
): Promise<HttpResponse<{ is_whitelisted: boolean; rules: IpWhitelistRule[] }>> {
  return httpClient.get<{ is_whitelisted: boolean; rules: IpWhitelistRule[] }>(
    '/security/ip-whitelist/check',
    { params: { ip } }
  );
}

/**
 * @brief 获取 IP 白名单统计
 * @description GET /api/security/ip-whitelist/statistics
 */
export function getIpWhitelistStatistics(): Promise<HttpResponse<Record<string, unknown>>> {
  return httpClient.get<Record<string, unknown>>('/security/ip-whitelist/statistics');
}

/**
 * @brief 获取 IP 归属地信息
 * @description GET /api/security/ip-whitelist/location
 */
export function getIpLocation(
  ip: string
): Promise<HttpResponse<{ country?: string; province?: string; city?: string; isp?: string }>> {
  return httpClient.get<{ country?: string; province?: string; city?: string; isp?: string }>(
    '/security/ip-whitelist/location',
    { params: { ip } }
  );
}

// ============ 敏感操作审计 API ============

/**
 * @brief 获取敏感操作记录列表
 * @description GET /api/security/sensitive-audit
 */
export function getSensitiveOperations(
  params?: SensitiveOperationQueryParams
): Promise<HttpResponse<SensitiveOperation[]>> {
  return httpClient.get<SensitiveOperation[]>('/security/sensitive-audit', { params });
}

/**
 * @brief 获取敏感操作详情
 * @description GET /api/security/sensitive-audit/:id
 */
export function getSensitiveOperationDetail(
  id: number
): Promise<HttpResponse<SensitiveOperation>> {
  return httpClient.get<SensitiveOperation>(`/security/sensitive-audit/${id}`);
}

/**
 * @brief 获取当前用户待验证操作
 * @description GET /api/security/sensitive-audit/pending
 */
export function getPendingVerifications(): Promise<HttpResponse<SensitiveOperation[]>> {
  return httpClient.get<SensitiveOperation[]>('/security/sensitive-audit/pending');
}

/**
 * @brief 发起敏感操作验证
 * @description POST /api/security/sensitive-audit/initiate
 */
export function initiateVerification(
  data: InitiateVerifyParams
): Promise<HttpResponse<{ operation_id: number; verify_code?: string }>> {
  return httpClient.post<{ operation_id: number; verify_code?: string }>(
    '/security/sensitive-audit/initiate',
    data
  );
}

/**
 * @brief 提交验证
 * @description POST /api/security/sensitive-audit/:operationId/verify
 */
export function submitVerification(
  operationId: number,
  data: SubmitVerifyParams
): Promise<HttpResponse<void>> {
  return httpClient.post<void>(
    `/security/sensitive-audit/${operationId}/verify`,
    data
  );
}

/**
 * @brief 取消验证
 * @description PUT /api/security/sensitive-audit/:operationId/cancel
 */
export function cancelVerification(
  operationId: number
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(
    `/security/sensitive-audit/${operationId}/cancel`
  );
}

/**
 * @brief 管理员审批
 * @description PUT /api/security/sensitive-audit/:operationId/approve
 */
export function approveVerification(
  operationId: number,
  approved: boolean,
  reason?: string
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(
    `/security/sensitive-audit/${operationId}/approve`,
    { approved, reason }
  );
}

/**
 * @brief 重新发送验证码
 * @description POST /api/security/sensitive-audit/:operationId/resend
 */
export function resendVerifyCode(
  operationId: number
): Promise<HttpResponse<{ verify_code: string }>> {
  return httpClient.post<{ verify_code: string }>(
    `/security/sensitive-audit/${operationId}/resend`
  );
}

/**
 * @brief 获取敏感操作统计
 * @description GET /api/security/sensitive-audit/statistics
 */
export function getSensitiveOperationStatistics(
  params?: { start_date?: string; end_date?: string }
): Promise<HttpResponse<Record<string, unknown>>> {
  return httpClient.get<Record<string, unknown>>(
    '/security/sensitive-audit/statistics',
    { params }
  );
}

/**
 * @brief 获取操作类型列表
 * @description GET /api/security/sensitive-audit/types
 */
export function getOperationTypes(): Promise<HttpResponse<Array<{ type: string; description: string }>>> {
  return httpClient.get<Array<{ type: string; description: string }>>(
    '/security/sensitive-audit/types'
  );
}

/**
 * @brief 获取验证码有效期配置
 * @description GET /api/security/sensitive-audit/expire-time
 */
export function getVerifyCodeExpireTime(): Promise<HttpResponse<{ expire_minutes: number }>> {
  return httpClient.get<{ expire_minutes: number }>(
    '/security/sensitive-audit/expire-time'
  );
}

/**
 * @brief 删除敏感操作记录
 * @description DELETE /api/security/sensitive-audit/:id
 */
export function deleteSensitiveOperation(
  id: number
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(`/security/sensitive-audit/${id}`);
}

/**
 * @brief 批量删除敏感操作记录
 * @description DELETE /api/security/sensitive-audit/batch
 */
export function batchDeleteSensitiveOperations(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>('/security/sensitive-audit/batch', { data: { ids } });
}

// ============ 导出 ============

export const securityService = {
  // IP 白名单
  getIpWhitelist,
  getIpWhitelistDetail,
  createIpWhitelist,
  updateIpWhitelist,
  deleteIpWhitelist,
  batchDeleteIpWhitelist,
  enableIpWhitelist,
  disableIpWhitelist,
  batchEnableIpWhitelist,
  batchDisableIpWhitelist,
  reorderIpWhitelist,
  checkIpWhitelist,
  getIpWhitelistStatistics,
  getIpLocation,

  // 敏感操作审计
  getSensitiveOperations,
  getSensitiveOperationDetail,
  getPendingVerifications,
  initiateVerification,
  submitVerification,
  cancelVerification,
  approveVerification,
  resendVerifyCode,
  getSensitiveOperationStatistics,
  getOperationTypes,
  getVerifyCodeExpireTime,
  deleteSensitiveOperation,
  batchDeleteSensitiveOperations,
};
