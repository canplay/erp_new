/**
 * @file report.ts
 * @description 报表 API 接口
 * @date 2026-05-21
 * @description 统一使用带泛型的 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse } from '@/types/api';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ 类型定义 ============

/** 报表状态 */
export type ReportStatus = 'draft' | 'published' | 'archived';

/** 报表任务状态 */
export type ReportTaskStatus = 'pending' | 'running' | 'completed' | 'failed';

/** 报表类型 */
export type ReportType = 'table' | 'chart' | 'dashboard';

/** 报表定义 */
export interface Report {
  id: string;
  name: string;
  description?: string;
  reportType: ReportType;
  queryParams: Record<string, unknown>;
  result?: Record<string, unknown>;
  status: ReportStatus;
  created_by: string;
  created_at: string;
  generatedAt?: string;
  updated_at: string;
}

/** 报表任务 */
export interface ReportTask {
  id: string;
  reportId: string;
  status: ReportTaskStatus;
  result?: Record<string, unknown>;
  error_message?: string;
  startedAt: string;
  completed_at?: string;
}

// ============ 请求参数 ============

/** 创建报表参数 */
export interface CreateReportParam {
  name: string;
  reportType: ReportType;
  queryParams?: Record<string, unknown>;
  description?: string;
}

/** 更新报表参数 */
export interface UpdateReportParam {
  name?: string;
  description?: string;
  queryParams?: Record<string, unknown>;
  status?: ReportStatus;
}

/** 列表查询参数 */
export interface ReportListParam {
  page?: number;
  page_size?: number;
  keyword?: string;
  reportType?: ReportType;
  status?: ReportStatus;
}

// ============ API 函数 ============

const baseUrl = '/reports';

/**
 * @brief 获取报表列表
 */
export function listReports(params?: ReportListParam) {
  try {
    return await httpClient.get(`${baseUrl}`, { params });
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 获取报表详情
 */
export function getReport(id: string) {
  try {
    return await httpClient.get(`${baseUrl}/${id}`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 创建报表
 */
export function createReport(data: CreateReportParam) {
  try {
    return await httpClient.post(`${baseUrl}`, data);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 更新报表
 */
export function updateReport(id: string, data: UpdateReportParam) {
  try {
    return await httpClient.put(`${baseUrl}/${id}`, data);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 删除报表
 */
export function deleteReport(id: string) {
  try {
    return await httpClient.delete(`${baseUrl}/${id}`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 生成报表
 */
export function generateReport(id: string) {
  try {
    return await httpClient.post(`${baseUrl}/${id}/generate`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 下载报表
 */
export function downloadReport(id: string) {
  try {
    return await httpClient.get(`${baseUrl}/${id}/download`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

// ============ 数据源相关类型 ============

/** 数据源类型 */
export type DataSourceType = 'mysql' | 'postgresql' | 'http' | 'file';

/** 数据源 */
export interface DataSource {
  id: string;
  name: string;
  dsType: DataSourceType;
  config: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

/** 报表模板 */
export interface ReportTemplate {
  id: string;
  name: string;
  reportType: ReportType;
  description?: string;
  queryParams: Record<string, unknown>;
}

// ============ 数据源 API ============

const dataSourceBaseUrl = '/data-sources';

/**
 * @brief 获取数据源列表
 */
export function listDataSources() {
  try {
    return await httpClient.get(`${dataSourceBaseUrl}`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 创建数据源
 */
export function createDataSource(data: Partial<DataSource>) {
  try {
    return await httpClient.post(`${dataSourceBaseUrl}`, data);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 更新数据源
 */
export function updateDataSource(id: string, data: Partial<DataSource>) {
  try {
    return await httpClient.put(`${dataSourceBaseUrl}/${id}`, data);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 删除数据源
 */
export function deleteDataSource(id: string) {
  try {
    return await httpClient.delete(`${dataSourceBaseUrl}/${id}`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

// ============ 报表模板 API ============

const templateBaseUrl = '/report-templates';

/**
 * @brief 获取报表模板列表
 */
export function listTemplates() {
  try {
    return await httpClient.get(`${templateBaseUrl}`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 从模板创建报表
 */
export function createFromTemplate(templateId: string, name: string) {
  try {
    return await httpClient.post(`${templateBaseUrl}/${templateId}/create`, { name });
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

// ============ 报表执行 API ============

/**
 * @brief 执行报表
 */
export function executeReport(id: string) {
  try {
    return await httpClient.post(`${baseUrl}/${id}/execute`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 导出报表
 */
export function exportReport(id: string, format?: 'pdf' | 'excel' | 'csv') {
  try {
    return await httpClient.get(`${baseUrl}/${id}/export`, {
    responseType: 'blob',
    params: format ? { format } : undefined,  });
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 获取报表数据
 */
export function getReportData(id: string) {
  try {
    return await httpClient.get(`${baseUrl}/${id}/data`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 获取报表任务列表
 */
export function listReportTasks(reportId: string) {
  try {
    return await httpClient.get(`${baseUrl}/${reportId}/tasks`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}

/**
 * @brief 获取报表历史记录
 */
export function listReportHistory(reportId: string) {
  try {
    return await httpClient.get(`${baseUrl}/${reportId}/history`);
  } catch (error) {
    handleApiError(error, '报表管理');
    throw error;
  }
}