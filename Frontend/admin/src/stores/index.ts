import { createPinia } from 'pinia';

// 创建 Pinia 实例
export const pinia = createPinia();

// 导出所有模块 Store（完整入口）
export { useAuthStore } from './auth';
export { useNotificationStore } from './notification';
export { usePermissionStore } from './permission';
export { useThemeStore } from './theme';
export { useOperationLogStore } from './operation-log';
export { useDictionaryStore } from './dictionary';
export { useEbikeStore } from './ebike';
export { useCtpStore } from './ctp';
export { useDeviceStore } from './device';
export { useExportTaskStore } from './exportTask';
export { useFeedbackStore } from './feedback';
export { useFileStore } from './file';
export { useLprStore } from './lpr';
export { useReportStore } from './report';
export { useScheduledTaskStore } from './scheduled-task';
export { useTenantStore } from './tenant';
export { useTowStore } from './tow';
export { useXltStore } from './xlt';

// 兼容 Quasar 自动导入机制的默认导出
export default pinia;
