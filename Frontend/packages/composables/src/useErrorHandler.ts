/**
 * @file useErrorHandler.ts
 * @description 全局错误处理 Composable
 * @date 2026-08-15
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';

export interface ErrorInfo {
  id: string;
  type: 'error' | 'warning' | 'info' | 'success';
  title: string;
  message: string;
  timestamp: number;
  dismissed: boolean;
}

export async function useErrorHandler() {
  const $q = useQuasar();
  const errorHistory = ref<ErrorInfo[]>([]);
  const MAX_HISTORY_SIZE = 50;

  function generateErrorId(): string {
    return `err_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  function handleError(
    error: unknown,
    options?: {
      title?: string;
      type?: 'error' | 'warning' | 'info';
      duration?: number;
      persistent?: boolean;
      showNotification?: boolean;
    }
  ) {
    const {
      title = '出错了',
      type = 'error',
      duration = 5000,
      persistent = false,
      showNotification = true,
    } = options || {};

    let message = '未知错误';
    if (typeof error === 'string') {
      message = error;
    } else if (error instanceof Error) {
      message = error.message;
    } else if (typeof error === 'object' && error !== null) {
      message = (error as { message?: string }).message || JSON.stringify(error);
    }

    const errorInfo: ErrorInfo = {
      id: generateErrorId(),
      type,
      title,
      message,
      timestamp: Date.now(),
      dismissed: false,
    };

    errorHistory.value.unshift(errorInfo);
    if (errorHistory.value.length > MAX_HISTORY_SIZE) {
      errorHistory.value = errorHistory.value.slice(0, MAX_HISTORY_SIZE);
    }

    logger.error(`[${title}]`, message, error);

    if (showNotification) {
      $q.notify({
        type,
        message: `${title}: ${message}`,
        timeout: persistent ? 0 : duration,
        actions: persistent
          ? [{ label: '关闭', color: 'white', handler: () => dismissError(errorInfo.id) }]
          : [{ label: '关闭', color: 'white' }],
      });
    }

    return errorInfo;
  }

  function dismissError(id: string) {
    const error = errorHistory.value.find((e) => e.id === id);
    if (error) {
      error.dismissed = true;
    }
  }

  function clearAllErrors() {
    errorHistory.value = [];
  }

  function getActiveErrors(): ErrorInfo[] {
    return errorHistory.value.filter((e) => !e.dismissed);
  }

  async function catchError<T>(
    promise: Promise<T>,
    errorHandler?: (error: unknown) => void
  ): Promise<T | undefined> {
    try {
      return await promise;
    } catch (error) {
      if (errorHandler) {
        errorHandler(error);
      } else {
        handleError(error);
      }
      return undefined;
    }
  }

  return {
    errorHistory,
    handleError,
    dismissError,
    clearAllErrors,
    getActiveErrors,
    catchError,
  };
}

export async function useGlobalErrorHandler() {
  const $q = useQuasar();

  function initGlobalErrorHandler() {
    window.addEventListener('unhandledrejection', (event) => {
      logger.error('[未处理的 Promise 错误]', event.reason);
      $q.notify({
        type: 'negative',
        message: `操作失败: ${event.reason?.message || '发生未知错误'}`,
        timeout: 5000,
      });
    });

    window.addEventListener('error', (event) => {
      logger.error('[全局错误]', event.error);
      $q.notify({
        type: 'negative',
        message: '系统错误: 应用程序遇到错误，请刷新页面重试',
        timeout: 5000,
      });
    });

    logger.info('【全局错误处理器】已初始化');
  }

  function cleanupGlobalErrorHandler() {
    window.removeEventListener('unhandledrejection', () => {});
    window.removeEventListener('error', () => {});
  }

  return {
    initGlobalErrorHandler,
    cleanupGlobalErrorHandler,
  };
}
