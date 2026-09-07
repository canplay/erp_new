/**
 * @file index.ts
 * @description 类型定义导出索引
 * @date 2026-04-04
 */

// 导出 API 通用类型
export * from './api';

// 导出用户相关类型 (不包含 LoginLog，避免与 log.ts 冲突)
export type { UserInfo, User, Role, Statistics } from './user';
export type { LoginForm } from './user';

// 导出日志相关类型 (包含 LoginLog 和 LoginLogParams)
export * from './log';

// 导出通知相关类型
export * from './notification';

// 导出 Socket 相关类型
export * from './socket';

// 导出权限相关类型
export * from './permission';

// 导出行级权限类型
export type { RowPermission } from './rowPermission';

// 导出字段脱敏类型
export type { FieldMaskRule, FieldMaskResult } from './fieldMask';
export { MaskStrategy } from './fieldMask';

// 导出导入导出类型
export * from './importExport';

// 导出系统监控类型
export * from './monitor';

// 导出 API 治理类型
export * from './apiGovernance';

// 导出表单生成器类型
export * from './formBuilder';

// 导出多语言增强类型
export * from './i18n';

// 导出高级搜索类型
export * from './advanced-search';
