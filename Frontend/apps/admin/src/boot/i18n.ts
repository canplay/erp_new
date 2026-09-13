import { defineBoot } from '#q-app';
import { createI18n } from 'vue-i18n';

import messages from '@/i18n';

export type MessageLanguages = keyof typeof messages;

// 默认中文；保留 en-US
export default defineBoot(({ app }) => {
  const i18n = createI18n({
    locale: 'zh-CN',
    fallbackLocale: 'en-US',
    legacy: false,
    messages,
  });

  app.use(i18n);
});
