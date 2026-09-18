/**
 * @file device.ts
 * @description 设备管理状态管理
 * @date 2026-05-05
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
import {
  getAllDevices,
  getDeviceStatistics,
  getAbnormalLogins,
  kickDevice,
  trustDevice,
  untrustDevice,
  deleteDevice as apiDeleteDevice,
  batchDeleteDevices,
  getMyDevices,
  kickOtherDevices,
  type LoginDevice,
  type LoginDeviceQueryParams,
} from '@/api/device';

export const useDeviceStore = defineStore('device', () => {
  // ============ 状态定义 ============

  /** 设备列表 */
  const devices = ref<LoginDevice[]>([]);

  /** 当前用户的设备列表 */
  const myDevices = ref<LoginDevice[]>([]);

  /** 异常登录列表 */
  const abnormalLogins = ref<LoginDevice[]>([]);

  /** 加载状态 */
  const isLoading = ref(false);

  /** 分页信息 */
  const pagination = ref({
    page: 1,
    page_size: 15,
    total: 0,
  });

  /** 设备统计数据 */
  const statistics = ref({
    total: 0,
    mobile: 0,
    desktop: 0,
    active: 0,
  });

  /** 搜索关键词 */
  const searchKeyword = ref('');

  // ============ 计算属性 ============

  /** 在线设备数量 */
  const onlineCount = computed(() => devices.value.filter((d) => d.isActive).length);

  /** 移动设备数量 */
  const mobileCount = computed(() => devices.value.filter((d) => d.deviceType === 'mobile').length);

  /** 桌面设备数量 */
  const desktopCount = computed(() => devices.value.filter((d) => d.deviceType === 'desktop').length);

  // ============ Actions ============

  /**
   * @brief 获取设备列表
   */
  async function fetchDevices(params?: LoginDeviceQueryParams) {
    isLoading.value = true;
    try {
      const response = await getAllDevices({
        keyword: searchKeyword.value,
        page: pagination.value.page,
        page_size: pagination.value.page_size,
        ...params,
      });
      const data = (response as unknown as { data?: { list?: LoginDevice[]; total?: number } }).data;
      if (data) {
        devices.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取设备列表失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取当前用户的设备列表
   */
  async function fetchMyDevices(params?: Omit<LoginDeviceQueryParams, 'user_id'>) {
    isLoading.value = true;
    try {
      const response = await getMyDevices(params);
      const data = (response as unknown as { data?: { list?: LoginDevice[] } }).data;
      if (data) {
        myDevices.value = data.list || [];
      }
    } catch (error) {
      logger.error('【获取我的设备列表失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取设备统计
   */
  async function fetchStatistics() {
    try {
      const response = await getDeviceStatistics();
      const data = (response as unknown as { data?: typeof statistics.value }).data;
      if (data) {
        statistics.value = data;
      }
    } catch (error) {
      logger.error('【获取设备统计失败】', error);
    }
  }

  /**
   * @brief 获取异常登录列表
   */
  async function fetchAbnormalLogins(params?: { page?: number; page_size?: number }) {
    isLoading.value = true;
    try {
      const response = await getAbnormalLogins({
        page: pagination.value.page,
        page_size: pagination.value.page_size,
        ...params,
      });
      const data = (response as unknown as { data?: { list?: LoginDevice[]; total?: number } }).data;
      if (data) {
        abnormalLogins.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取异常登录列表失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 踢出设备
   */
  async function kickDeviceById(id: number) {
    try {
      await kickDevice(id);
      // 从列表中移除
      devices.value = devices.value.filter((d) => d.id !== id);
      // 更新统计数据
      await fetchStatistics();
    } catch (error) {
      logger.error('【踢出设备失败】', error);
      throw error;
    }
  }

  /**
   * @brief 踢出其他设备（当前用户）
   */
  async function kickOtherDevicesAction() {
    try {
      await kickOtherDevices();
      // 刷新设备列表
      await fetchMyDevices();
    } catch (error) {
      logger.error('【踢出其他设备失败】', error);
      throw error;
    }
  }

  /**
   * @brief 标记设备为信任
   */
  async function trustDeviceById(id: number) {
    try {
      await trustDevice(id);
      // 更新设备状态
      const device = devices.value.find((d) => d.id === id);
      if (device) {
        device.isTrusted = true;
      }
    } catch (error) {
      logger.error('【标记设备信任失败】', error);
      throw error;
    }
  }

  /**
   * @brief 取消设备信任
   */
  async function untrustDeviceById(id: number) {
    try {
      await untrustDevice(id);
      // 更新设备状态
      const device = devices.value.find((d) => d.id === id);
      if (device) {
        device.isTrusted = false;
      }
    } catch (error) {
      logger.error('【取消设备信任失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除设备
   */
  async function removeDevice(id: number) {
    try {
      await apiDeleteDevice(id);
      devices.value = devices.value.filter((d) => d.id !== id);
      // 更新统计数据
      await fetchStatistics();
    } catch (error) {
      logger.error('【删除设备失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量删除设备
   */
  async function batchRemoveDevices(ids: number[]) {
    try {
      await batchDeleteDevices(ids);
      devices.value = devices.value.filter((d) => !ids.includes(d.id));
      // 更新统计数据
      await fetchStatistics();
    } catch (error) {
      logger.error('【批量删除设备失败】', error);
      throw error;
    }
  }

  /**
   * @brief 设置搜索关键词
   */
  function setSearchKeyword(keyword: string) {
    searchKeyword.value = keyword;
    pagination.value.page = 1;
  }

  /**
   * @brief 设置分页
   */
  function setPage(page: number) {
    pagination.value.page = page;
  }

  /**
   * @brief 加载所有数据
   */
  async function loadAll() {
    await Promise.all([fetchDevices(), fetchStatistics()]);
  }

  return {
    // 状态
    devices,
    myDevices,
    abnormalLogins,
    isLoading,
    pagination,
    statistics,
    searchKeyword,

    // 计算属性
    onlineCount,
    mobileCount,
    desktopCount,

    // 方法
    fetchDevices,
    fetchMyDevices,
    fetchStatistics,
    fetchAbnormalLogins,
    kickDeviceById,
    kickOtherDevicesAction,
    trustDeviceById,
    untrustDeviceById,
    removeDevice,
    batchRemoveDevices,
    setSearchKeyword,
    setPage,
    loadAll,
  };
});


import * as ctpApi from '@/api/ctp';
import type { CtpDevice, DeviceQueryParams, LockControlRequest } from '@/api/ctp';

export const useCtpStore = defineStore('ctp', () => {
  const devices = ref<CtpDevice[]>([]);
  const currentDevice = ref<CtpDevice | null>(null);
  const loading = ref(false);
  const total = ref(0);
  const page = ref(1);
  const page_size = ref(20);

  const deviceCount = computed(() => devices.value.length);
  const onlineDevices = computed(() => devices.value.filter(d => d.status === 'Locked' || d.status === 'Unlocked'));
  const offlineDevices = computed(() => devices.value.filter(d => d.status === 'Offline'));

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  interface DeviceApiResponse {
    devices?: LoginDevice[];
    list?: LoginDevice[];
    total?: number;
  }

  async function fetchDevices(params?: DeviceQueryParams) {
    loading.value = true;
    try {
      const res = await ctpApi.listDevices(params);
      const data = getData(res) as DeviceApiResponse;
      devices.value = data.devices ?? data.list ?? [];
      total.value = data.total ?? devices.value.length;
      page.value = params?.page ?? 1;
      page_size.value = params?.page_size ?? 20;
    } finally {
      loading.value = false;
    }
  }

  async function fetchDevice(deviceNo: string) {
    loading.value = true;
    try {
      const res = await ctpApi.getDevice(deviceNo);
      currentDevice.value = getData(res) as unknown as CtpDevice;
    } finally {
      loading.value = false;
    }
  }

  async function sendControl(cmd: LockControlRequest) {
    const res = await ctpApi.controlLock(cmd);
    return getData(res);
  }

  return {
    devices, currentDevice, loading, total, page, page_size,
    deviceCount, onlineDevices, offlineDevices,
    fetchDevices, fetchDevice, sendControl,
  };
});


import * as lprApi from '@/api/lpr';
import type { PassRecord, PassRecordQueryParams } from '@/api/lpr';

export const useLprStore = defineStore('lpr', () => {
  const records = ref<PassRecord[]>([]);
  const currentRecord = ref<PassRecord | null>(null);
  const loading = ref(false);
  const total = ref(0);
  const page = ref(1);
  const page_size = ref(20);

  const todayEntryCount = computed(() => records.value.filter(r => r.direction === 'entry').length);
  const todayExitCount = computed(() => records.value.filter(r => r.direction === 'exit').length);
  const highConfidence = computed(() => records.value.filter(r => r.confidence >= 0.9).length);

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  async function fetchRecords(params?: PassRecordQueryParams) {
    loading.value = true;
    try {
      const res = await lprApi.listPassRecords(params);
      const data = getData(res);
      records.value = (data.list as PassRecord[]) ?? (data.records as PassRecord[]) ?? [];
      total.value = (data.total as number) ?? records.value.length;
      page.value = params?.page ?? 1;
      page_size.value = params?.page_size ?? 20;
    } finally {
      loading.value = false;
    }
  }

  async function fetchRecord(id: number) {
    loading.value = true;
    try {
      const res = await lprApi.getPassRecord(id);
      currentRecord.value = getData(res) as unknown as PassRecord;
    } finally {
      loading.value = false;
    }
  }

  async function fetchStats(params?: { park_code?: string; start_date?: string; end_date?: string }) {
    const res = await lprApi.getPassStats(params);
    return getData(res);
  }

  return {
    records, currentRecord, loading, total, page, page_size,
    todayEntryCount, todayExitCount, highConfidence,
    fetchRecords, fetchRecord, fetchStats,
  };
});


import * as towApi from '@/api/tow';
import type { TowCar, TowCarQueryParams, DictItem } from '@/api/tow';

export const useTowStore = defineStore('tow', () => {
  const cars = ref<TowCar[]>([]);
  const currentCar = ref<TowCar | null>(null);
  const loading = ref(false);
  const total = ref(0);
  const page = ref(1);
  const page_size = ref(20);

  const carTypes = ref<DictItem[]>([]);
  const carColors = ref<DictItem[]>([]);
  const dcCauses = ref<DictItem[]>([]);

  const activeCars = computed(() => cars.value.filter(c => !c.delete));
  const carTypeOptions = computed(() => carTypes.value.map(d => ({ label: d.name, value: d.value })));
  const carColorOptions = computed(() => carColors.value.map(d => ({ label: d.name, value: d.value })));

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  function getDictList(res: unknown): DictItem[] {
    const data = getData(res);
    const list = (data.list as DictItem[] | undefined) ?? (data.items as DictItem[] | undefined);
    return Array.isArray(list) ? list : [];
  }

  async function fetchCars(params?: TowCarQueryParams) {
    loading.value = true;
    try {
      const res = await towApi.listTowCars(params);
      const data = getData(res);
      cars.value = (data.list as TowCar[]) ?? (data.cars as TowCar[]) ?? [];
      total.value = (data.total as number) ?? cars.value.length;
      page.value = params?.page ?? 1;
      page_size.value = params?.page_size ?? 20;
    } finally {
      loading.value = false;
    }
  }

  async function fetchCar(id: number) {
    loading.value = true;
    try {
      const res = await towApi.getTowCar(id);
      currentCar.value = getData(res) as unknown as TowCar;
    } finally {
      loading.value = false;
    }
  }

  async function fetchDicts() {
    const [types, colors, causes] = await Promise.all([
      towApi.listCarTypes(),
      towApi.listCarColors(),
      towApi.listDcCauses(),
    ]);
    carTypes.value = getDictList(types);
    carColors.value = getDictList(colors);
    dcCauses.value = getDictList(causes);
  }

  return {
    cars, currentCar, loading, total, page, page_size,
    carTypes, carColors, dcCauses,
    activeCars, carTypeOptions, carColorOptions,
    fetchCars, fetchCar, fetchDicts,
  };
});


import * as xltApi from '@/api/xlt';
import type { VehicleEvent, XltDeviceInfo } from '@/api/xlt';

export const useXltStore = defineStore('xlt', () => {
  const parkingVehicles = ref<Record<string, unknown>[]>([]);
  const records = ref<Record<string, unknown>[]>([]);
  const devices = ref<XltDeviceInfo[]>([]);
  const loading = ref(false);
  const total = ref(0);

  const currentParking = computed(() => parkingVehicles.value.length);
  const todayEntryCount = computed(() => 0);
  const todayExitCount = computed(() => 0);

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  async function fetchParkingVehicles(parkCode: string, plateNo?: string) {
    loading.value = true;
    try {
      const res = await xltApi.getParkingVehicle(parkCode, plateNo ?? '');
      parkingVehicles.value = [getData(res)];
    } finally {
      loading.value = false;
    }
  }

  async function fetchRecords(params?: Record<string, unknown>) {
    loading.value = true;
    try {
      const p = params as { park_code?: string; plate_no?: string; page?: number; page_size?: number } | undefined;
      const res = await xltApi.listParkingRecords(p);
      const data = getData(res);
      records.value = (data.list as Record<string, unknown>[]) ?? (data.records as Record<string, unknown>[]) ?? [];
      total.value = (data.total as number) ?? records.value.length;
    } finally {
      loading.value = false;
    }
  }

  async function fetchDevices() {
    loading.value = true;
    try {
      const res = await xltApi.listXltDevices();
      const data = getData(res);
      devices.value = (data.devices as XltDeviceInfo[]) ?? (data.list as XltDeviceInfo[]) ?? [];
    } finally {
      loading.value = false;
    }
  }

  async function entry(event: VehicleEvent) { return getData(await xltApi.vehicleEntry(event)); }
  async function exit(event: VehicleEvent) { return getData(await xltApi.vehicleExit(event)); }

  return {
    parkingVehicles, records, devices, loading, total,
    currentParking, todayEntryCount, todayExitCount,
    fetchParkingVehicles, fetchRecords, fetchDevices,
    entry, exit,
  };
});
