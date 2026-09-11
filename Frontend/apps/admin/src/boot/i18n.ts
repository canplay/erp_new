import { defineBoot } from '#q-app';
import { createI18n } from 'vue-i18n';

import messages from '@/i18n';

export type MessageLanguages = keyof typeof messages;

// 修复 (fix-plan-20260806 P17): 原硬编码 locale: 'zh-CN'，
// 启动时读取 localStorage 的 user-locale（LanguageSwitcher 写入），
// 使语言切换在刷新后保持。仅接受实际有翻译资源的语言码。
const SUPPORTED_LOCALES = Object.keys(messages);
const savedLocale = typeof localStorage !== 'undefined'
  ? localStorage.getItem('user-locale')
  : null;
const initialLocale = savedLocale && SUPPORTED_LOCALES.includes(savedLocale)
  ? savedLocale
  : 'zh-CN';

export default defineBoot(({ app }) => {
  const i18n = createI18n({
    locale: initialLocale,
    legacy: false,
    fallbackLocale: 'zh-CN',
    messages,
  });

  // Set i18n instance on app
  (app as { use: (plugin: object) => void }).use(i18n);
});
