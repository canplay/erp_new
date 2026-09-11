/**
 * @file apiGovernance.ts
 * @description API 治理相关接口
 * @date 2026-05-28
 */

import { httpClient } from '@/utils/alova';
import type {
  ApiLogEntry,
  ApiCallStatistics,
  ApiEndpointStatistics,
  ApiTrendPoint,
  ApiResponseTimeDistribution,
  ApiPerformanceBaseline,
  ApiCategoryStatistics,
} from '@/types/apiGovernance';

// ============ 请求参数类型 ============

export interface ListApiLogsParams {
  page?: number;
  page_size?: number;
  method?: string;
  pathKeyword?: string;
  status_code?: number;
  min_response_time?: number;
  max_response_time?: number;
  start_time?: string;
  end_time?: string;
  errorsOnly?: boolean;
}

export interface GetApiStatisticsParams {
  start_time?: string;
  end_time?: string;
  granularity?: 'hour' | 'day' | 'week';
}

// ============ API 函数 ============

/**
 * @brief 获取 API 调用日志列表
 */
export async function listApiLogs(params?: ListApiLogsParams) {
  return httpClient.get<{
    list: ApiLogEntry[];
    total: number;
  }>('/audit/api-call-logs', { params });
}

/**
 * @brief 获取 API 调用统计数据
 */
export async function getApiCallStatistics(params?: GetApiStatisticsParams) {
  return httpClient.get<ApiCallStatistics>('/audit/api-call-statistics', { params });
}

/**
 * @brief 获取 API 端点统计数据
 */
export async function getApiEndpointStatistics(params?: GetApiStatisticsParams) {
  return httpClient.get<ApiEndpointStatistics[]>('/audit/api-endpoint-statistics', { params });
}

/**
 * @brief 获取 API 调用趋势数据
 */
export async function getApiTrend(params?: GetApiStatisticsParams) {
  return httpClient.get<ApiTrendPoint[]>('/audit/api-trend', { params });
}

/**
 * @brief 获取响应时间分布
 */
export async function getApiResponseDistribution(params?: GetApiStatisticsParams) {
  return httpClient.get<ApiResponseTimeDistribution[]>('/audit/api-response-distribution', { params });
}

/**
 * @brief 获取 API 分类统计
 */
export async function getApiCategoryStatistics(params?: GetApiStatisticsParams) {
  return httpClient.get<ApiCategoryStatistics[]>('/audit/api-category-statistics', { params });
}

/**
 * @brief 获取性能基线数据
 */
export async function getApiPerformanceBaseline(params?: GetApiStatisticsParams) {
  return httpClient.get<ApiPerformanceBaseline[]>('/audit/api-performance-baseline', { params });
}

/**
 * @brief 导出 API 日志
 */
export async function exportApiLogs(params?: ListApiLogsParams) {
  return httpClient.get<Blob>('/audit/api-call-logs/export', { params });
}
