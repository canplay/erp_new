/**
 * @file device.ts
 * @description 登录设备管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

/**
 * @brief 设备类型
 */
export type DeviceType = 'desktop' | 'mobile' | 'tablet' | 'other';

/**
 * @brief 登录设备接口
 */
export interface LoginDevice {
  id: number;
  user_id: number;
  deviceId: string;
  deviceType: DeviceType;
  deviceName?: string;
  browser?: string;
  os?: string;
  ipAddress: string;
  ipLocation?: string;
  login_time: string;
  lastActiveTime?: string;
  logoutTime?: string;
  isActive: boolean;
  isTrusted: boolean;
  user_agent?: string;
  created_at: string;
}

/**
 * @brief 异常登录记录接口（扩展自 LoginDevice）
 */
export interface AbnormalLoginRecord extends LoginDevice {
  userName?: string;
  riskLevel?: string;
  loginType?: string;
  reason?: string;
}

/**
 * @brief 登录设备查询参数
 */
export interface LoginDeviceQueryParams {
  user_id?: number;
  deviceType?: DeviceType;
  isActive?: boolean;
  isTrusted?: boolean;
  keyword?: string;
  start_date?: string;
  end_date?: string;
  page?: number;
  page_size?: number;
}

/**
 * @brief 获取当前用户的登录设备列表
 */
export function getMyDevices(params?: Omit<LoginDeviceQueryParams, 'user_id'>) {
  return httpClient.get('/devices/my', { params });
}

/**
 * @brief 获取指定用户的登录设备列表（管理员）
 */
export function getUserDevices(user_id: number, params?: Omit<LoginDeviceQueryParams, 'user_id'>) {
  return httpClient.get(`/admin/devices/user/${user_id}`, { params });
}

/**
 * @brief 获取所有用户的登录设备列表（管理员）
 */
export function getAllDevices(params?: LoginDeviceQueryParams) {
  return httpClient.get('/admin/devices', { params });
}

/**
 * @brief 获取设备详情
 */
export function getDeviceDetail(id: number) {
  return httpClient.get(`/devices/${id}`);
}

/**
 * @brief 标记设备为信任设备
 */
export function trustDevice(id: number) {
  return httpClient.put(`/devices/${id}/trust`);
}

/**
 * @brief 取消设备信任
 */
export function untrustDevice(id: number) {
  return httpClient.put(`/devices/${id}/untrust`);
}

/**
 * @brief 踢出设备（强制下线）
 */
export function kickDevice(id: number) {
  return httpClient.put(`/devices/${id}/kick`);
}

/**
 * @brief 踢出用户所有设备
 */
export function kickAllUserDevices(user_id: number) {
  return httpClient.put(`/admin/devices/user/${user_id}/kick-all`);
}

/**
 * @brief 踢出当前用户所有其他设备
 */
export function kickOtherDevices() {
  return httpClient.put('/devices/kick-other');
}

/**
 * @brief 删除设备记录
 */
export function deleteDevice(id: number) {
  return httpClient.delete(`/devices/${id}`);
}

/**
 * @brief 批量删除设备记录
 */
export function batchDeleteDevices(ids: number[]) {
  return httpClient.delete('/devices/batch', { data: { ids } });
}

/**
 * @brief 获取设备统计
 */
export function getDeviceStatistics(params?: { user_id?: number }) {
  return httpClient.get('/admin/devices/statistics', { params });
}

/**
 * @brief 获取登录概况（管理员）
 */
export function getLoginOverview() {
  return httpClient.get('/admin/devices/overview');
}

/**
 * @brief 获取异常登录记录
 */
export function getAbnormalLogins(params?: { page?: number; page_size?: number }) {
  return httpClient.get('/admin/devices/abnormal', { params });
}