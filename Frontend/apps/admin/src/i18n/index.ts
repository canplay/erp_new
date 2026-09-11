/**
 * @file index.ts
 * @description 国际化配置入口
 * @date 2026-04-02
 * @updated 2026-04-07 修复模块导入问题
 */

import zhCN from './zh-CN';
import enUS from './en-US';

export const messages = {
  'zh-CN': zhCN,
  'en-US': enUS,
};

export const languages = [
  { label: '中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' },
];

export default messages;
