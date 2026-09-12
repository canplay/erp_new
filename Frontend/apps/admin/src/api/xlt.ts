/**
 * @file xlt.ts
 * @description XLT 信路通停车管理 API
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

// ==================== 类型定义 ====================

/** 车辆事件 */
export interface VehicleEvent {
  plate_no: string;
  plate_color: string;
  park_code: string;
  lane_code: string;
  event_time: string;
  vehicle_type: string;
  image_url: string;
  amount?: number;
  direction: string;
}

/** 在场车辆 */
export interface ParkingVehicle {
  plateNo: string;
  plateColor: string;
  parkCode: string;
  laneCode: string;
  entryTime: string;
  vehicleType: string;
  imageUrl: string;
  status: string;
  created_at: string;
}

/** 计费请求 */
export interface BillingRequest {
  plate_no: string;
  park_code: string;
  entry_time: string;
  exit_time: string;
}

/** 计费结果 */
export interface BillingResult {
  plate_no: string;
  park_code: string;
  entry_time: string;
  exit_time: string;
  duration_minutes: number;
  total_amount: number;
  rule: string;
}

/** 设备信息 */
export interface XltDeviceInfo {
  sn: string;
  client_id: string;
  dev_info: string;
  version: string;
  connected_at: string;
  last_heartbeat: string;
}

/** 通行记录查询参数 */
export interface VehicleQueryParams {
  park_code?: string;
  plate_no?: string;
  page?: number;
  page_size?: number;
}

// ==================== API 函数 ====================

/** 车辆入场 */
export function vehicleEntry(event: VehicleEvent) {
  try {
    return await httpClient.post('/v1/xlt/parking/entry', event);
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 车辆出场 */
export function vehicleExit(event: VehicleEvent) {
  try {
    return await httpClient.post('/v1/xlt/parking/exit', event);
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 查询在场车辆 */
export function getParkingVehicle(parkCode: string, plateNo: string) {
  try {
    return await httpClient.get(`/xlt/parking/vehicle/${parkCode}/${plateNo}`);
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 计算停车费用 */
export function calcBilling(req: BillingRequest) {
  try {
    return await httpClient.post('/v1/xlt/parking/billing', req);
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 查询进出记录列表 */
export function listParkingRecords(params?: VehicleQueryParams) {
  try {
    return await httpClient.get('/v1/xlt/parking/records', { params });
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 获取设备列表 */
export function listXltDevices() {
  try {
    return await httpClient.get('/v1/xlt/device/list');
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 开闸 */
export function openBarrier(sn: string) {
  try {
    return await httpClient.post('/v1/xlt/device/open', { sn });
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}

/** 关闸 */
export function closeBarrier(sn: string) {
  try {
    return await httpClient.post('/v1/xlt/device/close', { sn });
  } catch (error) {
    handleApiError(error, 'XLT停车管理');
    throw error;
  }
}
