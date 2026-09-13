/**
 * @file index.ts
 * @description 路由配置和导航守卫
 * @date 2026-04-02
 */

import type { RouteLocationNormalized } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { usePermissionStore } from '@/stores/auth';
import { logger } from '@/utils/logger';
import { getStorageItem } from '@/utils/storage';
import router from './routes';

declare module 'vue-router' {
  interface RouteMeta {
    // 是否需要登录
    requiresAuth?: boolean;
    // 需要的权限
    permissions?: string[];
    // 页面标题
    title?: string;
    // 菜单项（用于侧边栏渲染）
    menuItem?: { name: string; label: string; path: string; icon: string; permission?: string[] };
  }
}

/**
 * @brief 路由预加载配置
 * @description 预加载常用页面组件，提升页面切换速度
 * @note 修复 fetchUserInfo 竞态条件：在 initAuth 中统一恢复 userInfo
 */
function setupPreload() {
  // 预加载关键页面
  const preloadRoutes = ['Dashboard', 'Profile', 'UserList'];

  // 立即预加载，避免阻塞主线程
  preloadRoutes.forEach((routeName) => {
    const routes = router.getRoutes().filter((r) => r.name === routeName);
    routes.forEach((r) => {
      if (r.components?.default) {
        const component = r.components.default as { preload?: () => void };
        if (typeof component.preload === 'function') {
          component.preload();
        }
      }
    });
  });
  logger.info('【路由预加载】关键页面组件已预加载');
}

/**
 * @brief 路由初始化配置
 */
export function setupRouter() {
  // 初始化路由预加载
  setupPreload();

  // 全局前置守卫 - 路由鉴权
  router.beforeEach(async (
    to: RouteLocationNormalized,
    _from: RouteLocationNormalized,
    next: (to?: string | { path: string; query?: Record<string, string> }) => void
  ) => {
    const authStore = useAuthStore();

    // 更新页面标题
    if (to.meta.title) {
      document.title = `${to.meta.title} - 管理后台`;
      // 使用 Quasar 的 title 管理：document.title 是最可靠的方案
    } else {
      document.title = '管理后台';
    }

    // 需要登录的页面
    if (to.meta.requiresAuth) {
      if (!authStore.isLoggedIn) {
        // 未登录，跳转登录页
        next({
          path: '/login',
          query: { redirect: to.fullPath },
        });
        return;
      }

      // 初始化用户信息（如果还没有）
      if (!authStore.userInfo) {
        // 等待 permissionStore 加载完成，避免竞态条件
        const permissionStore = usePermissionStore();
        if (!permissionStore.isLoaded) {
          await permissionStore.fetchPermissions();
        }
        try {
          await authStore.fetchUserInfo();
        } catch {
          // 获取失败，可能是 token 过期
          next({
            path: '/login',
            query: { redirect: to.fullPath },
          });
          return;
        }
      }

      // 审计修复(C1-权限初始化): 登录后首次进入页面时预加载角色权限。
      // 等待 isLoaded 标志避免竞态; 失败时静默降级, 不阻塞路由导航。
      const permissionStore = usePermissionStore();
      if (!permissionStore.isLoaded) {
        try {
          await permissionStore.fetchPermissions();
        } catch (error) {
          logger.warn('【权限预加载失败】已静默降级, 不阻塞路由', error);
        }
      }

      // 首次登录需修改密码，跳转到个人中心
      if (getStorageItem<string>('must_change_password', '') === 'true' && to.path !== '/profile') {
        next({ path: '/profile' });
        return;
      }

      // 检查权限
      if (to.meta.permissions && to.meta.permissions.length > 0) {
        const hasPermission = to.meta.permissions.some(
          (perm: string) => authStore.hasPermission(perm)
        );
        if (!hasPermission) {
          logger.warn('【权限不足】', to.path);
          // 可以跳转到 403 页面
          next({ path: '/403' });
          return;
        }
      }
    }

    // 已登录访问登录页，跳转到首页
    if (to.path === '/login' && authStore.isLoggedIn) {
      next('/');
      return;
    }

    next();
  });

  // 后置守卫 - 页面加载完成后
  router.afterEach((to: RouteLocationNormalized) => {
    // 可以在这里记录访问日志
    if (to.meta.title) {
      logger.info('【页面访问】', to.meta.title);
    }
  });

  return router;
}

export { router };
export default router;

