/**
 * @file ip-whitelist.ts
 * @description IP白名单 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

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
export function getIpWhitelist(params?: IpWhitelistQueryParams) {
  return httpClient.get('/security/ip-whitelist', { params });
}

/**
 * @brief 获取IP白名单规则详情
 */
export function getIpWhitelistDetail(id: number) {
  return httpClient.get(`/security/ip-whitelist/${id}`);
}

/**
 * @brief 创建IP白名单规则
 */
export function createIpWhitelist(data: IpWhitelistCreateParams) {
  return httpClient.post('/security/ip-whitelist', data);
}

/**
 * @brief 更新IP白名单规则
 */
export function updateIpWhitelist(id: number, data: IpWhitelistUpdateParams) {
  return httpClient.put(`/security/ip-whitelist/${id}`, data);
}

/**
 * @brief 删除IP白名单规则
 */
export function deleteIpWhitelist(id: number) {
  return httpClient.delete(`/security/ip-whitelist/${id}`);
}

/**
 * @brief 批量删除IP白名单规则
 */
export function batchDeleteIpWhitelist(ids: number[]) {
  return httpClient.delete('/security/ip-whitelist/batch', { data: { ids } });
}

/**
 * @brief 启用IP白名单规则
 */
export function enableIpWhitelist(id: number) {
  return httpClient.put(`/security/ip-whitelist/${id}/enable`);
}

/**
 * @brief 禁用IP白名单规则
 */
export function disableIpWhitelist(id: number) {
  return httpClient.put(`/security/ip-whitelist/${id}/disable`);
}

/**
 * @brief 批量启用规则
 */
export function batchEnableIpWhitelist(ids: number[]) {
  return httpClient.put('/security/ip-whitelist/batch-enable', { ids });
}

/**
 * @brief 批量禁用规则
 */
export function batchDisableIpWhitelist(ids: number[]) {
  return httpClient.put('/security/ip-whitelist/batch-disable', { ids });
}

/**
 * @brief 调整规则优先级
 */
export function reorderIpWhitelist(orders: Array<{ id: number; priority: number }>) {
  return httpClient.put('/security/ip-whitelist/reorder', { orders });
}

/**
 * @brief 验证IP是否在白名单内
 */
export function checkIpWhitelist(ip: string) {
  return httpClient.get('/security/ip-whitelist/check', { params: { ip } });
}

/**
 * @brief 获取IP白名单统计
 */
export function getIpWhitelistStatistics() {
  return httpClient.get('/security/ip-whitelist/statistics');
}

/**
 * @brief 获取IP归属地信息
 */
export function getIpLocation(ip: string) {
  return httpClient.get('/security/ip-whitelist/location', { params: { ip } });
}
