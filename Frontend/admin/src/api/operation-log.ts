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
} from '@/types/log';
import type { OperationStatsParams } from '@/types';

// ============ API 函数 ============

/**
 * @brief 获取操作日志列表
 */
export function listOperationLogs(params?: {
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
  return httpClient.get('/audit/operation-logs', { params });
}

/**
 * @brief 获取单个操作日志详情
 */
export function getOperationLog(id: number) {
  return httpClient.get(`/audit/operation-logs/${id}`);
}

/**
 * @brief 获取操作统计
 */
export function getOperationStats(params?: OperationStatsParams) {
  return httpClient.get('/audit/operation-stats', { params });
}

/**
 * @brief 导出操作日志
 */
export function exportOperationLogs(params?: {
  keyword?: string;
  username?: string;
  module?: string;
  operation_type?: OperationType;
  start_date?: string;
  end_date?: string;
}) {
  return httpClient.get('/audit/operation-logs/export', {
    params,
    responseType: 'blob',
  });
}

/**
 * @brief 获取审计日志列表
 */
export function listAuditLogs(params?: {
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
  return httpClient.get('/audit/api-call-logs', { params });
}

/**
 * @brief 获取审计日志详情
 */
export function getAuditLog(id: number) {
  return httpClient.get(`/audit/api-call-logs/${id}`);
}

/**
 * @brief 导出审计日志
 */
export function exportAuditLogs(params?: {
  keyword?: string;
  resource_type?: string;
  start_date?: string;
  end_date?: string;
}) {
  return httpClient.get('/audit/api-call-logs/export', {
    params,
    responseType: 'blob',
  });
}

// ============ 类型导出 ============

export type { IOperationLog as OperationLog };
export type { IAuditLog as AuditLog };
export type { OperationType };