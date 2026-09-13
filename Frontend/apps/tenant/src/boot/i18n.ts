import { defineBoot } from '#q-app';
import { createI18n } from 'vue-i18n';

import { messages as sharedMessagesRaw } from '@erp-new-frontend-monorepo/i18n';
import appMessages from '@/i18n';

// Merge shared messages with app-specific messages
// App-specific messages take precedence
function mergeMessages(shared: Record<string, Record<string, string>>, app: Record<string, Record<string, string>>): Record<string, Record<string, string>> {
  const result: Record<string, Record<string, string>> = {};
  for (const locale of Object.keys(shared)) {
    result[locale] = { ...shared[locale], ...(app[locale] || {}) };
  }
  return result;
}

const sharedMessages = sharedMessagesRaw as unknown as Record<string, Record<string, string>>;

export type MessageLanguages = keyof typeof sharedMessages;

const SAVED_LOCALE_KEY = 'user-locale';
const SUPPORTED_LOCALES = Object.keys(sharedMessages);
const savedLocale = typeof localStorage !== 'undefined'
  ? localStorage.getItem(SAVED_LOCALE_KEY)
  : null;
const initialLocale = savedLocale && SUPPORTED_LOCALES.includes(savedLocale)
  ? savedLocale
  : 'zh-CN';

export default defineBoot(({ app }) => {
  const merged = mergeMessages(sharedMessages, appMessages);
  const i18n = createI18n({
    locale: initialLocale,
    legacy: false,
    fallbackLocale: 'zh-CN',
    messages: merged,
  });

  (app as { use: (plugin: object) => void }).use(i18n);
});
