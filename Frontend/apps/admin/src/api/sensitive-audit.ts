/**
 * @file sensitive-audit.ts
 * @description 敏感操作审计 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

/**
 * @brief 验证方式
 */
export type ConfirmType = 'password' | 'sms' | 'email' | 'authenticator' | 'admin';

/**
 * @brief 验证状态
 */
export type ConfirmStatus = 'pending' | 'success' | 'failed' | 'expired';

/**
 * @brief 操作状态
 */
export type OperationStatus = 'success' | 'failed';

/**
 * @brief 敏感操作记录接口
 */
export interface SensitiveOperation {
  id: number;
  operation_type: string;
  operationDesc?: string;
  user_id: number;
  userName: string;
  confirmType: ConfirmType;
  confirmStatus: ConfirmStatus;
  confirmData?: Record<string, unknown>;
  verifyCode?: string;
  verifyExpireTime?: string;
  targetResource?: string;
  targetResourceName?: string;
  ipAddress?: string;
  operationStatus: OperationStatus;
  failureReason?: string;
  created_at: string;
}

/**
 * @brief 敏感操作查询参数
 */
export interface SensitiveOperationQueryParams {
  operation_type?: string;
  user_id?: number;
  confirmType?: ConfirmType;
  confirmStatus?: ConfirmStatus;
  operationStatus?: OperationStatus;
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
  operationDesc?: string;
  confirmType: ConfirmType;
  targetResource?: string;
  targetResourceName?: string;
}

/**
 * @brief 提交验证
 */
export interface SubmitVerifyParams {
  verifyCode: string;
}

/**
 * @brief 获取敏感操作记录列表
 */
export function getSensitiveOperations(params?: SensitiveOperationQueryParams) {
  return httpClient.get('/security/sensitive-audit', { params });
}

/**
 * @brief 获取敏感操作详情
 */
export function getSensitiveOperationDetail(id: number) {
  return httpClient.get(`/security/sensitive-audit/${id}`);
}

/**
 * @brief 获取当前用户待验证操作
 */
export function getPendingVerifications() {
  return httpClient.get('/security/sensitive-audit/pending');
}

/**
 * @brief 发起敏感操作验证
 */
export function initiateVerification(data: InitiateVerifyParams) {
  return httpClient.post('/security/sensitive-audit/initiate', data);
}

/**
 * @brief 提交验证
 */
export function submitVerification(operationId: number, data: SubmitVerifyParams) {
  return httpClient.post(`/security/sensitive-audit/${operationId}/verify`, data);
}

/**
 * @brief 取消验证
 */
export function cancelVerification(operationId: number) {
  return httpClient.put(`/security/sensitive-audit/${operationId}/cancel`);
}

/**
 * @brief 管理员审批（针对admin确认类型）
 */
export function approveVerification(operationId: number, approved: boolean, reason?: string) {
  return httpClient.put(`/security/sensitive-audit/${operationId}/approve`, { approved, reason });
}

/**
 * @brief 重新发送验证码
 */
export function resendVerifyCode(operationId: number) {
  return httpClient.post(`/security/sensitive-audit/${operationId}/resend`);
}

/**
 * @brief 获取敏感操作统计
 */
export function getSensitiveOperationStatistics(params?: { start_date?: string; end_date?: string }) {
  return httpClient.get('/security/sensitive-audit/statistics', { params });
}

/**
 * @brief 获取操作类型列表
 */
export function getOperationTypes() {
  return httpClient.get('/security/sensitive-audit/types');
}

/**
 * @brief 获取验证码有效期配置
 */
export function getVerifyCodeExpireTime() {
  return httpClient.get('/security/sensitive-audit/expire-time');
}

/**
 * @brief 删除敏感操作记录
 */
export function deleteSensitiveOperation(id: number) {
  return httpClient.delete(`/security/sensitive-audit/${id}`);
}

/**
 * @brief 批量删除敏感操作记录
 */
export function batchDeleteSensitiveOperations(ids: number[]) {
  return httpClient.delete('/security/sensitive-audit/batch', { data: { ids } });
}
