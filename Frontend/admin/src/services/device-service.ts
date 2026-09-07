/**
 * @file device-service.ts
 * @brief 设备服务（device-service）
 * @date 2026-06-18
 * @description 对接 Backend/Rust device-service
 */

import { httpClient } from '@/utils/alova';
import type { HttpResponse } from '@/utils/alova';

// ============ 类型定义 ============

/**
 * @brief 设备状态
 */
export type DeviceStatus = 'online' | 'offline' | 'idle' | 'maintenance' | 'error';

/**
 * @brief 设备类型
 */
export type DeviceType = 'vehicle' | 'charger' | 'lock' | 'sensor' | 'gateway' | 'other';

/**
 * @brief 设备信息
 */
export interface Device {
  id: number;
  device_code: string;
  device_name: string;
  device_type: DeviceType;
  status: DeviceStatus;
  manufacturer?: string;
  model?: string;
  firmware_version?: string;
  hardware_version?: string;
  imei?: string;
  iccid?: string;
  sim_card?: string;
  battery_level?: number;
  signal_strength?: number;
  latitude?: number;
  longitude?: number;
  address?: string;
  last_online_time?: string;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 设备查询参数
 */
export interface DeviceQueryParams {
  page?: number;
  page_size?: number;
  keyword?: string;
  device_type?: DeviceType;
  status?: DeviceStatus;
  manufacturer?: string;
  department_id?: number;
  start_date?: string;
  end_date?: string;
}

/**
 * @brief 设备创建参数
 */
export interface DeviceCreateParams {
  device_code: string;
  device_name: string;
  device_type: DeviceType;
  manufacturer?: string;
  model?: string;
  imei?: string;
  iccid?: string;
  sim_card?: string;
  department_id?: number;
}

/**
 * @brief 设备更新参数
 */
export interface DeviceUpdateParams extends Partial<DeviceCreateParams> {
  status?: DeviceStatus;
  firmware_version?: string;
  hardware_version?: string;
  battery_level?: number;
  signal_strength?: number;
  latitude?: number;
  longitude?: number;
  address?: string;
}

/**
 * @brief 设备统计信息
 */
export interface DeviceStatistics {
  total: number;
  online: number;
  offline: number;
  idle: number;
  maintenance: number;
  error: number;
  by_type: Array<{ type: DeviceType; count: number }>;
  by_manufacturer: Array<{ manufacturer: string; count: number }>;
}

/**
 * @brief CTP 设备消息
 */
export interface CtpDeviceMessage {
  device_code: string;
  message_type: string;
  data: Record<string, unknown>;
  received_at: string;
}

// ============ 设备管理 API ============

/**
 * @brief 获取设备列表
 * @description GET /api/devices
 */
export function getDevices(
  params?: DeviceQueryParams
): Promise<HttpResponse<Device[]>> {
  return httpClient.get<Device[]>('/devices', { params })
}

/**
 * @brief 获取所有设备（不分页）
 * @description GET /api/devices/all
 */
export function getAllDevices(): Promise<HttpResponse<Device[]>> {
  return httpClient.get<Device[]>('/devices/all')
}

/**
 * @brief 获取单个设备详情
 * @description GET /api/devices/:id
 */
export function getDeviceDetail(
  id: number
): Promise<HttpResponse<Device>> {
  return httpClient.get<Device>(`/devices/${id}`)
}

/**
 * @brief 根据设备编码获取设备
 * @description GET /api/devices/code/:deviceCode
 */
export function getDeviceByCode(
  deviceCode: string
): Promise<HttpResponse<Device>> {
  return httpClient.get<Device>(`/devices/code/${deviceCode}`)
}

/**
 * @brief 创建设备
 * @description POST /api/devices
 */
export function createDevice(
  data: DeviceCreateParams
): Promise<HttpResponse<Device>> {
  return httpClient.post<Device>('/devices', data)
}

/**
 * @brief 批量创建设备
 * @description POST /api/devices/batch
 */
export function batchCreateDevices(
  devices: DeviceCreateParams[]
): Promise<HttpResponse<{ created: number }>> {
  return httpClient.post<{ created: number }>('/devices/batch', { devices })
}

/**
 * @brief 更新设备
 * @description PUT /api/devices/:id
 */
export function updateDevice(
  id: number,
  data: DeviceUpdateParams
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/devices/${id}`, data)
}

/**
 * @brief 批量更新设备状态
 * @description PUT /api/devices/batch-status
 */
export function batchUpdateDeviceStatus(
  ids: number[],
  status: DeviceStatus
): Promise<HttpResponse<void>> {
  return httpClient.put<void>('/devices/batch-status', { ids, status })
}

/**
 * @brief 删除设备
 * @description DELETE /api/devices/:id
 */
export function deleteDevice(id: number): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(`/devices/${id}`)
}

/**
 * @brief 批量删除设备
 * @description DELETE /api/devices/batch
 */
export function batchDeleteDevices(
  ids: number[]
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>('/devices/batch', { data: { ids } })
}

/**
 * @brief 更新设备状态
 * @description PUT /api/devices/:id/status
 */
export function updateDeviceStatus(
  id: number,
  status: DeviceStatus
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/devices/${id}/status`, { status })
}

/**
 * @brief 重启设备
 * @description POST /api/devices/:id/restart
 */
export function restartDevice(id: number): Promise<HttpResponse<void>> {
  return httpClient.post<void>(`/devices/${id}/restart`)
}

/**
 * @brief 升级设备固件
 * @description POST /api/devices/:id/upgrade
 */
export function upgradeDeviceFirmware(
  id: number,
  firmwareUrl: string
): Promise<HttpResponse<{ task_id: string }>> {
  return httpClient.post<{ task_id: string }>(
    `/devices/${id}/upgrade`,
    { firmware_url: firmwareUrl }
  );
}

// ============ 设备统计 API ============

/**
 * @brief 获取设备统计信息
 * @description GET /api/devices/statistics
 */
export function getDeviceStatistics(): Promise<HttpResponse<DeviceStatistics>> {
  return httpClient.get<DeviceStatistics>('/devices/statistics')
}

/**
 * @brief 获取设备在线状态
 * @description GET /api/devices/online-status
 */
export function getDeviceOnlineStatus(
  deviceIds: number[]
): Promise<HttpResponse<Array<{ device_id: number; online: boolean }>>> {
  return httpClient.post<Array<{ device_id: number; online: boolean }>>(
    '/devices/online-status',
    { device_ids: deviceIds }
  );
}

/**
 * @brief 获取设备位置信息
 * @description GET /api/devices/:id/location
 */
export function getDeviceLocation(
  id: number
): Promise<HttpResponse<{ latitude: number; longitude: number; address?: string; updated_at: string }>> {
  return httpClient.get<{ latitude: number; longitude: number; address?: string; updated_at: string }>(
    `/devices/${id}/location`
  );
}

// ============ CTP 协议 API ============

/**
 * @brief 获取 CTP 设备消息列表
 * @description GET /api/devices/ctp/messages
 */
export function getCtpMessages(
  params?: {
    device_code?: string;
    message_type?: string;
    page?: number;
    page_size?: number;
  }
): Promise<HttpResponse<CtpDeviceMessage[]>> {
  return httpClient.get<CtpDeviceMessage[]>('/devices/ctp/messages', { params })
}

/**
 * @brief 向 CTP 设备发送指令
 * @description POST /api/devices/ctp/command
 */
export function sendCtpCommand(
  deviceCode: string,
  command: {
    type: string;
    data: Record<string, unknown>;
  }
): Promise<HttpResponse<{ task_id: string }>> {
  return httpClient.post<{ task_id: string }>(
    `/devices/ctp/command/${deviceCode}`,
    command
  );
}

/**
 * @brief 获取 CTP 设备连接状态
 * @description GET /api/devices/ctp/status
 */
export function getCtpDeviceStatus(
  deviceCode: string
): Promise<HttpResponse<{
  connected: boolean;
  last_heartbeat: string;
  protocol_version: string;
}>> {
  return httpClient.get<{
    connected: boolean;
    last_heartbeat: string;
    protocol_version: string;
  }>(`/devices/ctp/status/${deviceCode}`);
}

// ============ 设备分组 API ============

/**
 * @brief 获取设备分组列表
 * @description GET /api/devices/groups
 */
export function getDeviceGroups(): Promise<HttpResponse<Array<{
  id: number;
  name: string;
  description?: string;
  device_count: number;
}>>> {
  return httpClient.get<Array<{
    id: number;
    name: string;
    description?: string;
    device_count: number;
  }>>('/devices/groups');
}

/**
 * @brief 创建设备分组
 * @description POST /api/devices/groups
 */
export function createDeviceGroup(
  data: { name: string; description?: string }
): Promise<HttpResponse<{ id: number }>> {
  return httpClient.post<{ id: number }>('/devices/groups', data)
}

/**
 * @brief 向分组添加设备
 * @description POST /api/devices/groups/:groupId/devices
 */
export function addDevicesToGroup(
  groupId: number,
  deviceIds: number[]
): Promise<HttpResponse<void>> {
  return httpClient.post<void>(`/devices/groups/${groupId}/devices`, { device_ids: deviceIds })
}

/**
 * @brief 从分组移除设备
 * @description DELETE /api/devices/groups/:groupId/devices
 */
export function removeDevicesFromGroup(
  groupId: number,
  deviceIds: number[]
): Promise<HttpResponse<void>> {
  return httpClient.delete<void>(`/devices/groups/${groupId}/devices`, {
    data: { device_ids: deviceIds },
  });
}

// ============ 导出 ============

export const deviceService = {
  // 设备管理
  getDevices,
  getAllDevices,
  getDeviceDetail,
  getDeviceByCode,
  createDevice,
  batchCreateDevices,
  updateDevice,
  batchUpdateDeviceStatus,
  deleteDevice,
  batchDeleteDevices,
  updateDeviceStatus,
  restartDevice,
  upgradeDeviceFirmware,

  // 设备统计
  getDeviceStatistics,
  getDeviceOnlineStatus,
  getDeviceLocation,

  // CTP 协议
  getCtpMessages,
  sendCtpCommand,
  getCtpDeviceStatus,

  // 设备分组
  getDeviceGroups,
  createDeviceGroup,
  addDevicesToGroup,
  removeDevicesFromGroup,
};
