/**
 * @file log.ts
 * @brief 日志管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse, PaginationParams, PaginationResponse } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

/**
 * @brief 登录日志
 */
export interface LoginLog {
  id: number;
  user_id: number;
  username: string;
  status: number; // 1: 成功, 0: 失败
  ip?: string;
  location?: string;
  user_agent?: string;
  error_msg?: string;
  created_at: number;
}

/**
 * @brief 操作日志
 */
export interface OperationLog {
  id: number;
  operator: string;
  module: string;
  action: string;
  resource?: string;
  ip?: string;
  request_params?: string;
  response_result?: string;
  created_at: number;
}

/**
 * @brief 登录日志查询参数
 */
export interface ListLoginLogsParams extends PaginationParams {
  keyword?: string;
  status?: number;
  start_date?: string;
  end_date?: string;
}

/**
 * @brief 操作日志查询参数
 */
export interface ListOperationLogsParams extends PaginationParams {
  keyword?: string;
  operator?: string;
  module?: string;
  action?: string;
  start_date?: string;
  end_date?: string;
}

/**
 * @brief 获取登录日志列表
 */
export function listLoginLogs(params?: ListLoginLogsParams) {
  try {
    return await httpClient.get('/audit/login-logs', { params });
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 获取登录日志详情
 */
export function getLoginLog(id: number) {
  try {
    return await httpClient.get(`/audit/login-logs/${id}`);
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 导出登录日志
 */
export function exportLoginLogs(params?: Omit<ListLoginLogsParams, 'page' | 'page_size'>) {
  try {
    return await httpClient.get('/audit/login-logs/export', {
    params,
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 获取操作日志列表
 */
export function listOperationLogs(params?: ListOperationLogsParams) {
  try {
    return await httpClient.get('/audit/operation-logs', { params });
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 获取操作日志详情
 */
export function getOperationLog(id: number) {
  try {
    return await httpClient.get(`/audit/operation-logs/${id}`);
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 导出操作日志
 */
export function exportOperationLogs(params?: Omit<ListOperationLogsParams, 'page' | 'page_size'>) {
  try {
    return await httpClient.get('/audit/operation-logs/export', {
    params,
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 清理登录日志
 */
export function clearLoginLogs(days?: number) {
  try {
    return await httpClient.delete('/audit/login-logs', {
    params: { days },  });
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}

/**
 * @brief 清理操作日志
 */
export function clearOperationLogs(days?: number) {
  try {
    return await httpClient.delete('/audit/operation-logs', {
    params: { days },  });
  } catch (error) {
    handleApiError(error, '日志管理');
    throw error;
  }
}
