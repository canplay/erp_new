/**
 * @file device.ts
 * @description 登录设备管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

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
export async function getMyDevices(params?: Omit<LoginDeviceQueryParams, 'user_id'>) {
  try {
    return await httpClient.get('/devices/my', { params });
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 获取指定用户的登录设备列表（管理员）
 */
export async function getUserDevices(user_id: number, params?: Omit<LoginDeviceQueryParams, 'user_id'>) {
  try {
    return await httpClient.get(`/admin/devices/user/${user_id}`, { params });
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 获取所有用户的登录设备列表（管理员）
 */
export async function getAllDevices(params?: LoginDeviceQueryParams) {
  try {
    return await httpClient.get('/admin/devices', { params });
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 获取设备详情
 */
export async function getDeviceDetail(id: number) {
  try {
    return await httpClient.get(`/devices/${id}`);
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 标记设备为信任设备
 */
export async function trustDevice(id: number) {
  try {
    return await httpClient.put(`/devices/${id}/trust`);
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 取消设备信任
 */
export async function untrustDevice(id: number) {
  try {
    return await httpClient.put(`/devices/${id}/untrust`);
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 踢出设备（强制下线）
 */
export async function kickDevice(id: number) {
  try {
    return await httpClient.put(`/devices/${id}/kick`);
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 踢出用户所有设备
 */
export async function kickAllUserDevices(user_id: number) {
  try {
    return await httpClient.put(`/admin/devices/user/${user_id}/kick-all`);
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 踢出当前用户所有其他设备
 */
export async function kickOtherDevices() {
  try {
    return await httpClient.put('/devices/kick-other');
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 删除设备记录
 */
export async function deleteDevice(id: number) {
  try {
    return await httpClient.delete(`/devices/${id}`);
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 批量删除设备记录
 */
export async function batchDeleteDevices(ids: number[]) {
  try {
    return await httpClient.delete('/devices/batch', { data: { ids } });
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 获取设备统计
 */
export async function getDeviceStatistics(params?: { user_id?: number }) {
  try {
    return await httpClient.get('/admin/devices/statistics', { params });
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 获取登录概况（管理员）
 */
export async function getLoginOverview() {
  try {
    return await httpClient.get('/admin/devices/overview');
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}

/**
 * @brief 获取异常登录记录
 */
export async function getAbnormalLogins(params?: { page?: number; page_size?: number }) {
  try {
    return await httpClient.get('/admin/devices/abnormal', { params });
  } catch (error) {
    handleApiError(error, '设备管理');
    throw error;
  }
}