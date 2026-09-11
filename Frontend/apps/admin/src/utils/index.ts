/**
 * @file index.ts
 * @description 通用工具函数（统一导出入口）
 * @date 2026-04-03
 *
 * 拆分说明：
 *   string.ts  — isEmpty, truncate, capitalize, randomString
 *   format.ts  — formatDate, relativeTime, formatNumber, formatFileSize
 *   validate.ts — isValidEmail, isValidPhone, isValidUrl
 *   object.ts  — deepClone, pick, omit, debounce, throttle
 *   storage.ts — getStorageItem, setStorageItem, removeStorageItem
 *   privacy.ts — 隐私脱敏
 *   logger.ts  — 日志工具
 *   sanitize.ts — 输入消毒
 */

export * from './privacy';
export * from './logger';
export * from './sanitize';
export * from './string';
export * from './format';
export * from './validate';
export * from './object';
export * from './storage';
