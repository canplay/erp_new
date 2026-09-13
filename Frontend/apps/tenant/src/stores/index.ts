import { createPinia } from 'pinia';

// 创建 Pinia 实例
export const pinia = createPinia();

// 导出所有模块 Store（完整入口）
// 已将18个store文件合并为6个文件，每个文件导出多个store
export { useAuthStore, usePermissionStore, useNotificationStore, useThemeStore } from './auth';
export { useDictionaryStore, useTenantStore, useOperationLogStore } from './system';
export { useFileStore, useExportTaskStore } from './file';
export { useDeviceStore, useCtpStore, useLprStore, useTowStore, useXltStore } from './device';
export { useReportStore, useScheduledTaskStore } from './report';
export { useEbikeStore, useFeedbackStore } from './ebike';

// 兼容 Quasar 自动导入机制的默认导出
export default pinia;
