import { defineBoot } from '#q-app';
import { Notify } from 'quasar';
import { setupAlova, setErrorNotifier } from '@erp-new-frontend-monorepo/boot/alova';

export default defineBoot(async (/* { app, router, ... } */) => {
  // 全局 API 错误提示（非 401 错误统一 toast）
  setErrorNotifier((msg) => {
    Notify.create({ type: 'negative', message: msg, timeout: 4000 });
  });
  await setupAlova();
});
