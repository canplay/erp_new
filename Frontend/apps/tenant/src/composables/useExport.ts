/**
 * @file useExport.ts
 * @description Unified export composable - CSV/Excel export, format utilities, progress tracking
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';

// ── Types ──

export interface ExportOptions {
  fieldMapping?: Record<string, string>;
  filename?: string;
  separator?: string;
  format?: 'csv' | 'json' | 'xlsx';
  sheetName?: string;
  dateFormat?: string;
  title?: string;
  description?: string;
}

export interface Column {
  name: string;
  label: string;
  field: string;
  align?: 'left' | 'right' | 'center';
  sortable?: boolean;
}

export interface ExportProgress {
  percent: number;
  stage: 'preparing' | 'processing' | 'downloading' | 'complete';
  message: string;
}

export interface UseExportProgressReturn {
  isExporting: ReturnType<typeof ref<boolean>>;
  exportProgress: ReturnType<typeof ref<ExportProgress>>;
  updateProgress: (percent: number, stage: string, message: string) => void;
  resetProgress: () => void;
  notifySuccess: (count: number) => void;
  notifyError: (caption?: string) => void;
  cleanup: () => void;
}

// ── Progress tracker (merged from useExportProgress.ts) ──

export function useExportProgress(): UseExportProgressReturn {
  const $q = useQuasar();

  const isExporting = ref(false);
  const exportProgress = ref<ExportProgress>({
    percent: 0,
    stage: 'preparing',
    message: '',
  });

  function updateProgress(percent: number, stage: string, message: string) {
    exportProgress.value = { percent, stage: stage as ExportProgress['stage'], message };
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

// ── Format utilities (merged from useExportFormats.ts) ──

function convertToCSV<T extends Record<string, unknown>>(
  data: T[],
  headers?: string[],
  fieldMapping?: Record<string, string>,
): string {
  if (!data || data.length === 0) return '';

  const keys = headers || (data[0] ? Object.keys(data[0]) : []);
  const mappedKeys = fieldMapping ? keys.map((key) => fieldMapping[key] || key) : keys;

  const headerRow = mappedKeys.join(',');
  const dataRows = data.map((row) => {
    return keys
      .map((key) => {
        const value = row[key];
        if (value == null) return '';
        let strValue = '';
        if (typeof value === 'object') {
          strValue = JSON.stringify(value);
        } else if (typeof value === 'string') {
          strValue = value;
        } else if (typeof value === 'number' || typeof value === 'boolean') {
          strValue = String(value);
        } else if (typeof value === 'bigint' || typeof value === 'symbol' || typeof value === 'function') {
          strValue = String(value);
        }
        if (strValue.includes(',') || strValue.includes('"') || strValue.includes('\n')) {
          return `"${strValue.replace(/"/g, '""')}"`;
        }
        return strValue;
      })
      .join(',');
  });

  return [headerRow, ...dataRows].join('\n');
}

function downloadFile(content: string | Blob, filename: string, mimeType: string) {
  const blob = content instanceof Blob ? content : new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  URL.revokeObjectURL(url);
}

function formatDate(date: Date, format?: string): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');

  if (format === 'date') {
    return `${year}-${month}-${day}`;
  }
  if (format === 'time') {
    return `${hours}:${minutes}:${seconds}`;
  }
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

function formatCellValue(value: unknown, dateFormat?: string): unknown {
  if (value === null || value === undefined) return '';
  if (value instanceof Date) return formatDate(value, dateFormat);
  if (typeof value === 'string') {
    const parsedDate = new Date(value);
    if (!isNaN(parsedDate.getTime()) && value.includes('-')) {
      return formatDate(parsedDate, dateFormat);
    }
  }
  return value;
}

async function _exportToExcel<T extends Record<string, unknown>>(
  data: T[],
  options: ExportOptions = {},
  updateProgress: (percent: number, stage: string, message: string) => void,
) {
  const {
    filename = 'export',
    sheetName = 'Sheet1',
    fieldMapping,
    dateFormat = 'YYYY-MM-DD HH:mm:ss',
    title,
    description,
  } = options;

  const $q = useQuasar();

  try {
    const ExcelJS = await import('exceljs');
    const workbook = new ExcelJS.Workbook();
    const worksheet = workbook.addWorksheet(sheetName, {
      views: [{ state: 'frozen', xSplit: 0, ySplit: 1 }],
    });

    const totalSteps = data.length + 5;
    let currentStep = 0;

    const updateProgressInternal = (step: number, message: string) => {
      currentStep = step;
      updateProgress(Math.floor((currentStep / totalSteps) * 100), 'processing', message);
    };

    if (title) {
      updateProgressInternal(++currentStep, '添加标题...');
      worksheet.mergeCells(
        'A1:' + String.fromCharCode(65 + (data[0] ? Object.keys(data[0]).length - 1 : 0)) + '1',
      );
      const titleCell = worksheet.getCell('A1');
      titleCell.value = title;
      titleCell.font = { size: 16, bold: true, color: { argb: 'FF1976D2' } };
      titleCell.alignment = { horizontal: 'center' };
    }

    if (description) {
      updateProgressInternal(++currentStep, '添加描述...');
      const descRow = title ? 2 : 1;
      worksheet.mergeCells(
        `A${descRow}:` +
          String.fromCharCode(65 + (data[0] ? Object.keys(data[0]).length - 1 : 0)) +
          descRow,
      );
      const descCell = worksheet.getCell(`A${descRow}`);
      descCell.value = description;
      descCell.font = { size: 10, italic: true, color: { argb: 'FF666666' } };
    }

    updateProgressInternal(++currentStep, '添加表头...');
    const headers = data[0] ? Object.keys(data[0]) : [];
    const mappedHeaders = fieldMapping ? headers.map((h) => fieldMapping[h] || h) : headers;

    const headerRowIndex = (title ? 2 : 0) + (description ? 1 : 0) + 1;
    const headerRow = worksheet.getRow(headerRowIndex);
    headerRow.values = mappedHeaders;
    headerRow.font = { bold: true, color: { argb: 'FFFFFFFF' } };
    headerRow.fill = {
      type: 'pattern',
      pattern: 'solid',
      fgColor: { argb: 'FF1976D2' },
    };
    headerRow.alignment = { horizontal: 'center' };
    headerRow.eachCell((cell) => {
      cell.border = {
        top: { style: 'thin' },
        left: { style: 'thin' },
        bottom: { style: 'thin' },
        right: { style: 'thin' },
      };
    });

    const dataStartRow = headerRowIndex + 1;
    data.forEach((row, index) => {
      updateProgressInternal(headerRowIndex + index + 1, `导出数据 ${index + 1}/${data.length}...`);

      const rowValues = headers.map((key) => formatCellValue(row[key], dateFormat));
      const dataRow = worksheet.getRow(dataStartRow + index);
      dataRow.values = rowValues as typeof dataRow.values;
      dataRow.eachCell((cell) => {
        cell.border = {
          top: { style: 'thin' },
          left: { style: 'thin' },
          bottom: { style: 'thin' },
          right: { style: 'thin' },
        };
        if (index % 2 === 1) {
          cell.fill = {
            type: 'pattern',
            pattern: 'solid',
            fgColor: { argb: 'FFF5F5F5' },
          };
        }
      });
    });

    worksheet.columns.forEach((column) => {
      column.width = 15;
    });

    updateProgressInternal(totalSteps - 1, '生成文件...');
    const buffer = await workbook.xlsx.writeBuffer();

    const timestamp = formatDate(new Date(), 'date');
    const fullFilename = `${filename}_${timestamp}.xlsx`;

    downloadFile(
      buffer as Blob,
      fullFilename,
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    );

    $q.notify({
      type: 'positive',
      message: `成功导出 ${data.length} 条数据`,
      position: 'top',
    });
  } catch (error) {
    logger.error('【导出 Excel 失败】', error);
    $q.notify({
      type: 'negative',
      message: '导出失败，请重试',
      caption: (error as Error).message,
      position: 'top',
    });
  }
}

// ── Main composable ──

export function useExport() {
  const $q = useQuasar();
  const { updateProgress } = useExportProgress();

  function exportToExcel(data: unknown[], headers: Column[], filename: string = 'export.xlsx') {
    try {
      const colNames = headers.map((h) => h.name);
      const fieldMapping: Record<string, string> = {};
      headers.forEach((h) => { fieldMapping[h.name] = h.label; });
      const progressFn = updateProgress;
      _exportToExcel(data as Array<Record<string, unknown>>, { filename, fieldMapping }, progressFn);
      $q.notify({ type: 'positive', message: 'Export successful' });
    } catch (error) {
      logger.error('Export failed:', error);
      $q.notify({ type: 'negative', message: 'Export failed' });
    }
  }

  function exportToCSV(data: unknown[], headers: Column[], filename: string = 'export.csv') {
    try {
      const csv = convertToCSV(data as Array<Record<string, unknown>>, headers.map((h) => h.name));
      downloadFile(csv, filename, 'text/csv');
      $q.notify({ type: 'positive', message: 'Export successful' });
    } catch (error) {
      logger.error('Export failed:', error);
      $q.notify({ type: 'negative', message: 'Export failed' });
    }
  }

  return { exportToExcel, exportToCSV };
}

// ── Re-exports for backward compatibility ──
export { convertToCSV, downloadFile, formatDate, formatCellValue };
export const useExportFormats = useExport;
