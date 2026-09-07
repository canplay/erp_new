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
