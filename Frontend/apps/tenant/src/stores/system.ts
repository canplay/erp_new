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



/**
 * @file tenant.ts
 * @description 租户状态管理
 * @date 2026-05-05
 */

import { setStorageItem } from '@/utils/storage';
import {
  // 管理员 API
  listAllTenants,
  createTenant,
  updateTenantAdmin,
  deleteTenant,
  enableTenant,
  disableTenant,
  resetTenantQuota,
  searchUsersByEmail,
  getTenantPlans,
  // 用户端 API
  getCurrentTenant,
  updateTenant,
  listTenantUsers,
  addTenantUser,
  updateTenantUser,
  removeTenantUser,
  switchTenant,
  getUsageStats,
} from '@/api/tenant';
import type { Tenant, TenantUser, TenantUsage, Plan } from '@/types/tenant';
import type { User } from '@/types/user';

export const useTenantStore = defineStore('tenant', () => {
  // ============ 状态定义 ============

  /** 当前租户 */
  const currentTenant = ref<Tenant | null>(null);

  /** 租户用户列表 */
  const users = ref<TenantUser[]>([]);

  /** 使用统计 */
  const usageStats = ref<TenantUsage | null>(null);

  /** 加载状态 */
  const loading = ref(false);

  /** 所有租户列表（管理员用） */
  const allTenants = ref<Tenant[]>([]);

  /** 所有租户分页信息 */
  const tenantsPagination = ref({
    page: 1,
    page_size: 20,
    total: 0,
  });

  /** 套餐列表 */
  const plans = ref<Array<{
    id: string;
    name: string;
    price: number;
    interval: string;
    maxUsers: number;
    maxStorage: number;
  }>>([]);

  // ============ 计算属性 ============

  /** 用户使用百分比 */
  const userUsagePercent = computed(() => {
    if (!usageStats.value) return 0;
    // 修复 (2026-08-07): 兼容后端扁平结构 {users_count, storage_used}
    // 与嵌套结构 {users:{used,limit}} —— 原直接 .users.used 在扁平结构下
    // undefined.used 崩溃 → 租户页 ErrorBoundary 'Cannot read properties
    // of undefined (reading \'used\')'
    const users = usageStats.value.users ?? { used: usageStats.value.users_count ?? 0, limit: 0 };
    if (!users.limit) return 0;
    return Math.round((users.used / users.limit) * 100);
  });

  /** 存储使用百分比 */
  const storageUsagePercent = computed(() => {
    if (!usageStats.value) return 0;
    const storage = usageStats.value.storage ?? { used: usageStats.value.storage_used ?? 0, limit: 0 };
    if (!storage.limit) return 0;
    return Math.round((storage.used / storage.limit) * 100);
  });

  /** API调用使用百分比 */
  const apiUsagePercent = computed(() => {
    if (!usageStats.value) return 0;
    const apiCalls = usageStats.value.apiCalls ?? { used: 0, limit: 0 };
    if (!apiCalls.limit) return 0;
    return Math.round((apiCalls.used / apiCalls.limit) * 100);
  });

  /** 是否超限 */
  const isOverLimit = computed(() => {
    if (!usageStats.value) return false;
    const users = usageStats.value.users ?? { used: usageStats.value.users_count ?? 0, limit: 0 };
    const storage = usageStats.value.storage ?? { used: usageStats.value.storage_used ?? 0, limit: 0 };
    const apiCalls = usageStats.value.apiCalls ?? { used: 0, limit: 0 };
    return (
      (users.limit > 0 && users.used >= users.limit) ||
      (storage.limit > 0 && storage.used >= storage.limit) ||
      (apiCalls.limit > 0 && apiCalls.used >= apiCalls.limit)
    );
  });

  // ============ 方法 ============

  /**
   * @brief 加载当前租户
   */
  async function loadCurrentTenant() {
    loading.value = true;
    try {
      const response = await getCurrentTenant();
      currentTenant.value = (response as { data?: Tenant }).data ?? null;
    } catch (error) {
      logger.error('【加载租户信息失败】', error);
    } finally {
      loading.value = false;
    }
  }

  /**
   * @brief 更新租户信息
   */
  async function saveTenant(data: Partial<Tenant>): Promise<boolean | undefined> {
    if (!currentTenant.value) return undefined;

    try {
      await updateTenant(currentTenant.value.id, data);
      Object.assign(currentTenant.value, data);
      return true;
    } catch (error) {
      logger.error('【更新租户信息失败】', error);
      throw error;
    }
  }

  /**
   * @brief 加载租户用户
   */
  async function loadUsers(params?: { role?: string; keyword?: string }) {
    loading.value = true;
    try {
      const response = await listTenantUsers(params);
      const data = (response as unknown as { data?: { list?: TenantUser[] } }).data;
      users.value = data?.list || [];
    } catch (error) {
      logger.error('【加载租户用户失败】', error);
    } finally {
      loading.value = false;
    }
  }

  /**
   * @brief 添加租户用户
   */
  async function addUser(data: { user_id: number; role: string; department?: string; position?: string }) {
    try {
      await addTenantUser(data);
      await loadUsers();
      return true;
    } catch (error) {
      logger.error('【添加租户用户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 更新租户用户
   */
  async function updateUser(user_id: number, data: Partial<TenantUser>) {
    try {
      await updateTenantUser(user_id, data);
      const index = users.value.findIndex((u) => u.user_id === user_id);
      if (index >= 0 && users.value[index]) {
        const filtered = Object.fromEntries(
          Object.entries(data).filter(([, v]) => v !== undefined)
        );
        Object.assign(users.value[index], filtered);
      }
      return true;
    } catch (error) {
      logger.error('【更新租户用户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 移除租户用户
   */
  async function removeUser(user_id: number) {
    try {
      await removeTenantUser(user_id);
      users.value = users.value.filter((u) => u.user_id !== user_id);
      return true;
    } catch (error) {
      logger.error('【移除租户用户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 切换租户
   */
  async function switchTo(tenantId: number) {
    try {
      await switchTenant(tenantId);
      // 审计修复 (C5): 持久化活跃租户, 请求层 (utils/alova.ts) 据此注入 X-Tenant-Id 头
      setStorageItem('myai_active_tenant_id', String(tenantId));
      await loadCurrentTenant();
      return true;
    } catch (error) {
      logger.error('【切换租户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 加载使用统计
   */
  async function loadUsageStats() {
    try {
      const response = await getUsageStats();
      usageStats.value = ((response as { data?: TenantUsage }).data ?? null);
    } catch (error) {
      logger.error('【加载使用统计失败】', error);
    }
  }

  // ============ 管理员方法 ============

  /**
   * @brief 加载所有租户（管理员）
   */
  async function loadAllTenants(params?: { keyword?: string; status?: string; plan?: string }) {
    loading.value = true;
    try {
      const response = await listAllTenants({
        page: tenantsPagination.value.page,
        page_size: tenantsPagination.value.page_size,
        ...params,
      });
      const data = (response as { data?: { list: Tenant[]; total: number } }).data;
      if (data) {
        allTenants.value = data.list || [];
        tenantsPagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【加载租户列表失败】', error);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  /**
   * @brief 创建租户（管理员）
   */
  async function addTenant(data: {
    name: string;
    code: string;
    domain?: string;
    plan: string;
    adminEmail: string;
    adminName?: string;
    maxUsers?: number;
    maxStorage?: number;
    expires_at?: string;
  }) {
    try {
      await createTenant(data);
      await loadAllTenants();
      return true;
    } catch (error) {
      logger.error('【创建租户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 更新租户（管理员）
   */
  async function updateTenantById(id: number, data: {
    name?: string;
    domain?: string;
    plan?: string;
    status?: string;
    maxUsers?: number;
    maxStorage?: number;
    expires_at?: string;
  }) {
    try {
      await updateTenantAdmin(id, data);
      const index = allTenants.value.findIndex((t) => t.id === id);
      if (index >= 0) {
        const filtered = Object.fromEntries(
          Object.entries(data).filter(([, v]) => v !== undefined)
        );
        Object.assign(allTenants.value[index]!, filtered);
      }
      return true;
    } catch (error) {
      logger.error('【更新租户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除租户（管理员）
   */
  async function removeTenant(id: number) {
    try {
      await deleteTenant(id);
      allTenants.value = allTenants.value.filter((t) => t.id !== id);
      return true;
    } catch (error) {
      logger.error('【删除租户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 启用租户（管理员）
   */
  async function enableTenantById(id: number) {
    try {
      await enableTenant(id);
      const tenant = allTenants.value.find((t) => t.id === id);
      if (tenant) {
        tenant.status = 'active';
      }
      return true;
    } catch (error) {
      logger.error('【启用租户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 禁用租户（管理员）
   */
  async function disableTenantById(id: number) {
    try {
      await disableTenant(id);
      const tenant = allTenants.value.find((t) => t.id === id);
      if (tenant) {
        tenant.status = 'suspended';
      }
      return true;
    } catch (error) {
      logger.error('【禁用租户失败】', error);
      throw error;
    }
  }

  /**
   * @brief 重置租户配额（管理员）
   */
  async function resetQuota(id: number) {
    try {
      await resetTenantQuota(id);
      await loadAllTenants();
      return true;
    } catch (error) {
      logger.error('【重置配额失败】', error);
      throw error;
    }
  }

  /**
   * @brief 搜索用户（按邮箱）
   */
  async function searchUserByEmail(email: string) {
    try {
      const response = await searchUsersByEmail(email);
      return (response as { data?: User[] }).data || null;
    } catch (error) {
      logger.error('【搜索用户失败】', error);
      return null;
    }
  }

  /**
   * @brief 加载套餐列表
   */
  async function loadPlans() {
    try {
      const response = await getTenantPlans();
      const data = (response as unknown as { data?: Plan[] }).data;
      if (data) {
        plans.value = data.map(p => ({
          id: p.id,
          name: p.name,
          price: p.price,
          interval: p.interval,
          maxUsers: (p as unknown as { maxUsers?: number }).maxUsers ?? 0,
          maxStorage: (p as unknown as { maxStorage?: number }).maxStorage ?? 0,
        }));
      }
    } catch (error) {
      logger.error('【加载套餐列表失败】', error);
    }
  }

  return {
    // 状态
    currentTenant,
    users,
    usageStats,
    loading,
    allTenants,
    tenantsPagination,
    plans,

    // 计算属性
    userUsagePercent,
    storageUsagePercent,
    apiUsagePercent,
    isOverLimit,

    // 用户端方法
    loadCurrentTenant,
    saveTenant,
    loadUsers,
    addUser,
    updateUser,
    removeUser,
    switchTo,
    loadUsageStats,

    // 管理员方法
    loadAllTenants,
    addTenant,
    updateTenantById,
    removeTenant,
    enableTenantById,
    disableTenantById,
    resetQuota,
    searchUserByEmail,
    loadPlans,
  };
});


/**
 * @file operation-log.ts
 * @description 操作日志状态管理
 * @date 2026-04-03
 */

import {
  listOperationLogs,
  listAuditLogs,
  getOperationLog,
} from '@/api/operation-log';
import type { OperationLog, AuditLog, OperationType } from '@/types/log';

export const useOperationLogStore = defineStore('operationLog', () => {
  // ============ 状态定义 ============

  /** 操作日志列表 */
  const operationLogs = ref<OperationLog[]>([]);

  /** 审计日志列表 */
  const auditLogs = ref<AuditLog[]>([]);

  /** 加载状态 */
  const isLoading = ref(false);

  /** 分页信息 */
  const pagination = ref({
    page: 1,
    page_size: 20,
    total: 0,
  });

  /** 筛选条件 */
  const filters = ref<{
    keyword: string;
    username: string;
    module: string;
    operation_type: OperationType | null;
    success: boolean | null;
    start_date: string;
    end_date: string;
  }>({
    keyword: '',
    username: '',
    module: '',
    operation_type: null,
    success: null,
    start_date: '',
    end_date: '',
  });

  // ============ 计算属性 ============

  /** 危险操作（如删除） */
  const dangerousLogs = computed(() =>
    operationLogs.value.filter((log) => log.operation_type === 'delete' || !log.success)
  );

  // ============ Actions ============

  /**
   * @brief 获取操作日志列表
   */
  async function fetchOperationLogs() {
    isLoading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        keyword?: string;
        username?: string;
        module?: string;
        operation_type?: OperationType;
        success?: boolean;
        start_date?: string;
        end_date?: string;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      };
      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.username) params.username = filters.value.username;
      if (filters.value.module) params.module = filters.value.module;
      if (filters.value.operation_type !== null) params.operation_type = filters.value.operation_type ?? undefined;
      if (filters.value.success !== null) params.success = filters.value.success ?? undefined;
      if (filters.value.start_date) params.start_date = filters.value.start_date;
      if (filters.value.end_date) params.end_date = filters.value.end_date;
      const response = await listOperationLogs(params);

      const data = (response as unknown as { data?: { list?: OperationLog[]; total?: number } }).data;
      if (data) {
        operationLogs.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取操作日志失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取审计日志列表
   */
  async function fetchAuditLogs() {
    isLoading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        keyword?: string;
        username?: string;
        resource_type?: string;
        action?: string;
        approval_status?: string;
        start_date?: string;
        end_date?: string;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      };
      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.username) params.username = filters.value.username;
      if (filters.value.start_date) params.start_date = filters.value.start_date;
      if (filters.value.end_date) params.end_date = filters.value.end_date;
      const response = await listAuditLogs(params);

      const data = (response as unknown as { data?: { list?: AuditLog[]; total?: number } }).data;
      if (data) {
        auditLogs.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取审计日志失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取操作日志详情
   * @param id - 日志ID
   */
  async function fetchOperationLogDetail(id: number) {
    try {
      const response = await getOperationLog(id);
      return (response as unknown as { data?: OperationLog }).data;
    } catch (error) {
      logger.error('【获取操作日志详情失败】', error);
      throw error;
    }
  }

  /**
   * @brief 设置筛选条件
   */
  function setFilters(newFilters: Partial<typeof filters.value>) {
    filters.value = { ...filters.value, ...newFilters };
    pagination.value.page = 1;
  }

  /**
   * @brief 重置筛选
   */
  function resetFilters() {
    filters.value = {
      keyword: '',
      username: '',
      module: '',
      operation_type: null,
      success: null,
      start_date: '',
      end_date: '',
    };
    pagination.value.page = 1;
  }

  return {
    // 状态
    operationLogs,
    auditLogs,
    isLoading,
    pagination,
    filters,

    // 计算属性
    dangerousLogs,

    // 方法
    fetchOperationLogs,
    fetchAuditLogs,
    fetchOperationLogDetail,
    setFilters,
    resetFilters,
  };
});

