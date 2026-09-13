/**
 * @file ctp.ts
 * @description CTP 平板锁设备管理 API
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

// ==================== 类型定义 ====================

/** CTP 设备数据类型枚举 */
export type CtpDataType = 0 | 1 | 2 | 5 | 6 | 7 | 8;
// 0=降板 1=升板 2=心跳 5=入位 6=出位 7=复位 8=逃费

/** 锁状态 */
export type LockStatus = 'Locked' | 'Unlocked' | 'Offline' | 'Fault';

/** 锁控制命令 */
export type CmdType = 'up' | 'down' | 'syn';

/** 状态字解析结果 */
export interface StatusParsed {
  lock_state: string;
  left_coil: string;
  right_coil: string;
  battery_raw: number;
  reset_flag: boolean;
  heartbeat: boolean;
  alarm: boolean;
}

/** CTP 设备 */
export interface CtpDevice {
  id: string;
  device_no: string;
  factory_id: string;
  status: LockStatus;
  battery: string | null;
  signal: number | null;
  voltage: string | null;
  park_code: string;
  created_at: string;
  updated_at: string;
  status_parsed?: StatusParsed;
  status_two_parsed?: {
    total_entry_count: number;
    total_exit_count: number;
    total_theft_count: number;
    current_occupancy: number;
  };
  battery_level?: string;
}

/** 设备上报数据 */
export interface DeviceDataUpload {
  device_no: string;
  data_type: CtpDataType;
  voltage?: string;
  status_one?: string;
  status_two?: string;
  data_time?: string;
}

/** 锁控制请求 */
export interface LockControlRequest {
  factory_id: string;
  device_no: string;
  cmd_type: CmdType;
  data?: string;
}

/** 设备查询参数 */
export interface DeviceQueryParams {
  park_code?: string;
  status?: LockStatus;
  page?: number;
  page_size?: number;
}

// ==================== API 函数 ====================

/** 接收设备上报数据 */
export async function reportDeviceData(data: DeviceDataUpload) {
  try {
    return await httpClient.post('/v1/ctp/report', data);
  } catch (error) {
    handleApiError(error, 'CTP设备管理');
    throw error;
  }
}

/** 发送锁控制命令 */
export async function controlLock(cmd: LockControlRequest) {
  try {
    return await httpClient.post('/v1/ctp/device/control', cmd);
  } catch (error) {
    handleApiError(error, 'CTP设备管理');
    throw error;
  }
}

/** 查询单个设备状态 */
export async function getDevice(deviceNo: string) {
  try {
    return await httpClient.get(`/v1/ctp/device/${deviceNo}`);
  } catch (error) {
    handleApiError(error, 'CTP设备管理');
    throw error;
  }
}

/** 分页获取设备列表 */
export async function listDevices(params?: DeviceQueryParams) {
  try {
    return await httpClient.post('/v1/ctp/device/list', { params });
  } catch (error) {
    handleApiError(error, 'CTP设备管理');
    throw error;
  }
}
