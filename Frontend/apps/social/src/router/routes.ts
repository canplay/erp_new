import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  // 登录页 - 独立布局
  {
    path: '/login',
    name: 'login',
    component: () => import('@erp-new-frontend-monorepo/pages/src/LoginPage.vue'),
    meta: { title: '登录', requiresAuth: false, cap: 'auth:login' },
  },

  // 主应用 - 需要登录
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      // 默认重定向到房间列表
      { path: '', redirect: '/rooms' },
      {
        path: 'rooms',
        name: 'rooms',
        component: () => import('@erp-new-frontend-monorepo/pages/src/RoomListPage.vue'),
        meta: { title: '聊天室', icon: 'chat', cap: 'social:chat:view' },
      },
      {
        path: 'chat/:roomId',
        name: 'chat',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ChatPage.vue'),
        props: true,
        meta: { title: '聊天', icon: 'chat', cap: 'social:chat:view' },
      },
      {
        path: 'contacts',
        name: 'contacts',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ContactPage.vue'),
        meta: { title: '通讯录', icon: 'contacts', cap: 'social:contact:list' },
      },
      {
        path: 'moments',
        name: 'moments',
        component: () => import('@erp-new-frontend-monorepo/pages/src/MomentsPage.vue'),
        meta: { title: '朋友圈', icon: 'photo', cap: 'social:moments:view' },
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@erp-new-frontend-monorepo/pages/src/SettingsPage.vue'),
        meta: { title: '设置', icon: 'settings', cap: 'settings:view' },
      },
    ],
  },

  // 404 页面 - 始终放在最后
  {
    path: '/:catchAll(.*)*',
    component: () => import('@erp-new-frontend-monorepo/pages/src/ErrorNotFound.vue'),
  },
];

export default routes;
