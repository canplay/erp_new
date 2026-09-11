/**
 * @file dictionary.ts
 * @description 数据字典状态管理
 * @date 2026-04-03
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
import {
  listDictionaryTypes,
  listDictionaryItems,
  getDictionaryItemsByType,
  createDictionaryType,
  updateDictionaryType,
  deleteDictionaryType,
  createDictionaryItem,
  updateDictionaryItem,
  deleteDictionaryItem,
  batchCreateDictionaryItems,
  reorderDictionaryItems,
  batchDeleteDictionaryItems,
  batchDeleteDictionaryTypes,
  type DictionaryType,
  type DictionaryItem,
} from '@/api/dictionary';

export const useDictionaryStore = defineStore('dictionary', () => {
  // ============ 状态定义 ============

  /** 字典类型列表 */
  const dictionaryTypes = ref<DictionaryType[]>([]);

  /** 当前类型的字典项 */
  const dictionaryItems = ref<DictionaryItem[]>([]);

  /** 加载状态 */
  const isLoading = ref(false);

  /** 分页信息 */
  const pagination = ref({
    page: 1,
    page_size: 20,
    total: 0,
  });

  /** 缓存的字典数据（key: typeCode, value: items） */
  const cachedDictionaries = ref<Record<string, DictionaryItem[]>>({});

  // ============ 计算属性 ============

  /** 启用的字典类型 */
  const enabledTypes = computed(() =>
    dictionaryTypes.value.filter((t) => t.status === 1)
  );

  // ============ Actions ============

  /**
   * @brief 获取字典类型列表
   */
  async function fetchDictionaryTypes() {
    isLoading.value = true;
    try {
      const response = await listDictionaryTypes({
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      });
      const data = (response as unknown as { data?: { list?: DictionaryType[]; total?: number } }).data;
      if (data) {
        dictionaryTypes.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取字典类型失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取字典项列表
   * @param type_id - 字典类型ID
   */
  async function fetchDictionaryItems(type_id: number) {
    isLoading.value = true;
    try {
      const response = await listDictionaryItems({ type_id });
      const data = (response as unknown as { data?: { list?: DictionaryItem[]; total?: number } }).data;
      if (data) {
        dictionaryItems.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取字典项失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取字典项（按类型编码，用于下拉选项）
   * @param typeCode - 字典类型编码
   */
  async function fetchDictionaryByTypeCode(typeCode: string) {
    // 先检查缓存
    if (cachedDictionaries.value[typeCode]) {
      return cachedDictionaries.value[typeCode];
    }

    try {
      const response = await getDictionaryItemsByType(typeCode);
      const items = (response as unknown as { data?: DictionaryItem[] }).data || [];
      cachedDictionaries.value[typeCode] = items;
      return items;
    } catch (error) {
      logger.error('【获取字典项失败】', error);
      throw error;
    }
  }

  /**
   * @brief 创建字典类型
   */
  async function addDictionaryType(data: {
    code: string;
    name: string;
    description?: string;
    sort?: number;
  }) {
    try {
      const response = await createDictionaryType(data);
      const newType = (response as unknown as { data?: DictionaryType }).data;
      if (newType) {
        dictionaryTypes.value.push(newType);
      }
      return newType;
    } catch (error) {
      logger.error('【创建字典类型失败】', error);
      throw error;
    }
  }

  /**
   * @brief 更新字典类型
   */
  async function editDictionaryType(
    id: number,
    data: {
      name?: string;
      description?: string;
      sort?: number;
      status?: number;
    }
  ) {
    try {
      await updateDictionaryType(id, data);
      const index = dictionaryTypes.value.findIndex((t) => t.id === id);
      if (index !== -1) {
        const existing = dictionaryTypes.value[index];
        if (existing) {
          const updated: DictionaryType = { ...existing };
          if (data.name !== undefined) updated.name = data.name;
          if (data.description !== undefined) updated.description = data.description;
          if (data.sort !== undefined) updated.sort = data.sort;
          if (data.status !== undefined) updated.status = data.status;
          updated.updated_at = new Date().toISOString();
          dictionaryTypes.value[index] = updated;
        }
      }
    } catch (error) {
      logger.error('【更新字典类型失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除字典类型
   */
  async function removeDictionaryType(id: number) {
    try {
      await deleteDictionaryType(id);
      dictionaryTypes.value = dictionaryTypes.value.filter((t) => t.id !== id);
    } catch (error) {
      logger.error('【删除字典类型失败】', error);
      throw error;
    }
  }

  /**
   * @brief 创建字典项
   */
  async function addDictionaryItem(data: {
    type_id: number;
    label: string;
    value: string;
    sort?: number;
    status?: number;
    is_default?: boolean;
    remark?: string;
  }) {
    try {
      const response = await createDictionaryItem(data);
      const newItem = (response as unknown as { data?: DictionaryItem }).data;
      if (newItem) {
        dictionaryItems.value.push(newItem);
      }
      return newItem;
    } catch (error) {
      logger.error('【创建字典项失败】', error);
      throw error;
    }
  }

  /**
   * @brief 更新字典项
   */
  async function editDictionaryItem(
    id: number,
    data: {
      label?: string;
      value?: string;
      sort?: number;
      status?: number;
      is_default?: boolean;
      remark?: string;
    }
  ) {
    try {
      await updateDictionaryItem(id, data);
      const index = dictionaryItems.value.findIndex((i) => i.id === id);
      if (index !== -1) {
        const existing = dictionaryItems.value[index];
        if (existing) {
          const updated: DictionaryItem = { ...existing };
          if (data.label !== undefined) updated.label = data.label;
          if (data.value !== undefined) updated.value = data.value;
          if (data.sort !== undefined) updated.sort = data.sort;
          if (data.status !== undefined) updated.status = data.status;
          if (data.is_default !== undefined) updated.is_default = data.is_default;
          if (data.remark !== undefined) updated.remark = data.remark;
          updated.updated_at = new Date().toISOString();
          dictionaryItems.value[index] = updated;
        }
      }
      // 清除缓存
      const type = dictionaryTypes.value.find((t) => t.id === dictionaryItems.value[index]?.type_id);
      if (type) {
        delete cachedDictionaries.value[type.code];
      }
    } catch (error) {
      logger.error('【更新字典项失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除字典项
   */
  async function removeDictionaryItem(id: number) {
    try {
      await deleteDictionaryItem(id);
      dictionaryItems.value = dictionaryItems.value.filter((i) => i.id !== id);
    } catch (error) {
      logger.error('【删除字典项失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量添加字典项
   */
  async function batchAddDictionaryItems(
    type_id: number,
    items: Array<{
      label: string;
      value: string;
      sort?: number;
      is_default?: boolean;
      remark?: string;
    }>
  ) {
    try {
      await batchCreateDictionaryItems(type_id, items);
      // 刷新列表
      await fetchDictionaryItems(type_id);
      // 清除缓存
      const type = dictionaryTypes.value.find((t) => t.id === type_id);
      if (type) {
        delete cachedDictionaries.value[type.code];
      }
    } catch (error) {
      logger.error('【批量创建字典项失败】', error);
      throw error;
    }
  }

  /**
   * @brief 调整字典项顺序
   */
  async function changeItemOrder(type_id: number, itemIds: number[]) {
    try {
      await reorderDictionaryItems(type_id, itemIds);
      // 重新排序本地数据
      const sortedItems = itemIds
        .map((id) => dictionaryItems.value.find((i) => i.id === id))
        .filter((item): item is DictionaryItem => item !== undefined);
      dictionaryItems.value = sortedItems;
    } catch (error) {
      logger.error('【调整顺序失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量删除字典项
   */
  async function batchRemoveDictionaryItems(ids: number[]) {
    try {
      await batchDeleteDictionaryItems(ids);
      dictionaryItems.value = dictionaryItems.value.filter((i) => !ids.includes(i.id));
    } catch (error) {
      logger.error('【批量删除字典项失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量删除字典类型
   */
  async function batchRemoveDictionaryTypes(ids: number[]) {
    try {
      await batchDeleteDictionaryTypes(ids);
      dictionaryTypes.value = dictionaryTypes.value.filter((t) => !ids.includes(t.id));
    } catch (error) {
      logger.error('【批量删除字典类型失败】', error);
      throw error;
    }
  }

  /**
   * @brief 调整字典类型顺序
   */
  function change_type_order(type_ids: number[]) {
    try {
      // 后端可能需要这个接口，暂时只更新本地数据
      const sortedTypes = type_ids
        .map((id) => dictionaryTypes.value.find((t) => t.id === id))
        .filter((t): t is DictionaryType => t !== undefined);
      dictionaryTypes.value = sortedTypes;
    } catch (error) {
      logger.error('【调整字典类型顺序失败】', error);
      throw error;
    }
  }

  /**
   * @brief 刷新所有字典缓存
   */
  async function refreshAllCache() {
    try {
      cachedDictionaries.value = {};
      // 预加载常用字典
      const commonCodes = ['status', 'gender', 'yes_no', 'user_status', 'device_type'];
      await Promise.all(commonCodes.map((code) => fetchDictionaryByTypeCode(code)));
    } catch (error) {
      logger.error('【刷新字典缓存失败】', error);
    }
  }

  /**
   * @brief 获取字典项的标签
   * @param typeCode - 字典类型编码
   * @param value - 字典项值
   */
  function getDictionaryLabel(typeCode: string, value: string | number): string {
    const items = cachedDictionaries.value[typeCode];
    if (!items) return String(value);
    const item = items.find((i) => i.value === String(value) || i.value === value);
    return item?.label || String(value);
  }

  /**
   * @brief 清除缓存
   */
  function clearCache() {
    cachedDictionaries.value = {};
  }

  /**
   * @brief 获取角色类型选项（从数据字典）
   * @description 从字典缓存中获取 role_type 类型的字典项作为角色类型选项
   */
  async function fetchRoleTypes(): Promise<{ label: string; value: string }[]> {
    try {
      // 先尝试从缓存获取
      if (cachedDictionaries.value['role_type']) {
        return cachedDictionaries.value['role_type'].map((item) => ({
          label: item.label,
          value: item.value,
        }));
      }
      // 从后端获取
      const items = await fetchDictionaryByTypeCode('role_type');
      return items.map((item: { label: string; value: string }) => ({
        label: item.label,
        value: item.value,
      }));
    } catch (error) {
      logger.error('【获取角色类型失败】', error);
      // 返回默认值
      return [
        { label: '普通用户', value: 'user' },
        { label: 'VIP用户', value: 'vip' },
        { label: '管理员', value: 'admin' },
      ];
    }
  }

  return {
    // 状态
    dictionaryTypes,
    dictionaryItems,
    isLoading,
    pagination,

    // 计算属性
    enabledTypes,
    cachedDictionaries,

    // 方法
    fetchDictionaryTypes,
    fetchDictionaryItems,
    fetchDictionaryByTypeCode,
    addDictionaryType,
    editDictionaryType,
    removeDictionaryType,
    addDictionaryItem,
    editDictionaryItem,
    removeDictionaryItem,
    batchAddDictionaryItems,
    changeItemOrder,
    batchRemoveDictionaryItems,
    batchRemoveDictionaryTypes,
    change_type_order,
    refreshAllCache,
    getDictionaryLabel,
    clearCache,
    fetchRoleTypes,
  };
});

