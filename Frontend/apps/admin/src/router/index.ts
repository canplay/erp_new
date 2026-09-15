import { defineRouter } from '#q-app';
import {
  createMemoryHistory,
  createRouter,
  createWebHashHistory,
  createWebHistory,
} from 'vue-router';
import { isAuthenticated } from '@erp-new-frontend-monorepo/boot/alova';
import { hasCap } from '@erp-new-frontend-monorepo/capabilities';

import MainLayout from '@/layouts/MainLayout.vue';

export default defineRouter(() => {
  const createHistory = import.meta.env.QUASAR_SERVER
    ? createMemoryHistory
    : import.meta.env.QUASAR_VUE_ROUTER_MODE === 'history'
      ? createWebHistory
      : createWebHashHistory;

  const Router = createRouter({
    scrollBehavior: () => ({ left: 0, top: 0 }),
    routes: [
      {
        path: '/auth/login',
        name: 'login',
        component: () => import('@/pages/auth/login/index.vue'),
      },
      {
        path: '/auth/register',
        name: 'register',
        component: () => import('@/pages/auth/register/index.vue'),
      },
      {
        path: '/',
        component: MainLayout,
        children: [
          { path: '', name: 'index', component: () => import('@/pages/index.vue') },
          {
            path: 'equipment',
            name: 'equipment',
            component: () => import('@/pages/equipment/index.vue'),
            meta: { cap: 'equipment' },
          },
          {
            path: 'equipment/:id',
            name: 'equipment-detail',
            component: () => import('@/pages/equipment/detail/index.vue'),
            meta: { cap: 'equipment' },
          },
          {
            path: 'daily-control',
            name: 'daily-control',
            component: () => import('@/pages/daily-control/index.vue'),
            meta: { cap: 'daily-control' },
          },
          {
            path: 'weekly-check',
            name: 'weekly-check',
            component: () => import('@/pages/weekly-check/index.vue'),
            meta: { cap: 'weekly-check' },
          },
          {
            path: 'monthly-report',
            name: 'monthly-report',
            component: () => import('@/pages/monthly-report/index.vue'),
            meta: { cap: 'monthly-report' },
          },
          {
            path: 'analytics',
            name: 'analytics',
            component: () => import('@/pages/analytics/index.vue'),
            meta: { cap: 'analytics' },
          },
          {
            path: 'hidden-danger',
            name: 'hidden-danger',
            component: () => import('@/pages/hidden-danger/index.vue'),
            meta: { cap: 'hidden-danger' },
          },
          {
            path: 'personnel',
            name: 'personnel',
            component: () => import('@/pages/personnel/index.vue'),
            meta: { cap: 'personnel' },
          },
          {
            path: 'regulatory',
            name: 'regulatory',
            component: () => import('@/pages/regulatory/index.vue'),
            meta: { cap: 'regulatory' },
          },
          {
            path: 'knowledge-graph',
            name: 'knowledge-graph',
            component: () => import('@/pages/knowledge-graph/index.vue'),
            meta: { cap: 'knowledge-graph' },
          },
          {
            path: 'users',
            name: 'users',
            component: () => import('@/pages/users/index.vue'),
            meta: { cap: 'users-platform' },
          },
          {
            path: 'users/:id',
            name: 'user-detail',
            component: () => import('@/pages/users/detail/index.vue'),
            meta: { cap: 'users-platform' },
          },
          {
            path: 'roles',
            name: 'roles',
            component: () => import('@/pages/roles/index.vue'),
            meta: { cap: 'roles' },
          },
          {
            path: 'roles/:id',
            name: 'role-detail',
            component: () => import('@/pages/roles/detail/index.vue'),
            meta: { cap: 'roles' },
          },
          {
            path: 'tenants',
            name: 'tenants',
            component: () => import('@/pages/tenants/index.vue'),
            meta: { cap: 'tenant-management' },
          },
          {
            path: 'tenants/:id',
            name: 'tenant-detail',
            component: () => import('@/pages/tenants/detail/index.vue'),
            meta: { cap: 'tenant-management' },
          },
          {
            path: 'audits',
            name: 'audits',
            component: () => import('@/pages/audits/index.vue'),
            meta: { cap: 'audit-platform' },
          },
          {
            path: 'billing',
            name: 'billing',
            component: () => import('@/pages/billing/index.vue'),
            meta: { cap: 'billing' },
          },
          {
            path: 'billing/invoices',
            name: 'billing-invoices',
            component: () => import('@/pages/billing/invoices/index.vue'),
            meta: { cap: 'billing' },
          },
          {
            path: 'billing/invoices/:id',
            name: 'invoice-detail',
            component: () => import('@/pages/billing/invoice-detail/index.vue'),
            meta: { cap: 'billing' },
          },
          {
            path: 'webhooks',
            name: 'webhooks',
            component: () => import('@/pages/webhooks/index.vue'),
            meta: { cap: 'webhooks' },
          },
          {
            path: 'webhooks/:id',
            name: 'webhook-detail',
            component: () => import('@/pages/webhooks/detail/index.vue'),
            meta: { cap: 'webhooks' },
          },
          {
            path: 'messages',
            name: 'messages',
            component: () => import('@/pages/messages/index.vue'),
            meta: { cap: 'messages' },
          },
          {
            path: 'impersonation',
            name: 'impersonation',
            component: () => import('@/pages/impersonation/index.vue'),
            meta: { cap: 'impersonation' },
          },
          {
            path: 'files',
            name: 'files',
            component: () => import('@/pages/files/index.vue'),
            meta: { cap: 'files' },
          },
          {
            path: 'ocr',
            name: 'ocr',
            component: () => import('@/pages/ocr/index.vue'),
            meta: { cap: 'ocr' },
          },
          {
            path: 'groups',
            name: 'groups',
            component: () => import('@/pages/groups/index.vue'),
            meta: { cap: 'user-groups' },
          },
          {
            path: 'catalog',
            name: 'catalog',
            component: () => import('@/pages/catalog/index.vue'),
            meta: { cap: 'catalog' },
          },
          {
            path: 'tickets',
            name: 'tickets',
            component: () => import('@/pages/tickets/index.vue'),
            meta: { cap: 'tickets' },
          },
          {
            path: 'health',
            name: 'health',
            component: () => import('@/pages/health/index.vue'),
            meta: { cap: 'health' },
          },
        ],
      },
      {
        path: '/:pathMatch(.*)*',
        name: 'not-found',
        component: () => import('@/pages/not-found/index.vue'),
      },
    ],

    // Leave this as is and make changes in quasar.conf.js instead!
    // quasar.conf.js -> build -> vueRouterMode
    // quasar.conf.js -> build -> publicPath
    history: createHistory(import.meta.env.QUASAR_VUE_ROUTER_BASE),
  });

  // 登录守卫：未登录访问非登录页 → /auth/login；已登录访问登录页 → /
  const publicPaths = [
    '/auth/login',
    '/auth/register',
  ];
  Router.beforeEach((to) => {
    const authed = isAuthenticated();
    if (publicPaths.some((p) => to.path.startsWith(p))) {
      return authed ? '/' : true;
    }
    if (!authed) return '/auth/login';
    // 能力门控：平台视角无该能力（tenant 专有路由）→ 回首页
    const cap = (to.meta as { cap?: string })?.cap;
    if (cap && !hasCap(cap, 'platform')) return '/';
    return true;
  });

  return Router;
});
