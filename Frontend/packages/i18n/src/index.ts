// @erp-new-frontend-monorepo/i18n
// Unified i18n for all apps

import { createI18n } from 'vue-i18n'

// Import locale files
import zhCN from './zh-CN/index'
import enUS from './en-US/index'

// Export message objects for merging in apps
export const messages = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

// Create i18n instance
const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en-US',
  messages,
})

export default i18n
export { zhCN, enUS }
