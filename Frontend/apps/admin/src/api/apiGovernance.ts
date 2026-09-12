/**
 * @file apiGovernance.ts
 * @description API 治理相关接口
 * @date 2026-05-28
 */

import { httpClient } from '@/utils/alova';
import type {
import { handleApiError } from '@/utils/apiErrorHandler';
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
  try {
    return await httpClient.get('/audit/api-call-logs', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 获取 API 调用统计数据
 */
export async function getApiCallStatistics(params?: GetApiStatisticsParams) {
  try {
    return await httpClient.get('/audit/api-call-statistics', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 获取 API 端点统计数据
 */
export async function getApiEndpointStatistics(params?: GetApiStatisticsParams) {
  try {
    return await httpClient.get('/audit/api-endpoint-statistics', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 获取 API 调用趋势数据
 */
export async function getApiTrend(params?: GetApiStatisticsParams) {
  try {
    return await httpClient.get('/audit/api-trend', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 获取响应时间分布
 */
export async function getApiResponseDistribution(params?: GetApiStatisticsParams) {
  try {
    return await httpClient.get('/audit/api-response-distribution', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 获取 API 分类统计
 */
export async function getApiCategoryStatistics(params?: GetApiStatisticsParams) {
  try {
    return await httpClient.get('/audit/api-category-statistics', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 获取性能基线数据
 */
export async function getApiPerformanceBaseline(params?: GetApiStatisticsParams) {
  try {
    return await httpClient.get('/audit/api-performance-baseline', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}

/**
 * @brief 导出 API 日志
 */
export async function exportApiLogs(params?: ListApiLogsParams) {
  try {
    return await httpClient.get('/audit/api-call-logs/export', { params });
  } catch (error) {
    handleApiError(error, 'API治理');
    throw error;
  }
}
