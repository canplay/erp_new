import { defineStore } from '#q-app';
import { createPinia } from 'pinia';

/*
 * admin 端 Pinia 安装（Quasar stores 约定）。
 * MainLayout 使用共享 useAuthStore/useAppStore（@erp-new-frontend-monorepo/stores），
 * 缺此目录时生产构建 useAuthStore() 抛 "no active pinia" → 业务页白屏（dev server 容错掩盖）。
 */

export default defineStore((/* { ssrContext } */) => {
  const pinia = createPinia();

  // 可在此添加 Pinia 插件
  // pinia.use(SomePiniaPlugin)

  return pinia;
});
