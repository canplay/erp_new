/**
 * @file sensitive-audit.ts
 * @description 敏感操作审计 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

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
export async function getSensitiveOperations(params?: SensitiveOperationQueryParams) {
  try {
    return await httpClient.get('/security/sensitive-audit', { params });
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 获取敏感操作详情
 */
export async function getSensitiveOperationDetail(id: number) {
  try {
    return await httpClient.get(`/security/sensitive-audit/${id}`);
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 获取当前用户待验证操作
 */
export async function getPendingVerifications() {
  try {
    return await httpClient.get('/security/sensitive-audit/pending');
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 发起敏感操作验证
 */
export async function initiateVerification(data: InitiateVerifyParams) {
  try {
    return await httpClient.post('/security/sensitive-audit/initiate', data);
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 提交验证
 */
export async function submitVerification(operationId: number, data: SubmitVerifyParams) {
  try {
    return await httpClient.post(`/security/sensitive-audit/${operationId}/verify`, data);
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 取消验证
 */
export async function cancelVerification(operationId: number) {
  try {
    return await httpClient.put(`/security/sensitive-audit/${operationId}/cancel`);
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 管理员审批（针对admin确认类型）
 */
export async function approveVerification(operationId: number, approved: boolean, reason?: string) {
  try {
    return await httpClient.put(`/security/sensitive-audit/${operationId}/approve`, { approved, reason });
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 重新发送验证码
 */
export async function resendVerifyCode(operationId: number) {
  try {
    return await httpClient.post(`/security/sensitive-audit/${operationId}/resend`);
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 获取敏感操作统计
 */
export async function getSensitiveOperationStatistics(params?: { start_date?: string; end_date?: string }) {
  try {
    return await httpClient.get('/security/sensitive-audit/statistics', { params });
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 获取操作类型列表
 */
export async function getOperationTypes() {
  try {
    return await httpClient.get('/security/sensitive-audit/types');
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 获取验证码有效期配置
 */
export async function getVerifyCodeExpireTime() {
  try {
    return await httpClient.get('/security/sensitive-audit/expire-time');
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 删除敏感操作记录
 */
export async function deleteSensitiveOperation(id: number) {
  try {
    return await httpClient.delete(`/security/sensitive-audit/${id}`);
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}

/**
 * @brief 批量删除敏感操作记录
 */
export async function batchDeleteSensitiveOperations(ids: number[]) {
  try {
    return await httpClient.delete('/security/sensitive-audit/batch', { data: { ids } });
  } catch (error) {
    handleApiError(error, '敏感操作审计');
    throw error;
  }
}
