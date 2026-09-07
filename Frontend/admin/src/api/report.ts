/**
 * @file report.ts
 * @description 报表 API 接口
 * @date 2026-05-21
 * @description 统一使用带泛型的 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse } from '@/types/api';

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
  return httpClient.get<ApiResponse<{ list: Report[]; total: number }>>(`${baseUrl}`, { params });
}

/**
 * @brief 获取报表详情
 */
export function getReport(id: string) {
  return httpClient.get<ApiResponse<Report>>(`${baseUrl}/${id}`);
}

/**
 * @brief 创建报表
 */
export function createReport(data: CreateReportParam) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${baseUrl}`, data);
}

/**
 * @brief 更新报表
 */
export function updateReport(id: string, data: UpdateReportParam) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${id}`, data);
}

/**
 * @brief 删除报表
 */
export function deleteReport(id: string) {
  return httpClient.delete<ApiResponse<void>>(`${baseUrl}/${id}`);
}

/**
 * @brief 生成报表
 */
export function generateReport(id: string) {
  return httpClient.post<ApiResponse<void>>(`${baseUrl}/${id}/generate`);
}

/**
 * @brief 下载报表
 */
export function downloadReport(id: string) {
  return httpClient.get<ApiResponse<string>>(`${baseUrl}/${id}/download`);
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
  return httpClient.get<ApiResponse<DataSource[]>>(`${dataSourceBaseUrl}`);
}

/**
 * @brief 创建数据源
 */
export function createDataSource(data: Partial<DataSource>) {
  return httpClient.post<ApiResponse<DataSource>>(`${dataSourceBaseUrl}`, data);
}

/**
 * @brief 更新数据源
 */
export function updateDataSource(id: string, data: Partial<DataSource>) {
  return httpClient.put<ApiResponse<void>>(`${dataSourceBaseUrl}/${id}`, data);
}

/**
 * @brief 删除数据源
 */
export function deleteDataSource(id: string) {
  return httpClient.delete<ApiResponse<void>>(`${dataSourceBaseUrl}/${id}`);
}

// ============ 报表模板 API ============

const templateBaseUrl = '/report-templates';

/**
 * @brief 获取报表模板列表
 */
export function listTemplates() {
  return httpClient.get<ApiResponse<ReportTemplate[]>>(`${templateBaseUrl}`);
}

/**
 * @brief 从模板创建报表
 */
export function createFromTemplate(templateId: string, name: string) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${templateBaseUrl}/${templateId}/create`, { name });
}

// ============ 报表执行 API ============

/**
 * @brief 执行报表
 */
export function executeReport(id: string) {
  return httpClient.post<ApiResponse<void>>(`${baseUrl}/${id}/execute`);
}

/**
 * @brief 导出报表
 */
export function exportReport(id: string, format?: 'pdf' | 'excel' | 'csv') {
  return httpClient.get(`${baseUrl}/${id}/export`, {
    responseType: 'blob',
    params: format ? { format } : undefined,
  });
}

/**
 * @brief 获取报表数据
 */
export function getReportData(id: string) {
  return httpClient.get<ApiResponse<Record<string, unknown>>>(`${baseUrl}/${id}/data`);
}

/**
 * @brief 获取报表任务列表
 */
export function listReportTasks(reportId: string) {
  return httpClient.get<ApiResponse<ReportTask[]>>(`${baseUrl}/${reportId}/tasks`);
}

/**
 * @brief 获取报表历史记录
 */
export function listReportHistory(reportId: string) {
  return httpClient.get<ApiResponse<ReportTask[]>>(`${baseUrl}/${reportId}/history`);
}