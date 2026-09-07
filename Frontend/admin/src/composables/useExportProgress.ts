/**
 * @file useExportProgress.ts
 * @description Export progress tracking and status management
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';

export interface ExportProgress {
  percent: number;
  stage: 'preparing' | 'processing' | 'downloading' | 'complete';
  message: string;
}

export interface UseExportProgressReturn {
  isExporting: ReturnType<typeof ref<boolean>>;
  exportProgress: ReturnType<typeof ref<ExportProgress>>;
  updateProgress: (percent: number, stage: ExportProgress['stage'], message: string) => void;
  resetProgress: () => void;
  notifySuccess: (count: number) => void;
  notifyError: (caption?: string) => void;
  cleanup: () => void;
}

export function useExportProgress(): UseExportProgressReturn {
  const $q = useQuasar();

  const isExporting = ref(false);
  const exportProgress = ref<ExportProgress>({
    percent: 0,
    stage: 'preparing',
    message: '',
  });

  function updateProgress(percent: number, stage: ExportProgress['stage'], message: string) {
    exportProgress.value = { percent, stage, message };
  }

  function resetProgress() {
    exportProgress.value = { percent: 0, stage: 'preparing', message: '' };
  }

  function notifySuccess(count: number) {
    $q.notify({
      type: 'positive',
      message: `成功导出 ${count} 条数据`,
      position: 'top',
    });
  }

  function notifyError(caption?: string) {
    const message = caption ? `导出失败，请重试: ${caption}` : '导出失败，请重试';
    $q.notify({
      type: 'negative',
      message,
      position: 'top',
    });
  }

  function cleanup() {
    setTimeout(() => {
      isExporting.value = false;
    }, 1000);
  }

  return {
    isExporting,
    exportProgress,
    updateProgress,
    resetProgress,
    notifySuccess,
    notifyError,
    cleanup,
  };
}
