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
