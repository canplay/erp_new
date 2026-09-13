import enUS from './en-US';
import zhCN from './zh-CN';

// 合并共享包领域术语 + 本应用词条
import { sharedMessages } from '@erp-new-frontend-monorepo/i18n';

export default {
  'en-US': { ...sharedMessages['en-US'], ...enUS },
  'zh-CN': { ...sharedMessages['zh-CN'], ...zhCN },
};
