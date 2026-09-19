/**
 * @file auth.ts
 * @description 认证状态管理 - 增强版（带安全存储）
 * @date 2026-05-21
 * @description 2026-05-21 安全修复：移除虚假的 Base64 加密，改用安全的存储方式
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { httpClient } from '@/utils/alova';
import router from '@/router';
import { logger } from '@/utils/logger';
import { usePermissionStore } from './permission';
import type { UserInfo as IUserInfo } from '@/types/user';

// 重新导出类型
export type { UserInfo as IUserInfo } from '@/types/user';
export type UserInfo = IUserInfo;

export interface LoginForm {
  username: string;
  password: string;
  remember_me?: boolean;
}

/** 登录响应 */
interface LoginResponse {
  token?: string;
  access_token?: string;
  refresh_token?: string;
  user_id?: number;
  username?: string;
  role?: string;
  must_change_password?: boolean;
}

// ============ 安全存储键 ============
const STORAGE_KEYS = {
  TOKEN: 'admin_token',
  USER_INFO: 'admin_user_info',
  REFRESH_TOKEN: 'admin_refresh_token',
};


// 审计修复 (M1/M2): 已移除假加密 encryptToken/decryptToken(XOR+btoa 可逆, 无安全意义,
// 且从未被调用)。token 存储策略: refresh_token 持久化于 localStorage 用于 401 自动刷新,
// access token 依赖 alova 拦截器附加 Authorization 头。

export const useAuthStore = defineStore('auth', () => {
  // ============ 状态定义 ============
  const token = ref<string | null>(null);
  const refreshToken = ref<string | null>(null);
  const userInfo = ref<UserInfo | null>(null);
  const isLoading = ref(false);

  // ============ 计算属性 ============
  const isLoggedIn = computed(() => !!token.value);
  const isAdmin = computed(() => userInfo.value?.role === 'admin');
  const currentRole = computed(() => userInfo.value?.role || '');

  // ============ 私有方法 ============

  /**
   * @brief 安全存储到 localStorage
   * @description 使用 try-catch 防止存储异常，不使用虚假加密
   */
  function safeSetItem(key: string, value: string | null): void {
    try {
      if (value === null) {
        localStorage.removeItem(key);
      } else {
        localStorage.setItem(key, value);
      }
    } catch (error) {
      logger.error('【安全存储失败】', { key, error });
    }
  }

  /**
   * @brief 安全从 localStorage 读取
   */
  function safeGetItem(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch (error) {
      logger.error('【安全读取失败】', { key, error });
      return null;
    }
  }

  /**
   * @brief 初始化认证状态（从 localStorage 恢复）
   */
  function initFromStorage(): void {
    // 读取 Token
    token.value = safeGetItem(STORAGE_KEYS.TOKEN);

    // 读取用户信息
    const storedUserInfo = safeGetItem(STORAGE_KEYS.USER_INFO);
    if (storedUserInfo) {
      try {
        userInfo.value = JSON.parse(storedUserInfo);
      } catch {
        userInfo.value = null;
      }
    }

    // 读取刷新令牌
    refreshToken.value = safeGetItem(STORAGE_KEYS.REFRESH_TOKEN);
  }

  // ============ Actions ============

  /**
   * @brief 用户登录
   */
  async function login(form: LoginForm) {
    isLoading.value = true;
    try {
      const response = await httpClient.post('/user/login', {
        username: form.username,
        password: form.password,
      });

      const data = (response as { data?: LoginResponse }).data;
      const accessToken = data?.token || data?.access_token;

      if (accessToken) {
        // 存储 Token
        token.value = accessToken;
        safeSetItem(STORAGE_KEYS.TOKEN, accessToken);

        // 审计修复 (M2): 无论是否勾选"记住我"都持久化 refresh_token,
        // 否则 401 自动刷新在默认登录方式下永远不可用(刷新链形同虚设)
        if (data?.refresh_token) {
          refreshToken.value = data.refresh_token;
          safeSetItem(STORAGE_KEYS.REFRESH_TOKEN, data.refresh_token);
        }

        // 获取用户信息
        await fetchUserInfo();

        // 首次登录需修改密码
        if (data?.must_change_password) {
          // 标记需要修改密码，路由守卫会检测并跳转到修改密码页
          // 使用 safeSetItem 包装，防止存储异常
          try {
            localStorage.setItem('must_change_password', 'true');
          } catch {
            logger.error('【存储 must_change_password 失败】');
          }
        }

        return true;
      }

      throw new Error('登录响应中未获取到 Token');
    } catch (error) {
      logger.error('【登录失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取当前用户信息
   */
  async function fetchUserInfo() {
    if (!token.value) return null;

    try {
      const response = await httpClient.get('/user/info');
      userInfo.value = (response as { data?: UserInfo }).data ?? null;

      // 存储用户信息
      safeSetItem(STORAGE_KEYS.USER_INFO, JSON.stringify(userInfo.value));

      return userInfo.value;
    } catch (error) {
      logger.error('【获取用户信息失败】', error);
      throw error;
    }
  }

  /**
   * @brief 更新用户资料
   */
  async function updateProfile(profile: Partial<UserInfo>) {
    try {
      const response = await httpClient.put('/user/info', profile);
      const respData = (response as { data?: Partial<UserInfo> }).data;
      if (respData) {
        userInfo.value = { ...userInfo.value, ...respData } as UserInfo;
        // 更新存储
        safeSetItem(STORAGE_KEYS.USER_INFO, JSON.stringify(userInfo.value));
      }
      return true;
    } catch (error) {
      logger.error('【更新资料失败】', error);
      throw error;
    }
  }

  /**
   * @brief 修改密码
   */
  async function changePassword(old_password: string, new_password: string) {
    try {
      await httpClient.put('/user/password', {
        old_password,
        new_password,
      });
      // 清除首次登录标记
      localStorage.removeItem('must_change_password');
      return true;
    } catch (error) {
      logger.error('【修改密码失败】', error);
      throw error;
    }
  }

  /**
   * @brief 刷新 Token
   */
  async function refreshAccessToken(): Promise<boolean> {
    if (!refreshToken.value) {
      return false;
    }

    try {
      const response = await httpClient.post('/auth/refresh', {
        refresh_token: refreshToken.value,
      });

      const data = (response as { data?: { access_token?: string; refresh_token?: string } }).data;
      if (data?.access_token) {
        token.value = data.access_token;
        safeSetItem(STORAGE_KEYS.TOKEN, data.access_token);

        if (data?.refresh_token) {
          refreshToken.value = data.refresh_token;
          safeSetItem(STORAGE_KEYS.REFRESH_TOKEN, data.refresh_token);
        }

        return true;
      }
      return false;
    } catch (error) {
      logger.error('【刷新 Token 失败】', error);
      return false;
    }
  }

  /**
   * @brief 退出登录
   */
  function logout() {
    // 清除权限信息
    const permissionStore = usePermissionStore();
    permissionStore.clearPermissions();

    // 清除认证状态
    token.value = null;
    refreshToken.value = null;
    userInfo.value = null;

    // 清除所有存储
    safeSetItem(STORAGE_KEYS.TOKEN, null);
    safeSetItem(STORAGE_KEYS.USER_INFO, null);
    safeSetItem(STORAGE_KEYS.REFRESH_TOKEN, null);

    void router.push('/login');
  }

  /**
   * @brief 初始化认证状态（页面刷新时调用）
   */
  async function initAuth() {
    // 先从 localStorage 恢复状态
    initFromStorage();

    if (token.value) {
      try {
        await fetchUserInfo();
      } catch {
        // Token 失效，尝试刷新
        const refreshed = await refreshAccessToken();
        if (!refreshed) {
          // 刷新失败，清除登录状态
          void logout();
        }
      }
    }
  }

  /**
   * @brief 检查是否有指定权限
   * @param permission - 权限标识符
   * @returns 是否拥有该权限
   */
  function hasPermission(permission: string): boolean {
    // 管理员拥有所有权限
    if (isAdmin.value) return true;

    // 委托给 permission store 进行检查
    const permissionStore = usePermissionStore();
    return permissionStore.hasPermission(permission, currentRole.value);
  }

  return {
    // 状态
    token,
    refreshToken,
    userInfo,
    isLoading,

    // 计算属性
    isLoggedIn,
    isAdmin,
    currentRole,

    // 方法
    login,
    logout,
    fetchUserInfo,
    updateProfile,
    changePassword,
    refreshAccessToken,
    hasPermission,
    initAuth,
  };
});


/**
 * @file permission.ts
 * @description 权限状态管理 - 简化版
 * @date 2026-08-15
 */

import { useAuthStore } from "./auth";
import { PermissionScope, type PermissionChangeLog, type PermissionOption, getSensitivePermissions } from "@/types/permission";
import { getRolePermissionConfig, getRoleDataPermissions, updateRoleDataPermissions, getRoleFieldPermissions, updateRoleFieldPermissions, getRoleInheritChain, setRoleInherit, removeRoleInherit } from "@/api/permission";

interface DataPermissionConfig { permission: string; scope?: PermissionScope; custom_scope?: number[]; }
interface FieldPermissionConfig { field: string; entity_type: string; canView: boolean; canEdit: boolean; }

export interface MenuItem { id: number; name: string; path: string; icon?: string; children?: MenuItem[]; permissions?: string[]; }
export interface InheritPermissionInfo { role: string; inherit_from: string[]; effective_permissions: string[]; }
export interface PermissionInfo { user_id: number; role: string; permissions: string[]; menus: MenuItem[]; }

export const usePermissionStore = defineStore("permission", () => {
  const menus = ref<MenuItem[]>([]);
  const permissions = ref<string[]>([]);
  const role = ref<string>("");
  const isLoaded = ref(false);
  const dataPermissions = ref<DataPermissionConfig[]>([]);
  const fieldPermissions = ref<FieldPermissionConfig[]>([]);
  const inheritChain = ref<InheritPermissionInfo | null>(null);
  
  // 简化：直接使用 permissions.value，移除不必要的 effectivePermissions 计算
  const effectivePermissions = computed(() => permissions.value);
  
  const cacheExpiry = ref<number>(0);
  const CACHE_DURATION = 5 * 60 * 1000;
  const permissionChangeLogs = ref<PermissionChangeLog[]>([]);
  const permissionLogsTotal = ref(0);

  function hasPermission(permission: string, userRole?: string): boolean {
    const currentRole = userRole || role.value;
    if (currentRole === "admin") return true;
    return permissions.value.includes(permission);
  }

  function hasRole(targetRole: string): boolean { return role.value === targetRole; }

  function hasDataPermission(permission: string, resourceOwner?: number, resourceDepartment?: number): boolean {
    if (role.value === "admin") return true;
    const dataPerm = dataPermissions.value.find((dp) => dp.permission === permission);
    if (!dataPerm) return false; // 简化：未配置时默认拒绝
    
    const scope = dataPerm.scope;
    switch (scope) {
      case PermissionScope.ALL: return true;
      case PermissionScope.SELF: if (resourceOwner) return resourceOwner === getCurrentUserId(); return false;
      case PermissionScope.DEPARTMENT:
      case PermissionScope.DEPARTMENT_AND_CHILDREN: {
        const currentDepartmentId = (useAuthStore().userInfo as { department_id?: number })?.department_id;
        if (!currentDepartmentId || !resourceDepartment) return false;
        return resourceDepartment === currentDepartmentId;
      }
      case PermissionScope.CUSTOM:
        if (dataPerm.custom_scope && dataPerm.custom_scope.length > 0) {
          if (resourceDepartment) return dataPerm.custom_scope.includes(resourceDepartment);
          return false;
        }
        return false;
      default: return true;
    }
  }

  function hasFieldPermission(entity_type: string, field: string, action: "view" | "edit"): boolean {
    if (role.value === "admin") return true;
    const fieldPerm = fieldPermissions.value.find((fp) => fp.entity_type === entity_type && fp.field === field);
    if (!fieldPerm) return false; // 简化：未配置时默认拒绝
    return action === "view" ? fieldPerm.canView : fieldPerm.canEdit;
  }

  const getMenus = computed(() => menus.value);
  const isCacheValid = computed(() => isLoaded.value && Date.now() < cacheExpiry.value);

  function getCurrentUserId(): number {
    const authStore = useAuthStore();
    return authStore.userInfo?.id || 0;
  }

  async function fetchPermissions() {
    if (isCacheValid.value) return;
    try {
      const authStore = useAuthStore();
      if (!authStore.userInfo?.role) { isLoaded.value = true; cacheExpiry.value = Date.now() + CACHE_DURATION; return; }
      const response = await httpClient.get(`/admin/roles/${encodeURIComponent(authStore.userInfo.role)}/permissions`);
      const data = (response as { data?: string[] }).data;
      if (data) {
        role.value = authStore.userInfo?.role || "";
        permissions.value = data;
        menus.value = [];
        isLoaded.value = true;
        cacheExpiry.value = Date.now() + CACHE_DURATION;
      }
    } catch (error) {
      logger.error("【获取权限信息失败】", error);
      throw error;
    }
  }

  async function fetchRoleMenus(role_name: string) {
    try {
      const response = await httpClient.get("/admin/roles/" + role_name + "/menus");
      return (response as { data?: MenuItem[] }).data;
    } catch (error) {
      logger.error("【获取角色菜单失败】", error);
      throw error;
    }
  }

  async function fetchRolePermissionConfig(role_name: string) {
    try {
      const response = await getRolePermissionConfig(role_name);
      return (response as { data?: unknown }).data;
    } catch (error) {
      logger.error("【获取角色权限配置失败】", error);
      throw error;
    }
  }

  async function updateUserRole(user_id: number, userRole: string) {
    try {
      await httpClient.put("/admin/users/" + user_id + "/role", { role: userRole });
      return true;
    } catch (error) {
      logger.error("【更新用户角色失败】", error);
      throw error;
    }
  }

  function clearPermissions() {
    menus.value = []; permissions.value = []; role.value = ""; isLoaded.value = false;
    dataPermissions.value = []; fieldPermissions.value = []; inheritChain.value = null;
    permissionChangeLogs.value = []; cacheExpiry.value = 0;
  }

  function filterAccessibleMenus(allMenus: MenuItem[]): MenuItem[] {
    if (role.value === "admin") return allMenus;
    return allMenus.filter((menu) => {
      if (menu.permissions && menu.permissions.length > 0) {
        return menu.permissions.some((p) => permissions.value.includes(p));
      }
      return true;
    });
  }

  async function loadDataPermissions(role_name: string) {
    try { const response = await getRoleDataPermissions(role_name); dataPermissions.value = (response as { data?: DataPermissionConfig[] }).data || []; }
    catch (error) { logger.error("【加载数据权限失败】", error); dataPermissions.value = []; }
  }

  async function saveDataPermissions(role_name: string) {
    try { await updateRoleDataPermissions(role_name, dataPermissions.value); return true; }
    catch (error) { logger.error("【保存数据权限失败】", error); throw error; }
  }

  function setDataPermissionScope(permission: string, scope: PermissionScope, custom_scope?: number[]) {
    const existing = dataPermissions.value.find((dp) => dp.permission === permission);
    if (existing) {
      existing.scope = scope;
      if (custom_scope !== undefined) existing.custom_scope = custom_scope;
    } else {
      dataPermissions.value.push({ permission, scope, ...(custom_scope !== undefined ? { custom_scope } : {}) });
    }
  }

  function removeDataPermission(permission: string) {
    const index = dataPermissions.value.findIndex((dp) => dp.permission === permission);
    if (index > -1) dataPermissions.value.splice(index, 1);
  }

  async function loadFieldPermissions(role_name: string) {
    try { const response = await getRoleFieldPermissions(role_name); fieldPermissions.value = (response as { data?: FieldPermissionConfig[] }).data || []; }
    catch (error) { logger.error("【加载字段权限失败】", error); fieldPermissions.value = []; }
  }

  async function saveFieldPermissions(role_name: string) {
    try { await updateRoleFieldPermissions(role_name, fieldPermissions.value); return true; }
    catch (error) { logger.error("【保存字段权限失败】", error); throw error; }
  }

  function setFieldPermission(entity_type: string, field: string, canView: boolean, canEdit: boolean) {
    const existing = fieldPermissions.value.find((fp) => fp.entity_type === entity_type && fp.field === field);
    if (existing) { existing.canView = canView; existing.canEdit = canEdit; }
    else { fieldPermissions.value.push({ field, entity_type, canView, canEdit }); }
  }

  async function loadInheritChain(role_name: string) {
    try { const response = await getRoleInheritChain(role_name); inheritChain.value = (response as { data?: InheritPermissionInfo }).data ?? null; }
    catch (error) { logger.error("【加载继承链失败】", error); inheritChain.value = null; }
  }

  async function setInherit(role_name: string, inherit_from: string[]) {
    try { await setRoleInherit(role_name, inherit_from); await loadInheritChain(role_name); return true; }
    catch (error) { logger.error("【设置继承失败】", error); throw error; }
  }

  async function removeInherit(role_name: string) {
    try { await removeRoleInherit(role_name); inheritChain.value = null; return true; }
    catch (error) { logger.error("【移除继承失败】", error); throw error; }
  }

  async function loadPermissionChangeLogs(params?: { page?: number; page_size?: number; role_name?: string; operator?: string; change_type?: "add" | "remove" | "update"; start_date?: string; end_date?: string; }) {
    try { const response = await httpClient.get("/permission-change-logs", { params }); const permData = (response as { data?: { list: PermissionChangeLog[]; total: number } }).data; permissionChangeLogs.value = permData?.list || []; permissionLogsTotal.value = permData?.total || 0; }
    catch (error) { logger.error("【加载权限变更日志失败】", error); permissionChangeLogs.value = []; permissionLogsTotal.value = 0; }
  }

  const permissionHistory = ref<Array<{ role: string; before: string[]; after: string[]; timestamp: number }>>([]);

  function recordPermissionChange(role_name: string, before: string[], after: string[]) {
    permissionHistory.value.push({ role: role_name, before, after, timestamp: Date.now() });
    if (permissionHistory.value.length > 50) permissionHistory.value.shift();
  }

  function getPermissionHistory(role_name?: string) {
    if (role_name) return permissionHistory.value.filter((h) => h.role === role_name);
    return permissionHistory.value;
  }

  function getAllSensitivePermissions(): PermissionOption[] { return getSensitivePermissions(); }
  function isSensitivePermission(permission: string): boolean { return getSensitivePermissions().some(p => p.key === permission); }

  return {
    menus, permissions, role, isLoaded, dataPermissions, fieldPermissions, inheritChain, effectivePermissions, permissionChangeLogs, permissionLogsTotal,
    getMenus, isCacheValid, hasPermission, hasRole, hasDataPermission, hasFieldPermission,
    fetchPermissions, fetchRoleMenus, fetchRolePermissionConfig, updateUserRole, clearPermissions, filterAccessibleMenus,
    loadDataPermissions, saveDataPermissions, setDataPermissionScope, removeDataPermission,
    loadFieldPermissions, saveFieldPermissions, setFieldPermission,
    loadInheritChain, setInherit, removeInherit,
    loadPermissionChangeLogs, recordPermissionChange, getPermissionHistory,
    getAllSensitivePermissions, isSensitivePermission,
  };
});


/**
 * @file notification.ts
 * @description 通知状态管理
 * @date 2026-04-03
 */

import {
  listNotificationRecords,
  markNotificationAsRead,
  markAllNotificationsAsRead,
  deleteNotification,
  deleteNotifications,
} from '@/api/notification';
import type { Notification, NotificationType } from '@/types/notification';

export const useNotificationStore = defineStore('notification', () => {
  // ============ 状态定义 ============

  /** 通知列表 */
  const notifications = ref<Notification[]>([]);

  /** 未读数量 */
  const unreadCount = ref(0);

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
    type: NotificationType | null;
    is_read: boolean | null;
  }>({
    type: null,
    is_read: null,
  });

  // ============ 计算属性 ============

  /** 是否全部已读 */
  const isAllRead = computed(() => unreadCount.value === 0);

  /** 最近通知（最新5条） */
  const recentNotifications = computed(() => notifications.value.slice(0, 5));

  // ============ Actions ============

  /**
   * @brief 获取通知列表
   */
  async function fetchNotifications() {
    isLoading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        type?: NotificationType;
        is_read?: boolean;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      };
      if (filters.value.type !== null) {
        params.type = filters.value.type ?? undefined;
      }
      if (filters.value.is_read !== null) {
        params.is_read = filters.value.is_read ?? undefined;
      }
      const response = await listNotificationRecords(params);

      const data = (response as { data?: { list: Notification[]; total: number } }).data;
      if (data) {
        notifications.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取通知列表失败】', error);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取未读数量
   */
  async function fetchUnreadCount() {
    try {
      const response = await listNotificationRecords({ page_size: 1 });
      unreadCount.value = (response as { data?: { total?: number } }).data?.total || 0;
    } catch (error) {
      logger.error('【获取未读数量失败】', error);
    }
  }

  /**
   * @brief 标记单条已读
   * @param id - 通知ID
   */
  async function markAsRead(id: number) {
    try {
      await markNotificationAsRead(id);
      const notification = notifications.value.find((n) => n.id === id);
      if (notification && !notification.is_read) {
        notification.is_read = true;
        notification.read_at = new Date().toISOString();
        unreadCount.value = Math.max(0, unreadCount.value - 1);
      }
    } catch (error) {
      logger.error('【标记已读失败】', error);
      throw error;
    }
  }

  /**
   * @brief 全部标记已读
   */
  async function markAllAsRead() {
    try {
      await markAllNotificationsAsRead();
      notifications.value.forEach((n) => {
        n.is_read = true;
        n.read_at = new Date().toISOString();
      });
      unreadCount.value = 0;
    } catch (error) {
      logger.error('【全部标记已读失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除通知
   * @param id - 通知ID
   */
  async function deleteNotificationById(id: number) {
    try {
      await deleteNotification(id);
      const index = notifications.value.findIndex((n) => n.id === id);
      if (index !== -1) {
        const notification = notifications.value[index];
        if (notification && !notification.is_read) {
          unreadCount.value = Math.max(0, unreadCount.value - 1);
        }
        notifications.value.splice(index, 1);
        pagination.value.total = Math.max(0, pagination.value.total - 1);
      }
    } catch (error) {
      logger.error('【删除通知失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量删除
   * @param ids - 通知ID数组
   */
  async function batchDelete(ids: number[]) {
    try {
      await deleteNotifications(ids);
      const deletedSet = new Set(ids);
      notifications.value = notifications.value.filter((n) => {
        if (deletedSet.has(n.id) && !n.is_read) {
          unreadCount.value = Math.max(0, unreadCount.value - 1);
          return false;
        }
        return !deletedSet.has(n.id);
      });
      pagination.value.total = Math.max(0, pagination.value.total - ids.length);
    } catch (error) {
      logger.error('【批量删除失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量标记已读
   * @param ids - 通知ID数组
   */
  async function batchMarkRead(ids: number[]) {
    try {
      // 使用 markNotificationAsRead 逐条标记
      await Promise.all(ids.map(id => markNotificationAsRead(id)));
      const idSet = new Set(ids);
      let count = 0;
      notifications.value.forEach((n) => {
        if (idSet.has(n.id) && !n.is_read) {
          n.is_read = true;
          n.read_at = new Date().toISOString();
          count++;
        }
      });
      unreadCount.value = Math.max(0, unreadCount.value - count);
    } catch (error) {
      logger.error('【批量标记已读失败】', error);
      throw error;
    }
  }

  /**
   * @brief 添加新通知（WebSocket 推送时调用）
   * @param notification - 新通知
   */
  function addNotification(notification: Notification) {
    notifications.value.unshift(notification);
    if (!notification.is_read) {
      unreadCount.value++;
    }
    pagination.value.total++;
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
    filters.value = { type: null, is_read: null };
    pagination.value.page = 1;
  }

  return {
    // 状态
    notifications,
    unreadCount,
    isLoading,
    pagination,
    filters,

    // 计算属性
    isAllRead,
    recentNotifications,

    // 方法
    fetchNotifications,
    fetchUnreadCount,
    markAsRead,
    markAllAsRead,
    deleteNotification: deleteNotificationById,
    batchDelete,
    batchMarkRead,
    addNotification,
    setFilters,
    resetFilters,
  };
});





/**
 * @file theme.ts
 * @description 主题状态管理
 * @date 2026-04-02
 * @note 整合了 useTheme.ts 的部分功能，避免重复代码
 */

import { useQuasar } from 'quasar';
import { getStorageItem, setStorageItem } from '@/utils/storage';

export type ThemeMode = 'light' | 'dark' | 'auto';

/**
 * @brief 预设主题颜色接口
 */
export interface PresetTheme {
  name: string;
  primary: string;
  secondary: string;
  accent: string;
}

/**
 * @brief 主题模式选项
 */
export const themeModeOptions = [
  { label: '浅色模式', value: 'light' as ThemeMode, icon: 'light_mode' },
  { label: '深色模式', value: 'dark' as ThemeMode, icon: 'dark_mode' },
  { label: '跟随系统', value: 'auto' as ThemeMode, icon: 'contrast' },
];

/**
 * @brief 预设主题列表
 */
export const presetThemes: PresetTheme[] = [
  { name: 'blue', primary: '#1976d2', secondary: '#26a69a', accent: '#ff6f00' },
  { name: 'purple', primary: '#7b1fa2', secondary: '#00bcd4', accent: '#ffc107' },
  { name: 'green', primary: '#388e3c', secondary: '#7b1fa2', accent: '#ff5722' },
  { name: 'orange', primary: '#f57c00', secondary: '#0288d1', accent: '#7b1fa2' },
  { name: 'red', primary: '#d32f2f', secondary: '#0288d1', accent: '#ffc107' },
  { name: 'teal', primary: '#00796b', secondary: '#f57c00', accent: '#7b1fa2' },
];

export const useThemeStore = defineStore('theme', () => {
  const $q = useQuasar();

  // 获取保存的主题模式，默认为 'auto'
  const savedMode = (getStorageItem<ThemeMode>('theme_mode', 'auto') || 'auto');
  const mode = ref<ThemeMode>(savedMode);

  // 主题颜色配置
  const primaryColor = ref<string>('#1976d2');
  const secondaryColor = ref<string>('#26a69a');
  const accentColor = ref<string>('#ff6f00');

  // 计算当前实际主题
  const isDark = ref<boolean>(
    mode.value === 'dark' || (mode.value === 'auto' && $q.dark.isActive)
  );

  // 监听系统主题变化
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

  /**
   * @brief 处理系统主题变化
   */
  function handleSystemThemeChange() {
    if (mode.value === 'auto') {
      isDark.value = mediaQuery.matches;
      applyThemeColors();
    }
  }

  /**
   * @brief 应用主题颜色到 CSS 变量
   */
  function applyThemeColors() {
    const root = document.documentElement;
    root.style.setProperty('--q-primary', primaryColor.value);
    root.style.setProperty('--q-secondary', secondaryColor.value);
    root.style.setProperty('--q-accent', accentColor.value);
  }

  /**
   * @brief 从 localStorage 加载主题配置
   */
  function loadThemeConfig() {
    const saved = getStorageItem<string>('themeConfig', '');
    if (saved) {
      try {
        const config = JSON.parse(saved);
        if (config.primaryColor) primaryColor.value = config.primaryColor;
        if (config.secondaryColor) secondaryColor.value = config.secondaryColor;
        if (config.accentColor) accentColor.value = config.accentColor;
      } catch {
        // 忽略解析错误
      }
    }
  }

  // 初始化
  function init() {
    // 加载主题配置
    loadThemeConfig();
    // 应用保存的主题
    applyMode(mode.value);
    // 应用主题颜色
    applyThemeColors();
    // 添加媒体查询监听器
    mediaQuery.addEventListener('change', handleSystemThemeChange);
  }

  // 应用主题模式
  function applyMode(newMode: ThemeMode) {
    mode.value = newMode;
    setStorageItem('theme_mode', newMode);

    switch (newMode) {
      case 'light':
        $q.dark.set(false);
        isDark.value = false;
        break;
      case 'dark':
        $q.dark.set(true);
        isDark.value = true;
        break;
      case 'auto':
        isDark.value = mediaQuery.matches;
        break;
    }
  }

  // 切换主题
  function toggleTheme() {
    if (mode.value === 'light') {
      applyMode('dark');
    } else if (mode.value === 'dark') {
      applyMode('light');
    } else {
      // auto 模式下，切换到与当前系统相反的主题
      applyMode(mediaQuery.matches ? 'light' : 'dark');
    }
  }

  // 设置特定模式
  function setMode(newMode: ThemeMode) {
    applyMode(newMode);
  }

  /**
   * @brief 设置主题颜色
   * @param preset - 预设主题
   */
  function setThemeColor(preset: PresetTheme) {
    primaryColor.value = preset.primary;
    secondaryColor.value = preset.secondary;
    accentColor.value = preset.accent;
    applyThemeColors();
    // 保存到 localStorage
    setStorageItem('themeConfig', {
      primaryColor: primaryColor.value,
      secondaryColor: secondaryColor.value,
      accentColor: accentColor.value,
    });
  }

  return {
    mode,
    isDark,
    primaryColor,
    secondaryColor,
    accentColor,
    init,
    toggleTheme,
    setMode,
    setThemeColor,
    applyThemeColors,
  };
});
