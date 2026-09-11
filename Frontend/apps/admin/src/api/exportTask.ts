/**
 * @file exportTask.ts
 * @description 导出任务 API
 * @date 2026-06-18
 * @description 统一使用 httpClient，遵循前后端对接规范
 */

import { httpClient } from '@/utils/alova';
import type { PaginationParams } from '@/utils/alova';

// ============ 类型定义 ============

/** 导出任务状态 */
export type ExportTaskStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'expired';

/** 导出任务格式 */
export type ExportTaskFormat = 'csv' | 'excel' | 'json' | 'pdf';

/** 导出任务 */
export interface ExportTask {
  id: string;
  task_name: string;
  task_type: string;
  status: ExportTaskStatus;
  format: ExportTaskFormat;
  progress: number;
  file_url?: string;
  file_name?: string;
  file_size?: number;
  created_by: string;
  created_at: string;
  completed_at?: string;
  expired_at?: string;
  params?: Record<string, unknown>;
}

/** 导出任务查询参数 */
export interface ExportTaskQuery extends PaginationParams {
  task_type?: string;
  status?: ExportTaskStatus;
  format?: ExportTaskFormat;
  keyword?: string;
  start_date?: string;
  end_date?: string;
  created_by?: string;
}

/** 创建导出任务参数 */
export interface ExportTaskCreateParams {
  task_type: string;
  task_name?: string;
  format?: ExportTaskFormat;
  params?: Record<string, unknown>;
  /** 异步创建时是否立即返回 */
  async?: boolean;
}

/** 批量导出参数 */
export interface BatchExportParams {
  task_types: string[];
  format?: ExportTaskFormat;
  params?: Record<string, unknown>;
}

/** 导出统计 */
export interface ExportTaskStats {
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  pending_tasks: number;
  total_exports: number;
  export_by_type: Record<string, number>;
}

// ============ 导出任务 API ============

/**
 * 获取导出任务数量
 */
export function countExportTasks(params: ExportTaskQuery) {
  return httpClient.get('/export/tasks/count', { params });
}

/**
 * 获取导出任务列表
 */
export function listExportTasks(params: ExportTaskQuery) {
  return httpClient.get('/export/tasks/list', { params });
}

/**
 * 获取导出任务详情
 */
export function getExportTask(taskId: string) {
  return httpClient.get(`/export/tasks/${taskId}`);
}

/**
 * 创建导出任务
 */
export function createExportTask(data: ExportTaskCreateParams) {
  return httpClient.post('/export/tasks/create', data);
}

/**
 * 批量创建导出任务
 */
export function batchCreateExportTasks(data: BatchExportParams) {
  return httpClient.post('/export/tasks/batch-create', data);
}

/**
 * 取消导出任务
 */
export function cancelExportTask(taskId: string) {
  return httpClient.post(`/export/tasks/${taskId}/cancel`);
}

/**
 * 删除导出任务
 */
export function deleteExportTask(taskId: string) {
  return httpClient.delete(`/export/tasks/${taskId}`);
}

/**
 * 下载导出文件
 */
export function downloadExportFile(taskId: string) {
  return httpClient.get(`/export/tasks/${taskId}/download`, {
    responseType: 'blob',
  });
}

/**
 * 获取导出任务统计
 */
export function getExportTaskStats(params?: {
  start_date?: string;
  end_date?: string;
  task_type?: string;
}) {
  return httpClient.get('/export/tasks/stats', { params });
}

/**
 * 获取导出任务进度
 */
export function getExportTaskProgress(taskId: string) {
  return httpClient.get(`/export/tasks/${taskId}/progress`);
}

/**
 * 重试导出任务
 */
export function retryExportTask(taskId: string) {
  return httpClient.post(`/export/tasks/${taskId}/retry`);
}

// ============ 快捷导出 API ============

/**
 * 导出用户列表
 */
export function exportUsers(params?: {
  keyword?: string;
  status?: number;
  role?: string;
  format?: 'csv' | 'excel';
}) {
  return httpClient.get('/export/users', {
    params,
    responseType: 'blob',
  });
}

/**
 * 导出登录日志
 */
export function exportLoginLogs(params?: {
  keyword?: string;
  status?: number;
  start_date?: string;
  end_date?: string;
  format?: 'csv' | 'excel';
}) {
  return httpClient.get('/export/login-logs', {
    params,
    responseType: 'blob',
  });
}

/**
 * 导出操作日志
 */
export function exportOperationLogs(params?: {
  keyword?: string;
  operator?: string;
  module?: string;
  action?: string;
  start_date?: string;
  end_date?: string;
  format?: 'csv' | 'excel';
}) {
  return httpClient.get('/export/operation-logs', {
    params,
    responseType: 'blob',
  });
}

/**
 * 导出审计日志
 */
export function exportAuditLogs(params?: {
  keyword?: string;
  resource_type?: string;
  action?: string;
  start_date?: string;
  end_date?: string;
  format?: 'csv' | 'excel';
}) {
  return httpClient.get('/export/audit-logs', {
    params,
    responseType: 'blob',
  });
}

// ============ 导出 ============

export const exportTaskApi = {
  // 基础导出任务
  count: countExportTasks,
  list: listExportTasks,
  get: getExportTask,
  create: createExportTask,
  batchCreate: batchCreateExportTasks,
  cancel: cancelExportTask,
  delete: deleteExportTask,
  download: downloadExportFile,
  stats: getExportTaskStats,
  progress: getExportTaskProgress,
  retry: retryExportTask,

  // 快捷导出
  users: exportUsers,
  loginLogs: exportLoginLogs,
  operationLogs: exportOperationLogs,
  auditLogs: exportAuditLogs,
};

export default exportTaskApi;
