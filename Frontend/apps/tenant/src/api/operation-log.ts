/**
 * @file operation-log.ts
 * @description 操作日志 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type {
  OperationLog as IOperationLog,
  AuditLog as IAuditLog,
  OperationType,
} from '@erp-new-frontend-monorepo/types/src/log';
import type { OperationStatsParams } from '@erp-new-frontend-monorepo/types';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ API 函数 ============

/**
 * @brief 获取操作日志列表
 */
export async function listOperationLogs(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  username?: string;
  module?: string;
  operation_type?: OperationType;
  success?: boolean;
  start_date?: string;
  end_date?: string;
}) {
  try {
    return await httpClient.get('/audit/operation-logs', { params });
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

/**
 * @brief 获取单个操作日志详情
 */
export async function getOperationLog(id: number) {
  try {
    return await httpClient.get(`/audit/operation-logs/${id}`);
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

/**
 * @brief 获取操作统计
 */
export async function getOperationStats(params?: OperationStatsParams) {
  try {
    return await httpClient.get('/audit/operation-stats', { params });
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

/**
 * @brief 导出操作日志
 */
export async function exportOperationLogs(params?: {
  keyword?: string;
  username?: string;
  module?: string;
  operation_type?: OperationType;
  start_date?: string;
  end_date?: string;
}) {
  try {
    return await httpClient.get('/audit/operation-logs/export', {
    params,
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

/**
 * @brief 获取审计日志列表
 */
export async function listAuditLogs(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  username?: string;
  resource_type?: string;
  action?: string;
  approval_status?: string;
  start_date?: string;
  end_date?: string;
}) {
  try {
    return await httpClient.get('/audit/api-call-logs', { params });
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

/**
 * @brief 获取审计日志详情
 */
export async function getAuditLog(id: number) {
  try {
    return await httpClient.get(`/audit/api-call-logs/${id}`);
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

/**
 * @brief 导出审计日志
 */
export async function exportAuditLogs(params?: {
  keyword?: string;
  resource_type?: string;
  start_date?: string;
  end_date?: string;
}) {
  try {
    return await httpClient.get('/audit/api-call-logs/export', {
    params,
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '操作日志');
    throw error;
  }
}

// ============ 类型导出 ============

export type { IOperationLog as OperationLog };
export type { IAuditLog as AuditLog };
export type { OperationType };