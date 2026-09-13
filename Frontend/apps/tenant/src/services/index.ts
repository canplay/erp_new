/**
 * @file index.ts
 * @brief services 统一导出
 * @date 2026-06-18
 */

export * from './config-service';
export * from './security-service';
export * from './permission-service';
export * from './dictionary-service';
export * from './device-service';

// 便捷导出
export { configService } from './config-service';
export { securityService } from './security-service';
export { permissionService } from './permission-service';
export { dictionaryService } from './dictionary-service';
export { deviceService } from './device-service';
