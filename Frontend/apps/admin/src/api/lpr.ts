/**
 * @file lpr.ts
 * @description LPR 车牌识别通行记录 API
 */

import { httpClient } from '@/utils/alova';

// ==================== 类型定义 ====================

/** 通行方向 */
export type PassDirection = 'entry' | 'exit';

/** 识别结果处理状态 */
export type RecordStatus = 'pending' | 'processed' | 'failed';

/** 车牌识别通行记录 */
export interface PassRecord {
  id: number;
  plate_no: string;
  plate_color: string;
  plate_type: string;
  vehicle_type: string;
  device_id: string;
  device_name: string;
  park_code: string;
  lane_code: string;
  direction: PassDirection;
  pass_time: string;
  image_url: string;
  confidence: number;
  status: RecordStatus;
  related_order_id: string;
  remark: string;
  created_at: string;
  updated_at: string;
}

/** 车辆授权信息 */
export interface VehicleAuthorization {
  is_authorized: boolean;
  auth_type: string;
  driver_name: string;
  driver_phone: string;
  valid_until: string | null;
}

/** 通行记录查询参数 */
export interface PassRecordQueryParams {
  plate_no?: string;
  park_code?: string;
  direction?: PassDirection;
  status?: RecordStatus;
  device_id?: string;
  start_time?: string;
  end_time?: string;
  page?: number;
  page_size?: number;
}

/** 车牌识别回调结果 */
export interface LprCallbackResult {
  success: boolean;
  message: string;
  code: number;
}

/** 车牌识别设备 */
export interface LprDevice {
  id: string;
  name: string;
  sn: string;
  status: 'online' | 'offline';
  park_code: string;
  lane_code?: string;
  direction?: string;
  ip_address?: string;
  last_heartbeat?: string;
  created_at?: string;
}

// ==================== API 函数 ====================

/** 获取设备列表 */
export function listLprDevices(params?: { park_code?: string; status?: string }) {
  return httpClient.get('/v1/lpr/devices', { params });
}

/** 获取设备详情 */
export function getLprDevice(id: string) {
  return httpClient.get(`/v1/lpr/devices/${id}`);
}

/** 查询通行记录列表 */
export function listPassRecords(params?: PassRecordQueryParams) {
  return httpClient.get('/v1/lpr/records', { params });
}

/** 获取通行记录详情 */
export function getPassRecord(id: number) {
  return httpClient.get(`/v1/lpr/records/${id}`);
}

/** 查询车辆授权信息 */
export function getVehicleAuth(plateNo: string, parkCode: string) {
  return httpClient.get('/lpr/vehicle/auth', {
    params: { plate_no: plateNo, park_code: parkCode },
  });
}

/** 获取通行统计 */
export function getPassStats(params?: { park_code?: string; start_date?: string; end_date?: string }) {
  return httpClient.get('/v1/lpr/stats', { params });
}
