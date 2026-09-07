import { defineRouter } from '#q-app';
import { createMemoryHistory, createRouter, createWebHashHistory, createWebHistory } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import routes from './routes';

/*
 * If not building with SSR mode, you can
 * directly export the Router instantiation;
 *
 * The function below can be async too; either use
 * async/await or return a Promise which resolves
 * with the Router instance.
 */

export default defineRouter((/* { store, ssrContext } */) => {
  const createHistory = import.meta.env.QUASAR_SERVER
    ? createMemoryHistory
    : import.meta.env.QUASAR_VUE_ROUTER_MODE === 'history'
      ? createWebHistory
      : createWebHashHistory;

  const Router = createRouter({
    scrollBehavior: () => ({ left: 0, top: 0 }),
    routes,

    // Leave this as is and make changes in quasar.conf.js instead!
    // quasar.conf.js -> build -> vueRouterMode
    // quasar.conf.js -> build -> publicPath
    history: createHistory(import.meta.env.QUASAR_VUE_ROUTER_BASE),
  });

  // --- Navigation guard: 认证检查 ---
  Router.beforeEach(async (to, from, next) => {
    const auth = useAuthStore();

    // 需要登录才能访问的路由
    const protectedRoutes = ['rooms', 'chat', 'contacts', 'moments', 'settings'];

    if (typeof to.name === 'string' && protectedRoutes.includes(to.name)) {
      if (!auth.isLoggedIn) {
        // 尝试从 localStorage 恢复会话
        const savedToken = localStorage.getItem('matrix_access_token');
        const savedUserId = localStorage.getItem('matrix_user_id');
        const savedDeviceId = localStorage.getItem('matrix_device_id');
        const savedRefreshToken = localStorage.getItem('matrix_refresh_token');

        if (savedToken && savedUserId && savedDeviceId) {
          try {
            await auth.restoreSession(savedToken, savedUserId, savedDeviceId, savedRefreshToken ?? undefined);
            next();
            return;
          } catch {
            // 尝试用 refresh_token 恢复
            const refreshed = await auth.tryRefreshToken().catch(() => false);
            if (refreshed) {
              next();
              return;
            }
            // 全部失败，跳转到登录页
            next({ path: '/login', query: { redirect: to.fullPath } });
            return;
          }
        }

        next({ path: '/login', query: { redirect: to.fullPath } });
        return;
      }
    }

    // 已登录用户访问登录页时重定向到首页
    if (to.path === '/login' && auth.isLoggedIn) {
      next({ path: '/rooms' });
      return;
    }

    next();
  });

  return Router;
});
