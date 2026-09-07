/**
 * @file useExport.ts
 * @description Export composable - main entry
 * @date 2026-08-23
 */
/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck

import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import type { ExportOptions, Column } from '@/composables/useExportFormats';
import type { ExportOptions as ExportOptionsV2, ExportProgress } from '@/composables/useExport/useExportProgress';
import { useExportFormats } from '@/composables/useExportFormats';
import { useExportProgress } from '@/composables/useExport/useExportProgress';

export type { ExportOptions, ExportOptionsV2, Column, ExportProgress };
export { useExportFormats, useExportProgress };

export function useExport() {
  const $q = useQuasar();

  function exportToExcel(data: unknown[], headers: Column[], filename: string = 'export.xlsx') {
    try {
      const workbook = useExportFormats.createWorkbook(data, headers);
      useExportFormats.downloadWorkbook(workbook, filename);
      $q.notify({ type: 'positive', message: 'Export successful' });
    } catch (error) {
      logger.error('Export failed:', error);
      $q.notify({ type: 'negative', message: 'Export failed' });
    }
  }

  function exportToCSV(data: unknown[], headers: Column[], filename: string = 'export.csv') {
    try {
      const csv = useExportFormats.dataToCSV(data, headers);
      useExportFormats.downloadFile(csv, filename);
      $q.notify({ type: 'positive', message: 'Export successful' });
    } catch (error) {
      logger.error('Export failed:', error);
      $q.notify({ type: 'negative', message: 'Export failed' });
    }
  }

  return { exportToExcel, exportToCSV };
}
