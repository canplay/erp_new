import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      // 默认重定向到房间列表
      { path: '', redirect: { path: '/rooms' } },
    ],
  },

  // 登录页 - 独立布局
  {
    path: '/login',
    name: 'login',
    component: () => import('@/pages/LoginPage.vue'),
  },

  // 主应用 - 需要登录
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      {
        path: 'rooms',
        name: 'rooms',
        component: () => import('@/pages/RoomListPage.vue'),
        meta: { title: '聊天室', icon: 'chat' },
      },
      {
        path: 'chat/:roomId',
        name: 'chat',
        component: () => import('@/pages/ChatPage.vue'),
        props: true,
        meta: { title: '聊天', icon: 'chat' },
      },
      {
        path: 'contacts',
        name: 'contacts',
        component: () => import('@/pages/ContactPage.vue'),
        meta: { title: '通讯录', icon: 'contacts' },
      },
      {
        path: 'moments',
        name: 'moments',
        component: () => import('@/pages/MomentsPage.vue'),
        meta: { title: '朋友圈', icon: 'photo' },
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@/pages/SettingsPage.vue'),
        meta: { title: '设置', icon: 'settings' },
      },
    ],
  },

  // 404 页面 - 始终放在最后
  {
    path: '/:catchAll(.*)*',
    component: () => import('@/pages/ErrorNotFound.vue'),
  },
];

export default routes;
