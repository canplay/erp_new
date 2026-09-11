/**
 * @file tenant.ts
 * @description 租户状态管理
 * @date 2026-05-05
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
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
