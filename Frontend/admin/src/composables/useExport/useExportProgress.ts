/**
 * @file useExport/useExportProgress.ts
 * @description Export progress state and utilities
 * @date 2026-08-23
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';

export interface ExportProgress {
  total: number;
  current: number;
  status: 'pending' | 'running' | 'completed' | 'failed';
  message?: string;
}

export interface ExportOptionsV2 {
  format: 'csv' | 'xlsx' | 'json';
  filename?: string;
  headers?: Array<{ key: string; label: string }>;
}

export function useExportProgress() {
  const $q = useQuasar();
  const progress = ref<ExportProgress>({
    total: 0,
    current: 0,
    status: 'pending',
  });

  function start(total: number) {
    progress.value = { total, current: 0, status: 'running' };
  }

  function update(current: number, message?: string) {
    progress.value.current = current;
    if (message) {
      progress.value.message = message;
    }
  }

  function complete() {
    progress.value.status = 'completed';
  }

  function fail(message: string) {
    progress.value.status = 'failed';
    progress.value.message = message;
    $q.notify({ type: 'negative', message: 'Export failed' });
  }

  return { progress, start, update, complete, fail };
}
