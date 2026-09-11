/**
 * @file tow.ts
 * @description 拖车服务 API
 */

import { httpClient } from '@/utils/alova';

// ==================== 类型定义 ====================

/** 车辆档案 */
export interface TowCar {
  id: number;
  license: string;
  vehicle: Record<string, unknown> | null;
  engine: string;
  car_type: string;
  dc_type: string;
  dc_causes: string;
  car_color: string;
  dc_date: string;
  dc_address: string;
  dc_key: string;
  dc_party_name: string;
  dc_party_cardid: string;
  dc_party_tel: string;
  p_name: string;
  driver: string;
  operator: string;
  create_date: string;
  update_date: string;
  delete: boolean;
  remark: string;
}

/** 车辆查询参数 */
export interface TowCarQueryParams {
  keyword?: string;
  car_type?: string;
  status?: string;
  page?: number;
  page_size?: number;
}

/** 字典项 */
export interface DictItem {
  id: number;
  name: string;
  value: string;
  sort_order?: number;
}

/** 拖车任务 */
export interface TowTask {
  id: number;
  car_id: number;
  license: string;
  status: string;
  assignee: string;
  created_at: string;
  updated_at: string;
}

// ==================== API 函数 ====================

/** 查询车辆列表 */
export function listTowCars(params?: TowCarQueryParams) {
  return httpClient.get('/tow/cars', { params });
}

/** 获取车辆详情 */
export function getTowCar(id: number) {
  return httpClient.get(`/tow/cars/${id}`);
}

/** 获取字典列表 */
export function listDictItems(dictType: string) {
  return httpClient.get('/tow/dict', { params: { dict_type: dictType } });
}

/** 拖车类型字典 */
export function listCarTypes() {
  return listDictItems('car_type');
}

/** 车辆颜色字典 */
export function listCarColors() {
  return listDictItems('car_color');
}

/** 拖车原因字典 */
export function listDcCauses() {
  return listDictItems('dc_causes');
}
