/**
 * @file ip-whitelist.ts
 * @description IP白名单 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

/**
 * @brief IP类型
 */
export type IpType = 'ip' | 'cidr' | 'range';

/**
 * @brief 生效对象类型
 */
export type IpWhitelistTargetType = 'all' | 'role' | 'user';

/**
 * @brief IP白名单规则接口
 */
export interface IpWhitelistRule {
  id: number;
  name: string;
  ipType: IpType;
  ipStart: string;
  ipEnd?: string;
  mask?: string;
  targetType: IpWhitelistTargetType;
  target_ids?: number[];
  effectStartTime?: string;
  effectEndTime?: string;
  description?: string;
  priority: number;
  status: number;
  created_by?: number;
  created_at: string;
  updated_at: string;
}

/**
 * @brief IP白名单查询参数
 */
export interface IpWhitelistQueryParams {
  ipType?: IpType;
  targetType?: IpWhitelistTargetType;
  status?: number;
  keyword?: string;
  page?: number;
  page_size?: number;
}

/**
 * @brief IP白名单创建参数
 */
export interface IpWhitelistCreateParams {
  name: string;
  ipType: IpType;
  ipStart: string;
  ipEnd?: string;
  mask?: string;
  targetType: IpWhitelistTargetType;
  target_ids?: number[];
  effectStartTime?: string;
  effectEndTime?: string;
  description?: string;
  priority?: number;
  status?: number;
}

/**
 * @brief IP白名单更新参数
 */
export interface IpWhitelistUpdateParams extends IpWhitelistCreateParams {
  id: number;
}

/**
 * @brief 获取IP白名单规则列表
 */
export async function getIpWhitelist(params?: IpWhitelistQueryParams) {
  try {
    return await httpClient.get('/security/ip-whitelist', { params });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 获取IP白名单规则详情
 */
export async function getIpWhitelistDetail(id: number) {
  try {
    return await httpClient.get(`/security/ip-whitelist/${id}`);
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 创建IP白名单规则
 */
export async function createIpWhitelist(data: IpWhitelistCreateParams) {
  try {
    return await httpClient.post('/security/ip-whitelist', data);
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 更新IP白名单规则
 */
export async function updateIpWhitelist(id: number, data: IpWhitelistUpdateParams) {
  try {
    return await httpClient.put(`/security/ip-whitelist/${id}`, data);
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 删除IP白名单规则
 */
export async function deleteIpWhitelist(id: number) {
  try {
    return await httpClient.delete(`/security/ip-whitelist/${id}`);
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 批量删除IP白名单规则
 */
export async function batchDeleteIpWhitelist(ids: number[]) {
  try {
    return await httpClient.delete('/security/ip-whitelist/batch', { data: { ids } });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 启用IP白名单规则
 */
export async function enableIpWhitelist(id: number) {
  try {
    return await httpClient.put(`/security/ip-whitelist/${id}/enable`);
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 禁用IP白名单规则
 */
export async function disableIpWhitelist(id: number) {
  try {
    return await httpClient.put(`/security/ip-whitelist/${id}/disable`);
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 批量启用规则
 */
export async function batchEnableIpWhitelist(ids: number[]) {
  try {
    return await httpClient.put('/security/ip-whitelist/batch-enable', { ids });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 批量禁用规则
 */
export async function batchDisableIpWhitelist(ids: number[]) {
  try {
    return await httpClient.put('/security/ip-whitelist/batch-disable', { ids });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 调整规则优先级
 */
export async function reorderIpWhitelist(orders: Array<{ id: number; priority: number }>) {
  try {
    return await httpClient.put('/security/ip-whitelist/reorder', { orders });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 验证IP是否在白名单内
 */
export async function checkIpWhitelist(ip: string) {
  try {
    return await httpClient.get('/security/ip-whitelist/check', { params: { ip } });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 获取IP白名单统计
 */
export async function getIpWhitelistStatistics() {
  try {
    return await httpClient.get('/security/ip-whitelist/statistics');
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}

/**
 * @brief 获取IP归属地信息
 */
export async function getIpLocation(ip: string) {
  try {
    return await httpClient.get('/security/ip-whitelist/location', { params: { ip } });
  } catch (error) {
    handleApiError(error, 'IP白名单');
    throw error;
  }
}
