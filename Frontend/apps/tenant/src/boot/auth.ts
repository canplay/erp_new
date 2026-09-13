/**
 * @file auth.ts
 * @description 认证状态初始化 boot 文件 - 页面加载时恢复登录态
 * @date 2026-08-01
 */

import { defineBoot } from '#q-app';
import { useAuthStore } from '@/stores/auth';
import { setupRouter } from '@/router';
import { logger } from '@/utils/logger';

export default defineBoot(async () => {
  const authStore = useAuthStore();

  // 审计修复(C1-权限初始化): 注册全局路由守卫。
  // 此前 setupRouter() 无任何调用方, 导致 beforeEach 鉴权守卫(含权限预加载)从未生效。
  setupRouter();

  // 页面刷新时恢复认证状态（从 localStorage 读取 token）
  await authStore.initAuth();

  // 如果 token 有效但 userInfo 为空，说明路由守卫还没触发过，
  // 此时需要确保后续导航能正确通过守卫检查
  if (authStore.isLoggedIn && !authStore.userInfo) {
    try {
      await authStore.fetchUserInfo();
    } catch (error) {
      logger.warn('【获取用户信息失败，可能是 token 过期】', error);
    }
  }
});
