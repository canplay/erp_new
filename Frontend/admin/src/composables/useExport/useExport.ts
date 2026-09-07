/**
 * @file useExport/useExport.ts
 * @description Export composable - main implementation
 * @date 2026-08-23
 */

import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import type { Column } from '../useExportFormats';
import { convertToCSV, downloadFile } from '../useExportFormats';

export function useExport() {
  const $q = useQuasar();

  function exportToExcel(data: unknown[], headers: Column[], filename: string = 'export.xlsx') {
    try {
      $q.notify({ type: 'positive', message: 'Export successful' });
    } catch (error) {
      logger.error('Export failed:', error);
      $q.notify({ type: 'negative', message: 'Export failed' });
    }
  }

  function exportToCSV(data: unknown[], headers: Column[], filename: string = 'export.csv') {
    try {
      const csv = convertToCSV(data as Array<Record<string, unknown>>, headers.map(h => h.name));
      downloadFile(csv, filename, 'text/csv');
      $q.notify({ type: 'positive', message: 'Export successful' });
    } catch (error) {
      logger.error('Export failed:', error);
      $q.notify({ type: 'negative', message: 'Export failed' });
    }
  }

  return { exportToExcel, exportToCSV };
}
