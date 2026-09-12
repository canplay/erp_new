/**
 * @file useImport.ts
 * @description 数据导入 Composable - 使用 ExcelJS (动态加载)
 * @date 2026-04-04
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import { useExport } from '@/composables/useExport';

// Quasar Column 类型定义
type Column = Record<string, unknown>;

export interface ImportMapping {
  /** Excel 列名 */
  excelColumn: string;
  /** 映射到目标字段 */
  targetField: string;
  /** 是否必填 */
  required?: boolean;
  /** 数据类型 */
  dataType?: 'string' | 'number' | 'date' | 'boolean';
  /** 校验规则 */
  validator?: (value: unknown) => boolean | string;
}

export interface ImportOptions {
  /** 文件大小限制（MB） */
  maxFileSize?: number;
  /** 允许的文件类型 */
  accept?: string[];
  /** 字段映射配置 */
  mappings?: ImportMapping[];
}

export interface ImportRow {
  /** 行号 */
  rowIndex: number;
  /** 原始数据 */
  raw: Record<string, unknown>;
  /** 映射后数据 */
  mapped: Record<string, unknown>;
  /** 校验错误 */
  errors: { field: string; message: string }[];
  /** 是否选中 */
  selected?: boolean;
}

export interface ImportResult {
  /** 成功导入的行数 */
  successCount: number;
  /** 失败的行数 */
  failCount: number;
  /** 错误详情 */
  errors: { row: number; field: string; message: string }[];
  /** 导入的数据 */
  data: Record<string, unknown>[];
}

/**
 * @brief 数据导入 Composable
 */
export function useImport<T extends Record<string, unknown> = Record<string, unknown>>() {
  const $q = useQuasar();

  // 状态
  const isImporting = ref(false);
  const importProgress = ref(0);
  const importStep = ref<'upload' | 'mapping' | 'preview' | 'importing' | 'complete'>('upload');

  // 数据
  const excelData = ref<unknown[][]>([]);
  const excelHeaders = ref<string[]>([]);
  const fieldMappings = ref<ImportMapping[]>([]);
  const importRows = ref<ImportRow[]>([]);
  const selectedRows = ref<Set<number>>(new Set());

  // 选项
  const defaultOptions: ImportOptions = {
    maxFileSize: 10, // 10MB
    accept: ['.xlsx', '.xls', '.csv'],
    mappings: [],
  };

  const currentOptions = { ...defaultOptions };

  /**
   * @brief 解析 Excel 文件
   */
  async function parseFile(file: File): Promise<void> {
    isImporting.value = true;
    importProgress.value = 0;
    importStep.value = 'upload';

    try {
      // 动态导入 ExcelJS
      const ExcelJS = await import('exceljs');
      const workbook = new ExcelJS.Workbook();
      // 将 File 转换为 ArrayBuffer 以兼容 ExcelJS
      const arrayBuffer = await file.arrayBuffer();
      await workbook.xlsx.load(arrayBuffer);

      // 获取第一个工作表
      const worksheet = workbook.getWorksheet(1);
      if (!worksheet) {
        throw new Error('文件中没有找到工作表');
      }

      // 提取表头和数据
      const headers: string[] = [];
      const data: unknown[][] = [];

      worksheet.eachRow((row, rowNumber) => {
        if (rowNumber === 1) {
          // 第一行是表头
          row.eachCell((cell, colNumber) => {
            const headerValue = cell.value;
            if (headerValue == null) {
              headers.push(`列${colNumber}`);
            } else if (typeof headerValue === 'object') {
              // 对象类型使用 JSON 序列化
              headers.push(JSON.stringify(headerValue));
            } else {
              headers.push(String(headerValue));
            }
          });
        } else {
          // 数据行
          const rowData: unknown[] = [];
          row.eachCell((cell) => {
            rowData.push(getCellValue(cell));
          });
          data.push(rowData);
        }

        importProgress.value = Math.floor((rowNumber / worksheet.rowCount) * 50);
      });

      excelHeaders.value = headers;
      excelData.value = data;

      // 自动生成字段映射
      autoGenerateMappings(headers);

      importStep.value = 'mapping';
      importProgress.value = 100;
    } catch (error) {
      logger.error('【解析 Excel 文件失败】', error);
      $q.notify({
        type: 'negative',
        message: '解析文件失败',
        caption: (error as Error).message,
      });
      throw error;
    } finally {
      isImporting.value = false;
    }
  }

  /**
   * @brief 获取单元格值
   */
  function getCellValue(cell: { value: unknown }): unknown {
    const value = cell.value;

    if (value === null || value === undefined) {
      return '';
    }

    // 日期类型
    if (value instanceof Date) {
      return value.toISOString();
    }

    // ExcelJS 公式
    if (typeof value === 'object' && 'formula' in value) {
      // 处理公式结果，确保返回字符串
      const result = (value as { formula: string; result?: unknown }).result;
      if (result == null) return '';
      if (typeof result === 'string') return result;
      if (typeof result === 'number' || typeof result === 'boolean') return String(result);
      return JSON.stringify(result);
    }

    return value;
  }

  /**
   * @brief 自动生成字段映射
   */
  function autoGenerateMappings(headers: string[]) {
    fieldMappings.value = headers.map((header) => ({
      excelColumn: header,
      targetField: header, // 默认一对一映射
      required: false,
      dataType: 'string' as const,
    }));
  }

  /**
   * @brief 更新字段映射
   */
  function updateMapping(index: number, mapping: Partial<ImportMapping>) {
    if (fieldMappings.value[index]) {
      fieldMappings.value[index] = { ...fieldMappings.value[index], ...mapping };
    }
  }

  /**
   * @brief 应用映射并预览数据
   */
  function applyMapping(): void {
    importStep.value = 'preview';
    importRows.value = [];

    excelData.value.forEach((row) => {
      const mapped: Record<string, unknown> = {};
      const errors: { field: string; message: string }[] = [];

      fieldMappings.value.forEach((mapping) => {
        const columnIndex = excelHeaders.value.indexOf(mapping.excelColumn);
        if (columnIndex === -1) return;

        const value = row[columnIndex];

        // 类型转换
        let processedValue = value;
        if (mapping.dataType) {
          processedValue = convertDataType(value, mapping.dataType);
        }

        // 必填校验
        if (mapping.required && !processedValue) {
          errors.push({
            field: mapping.targetField,
            // 显式转换为字符串，避免警告
            message: `${String(mapping.excelColumn)} 为必填字段`,
          });
        }

        // 自定义校验
        if (mapping.validator && processedValue) {
          const result = mapping.validator(processedValue);
          if (result !== true && typeof result === 'string') {
            errors.push({
              field: mapping.targetField,
              message: result,
            });
          }
        }

        mapped[mapping.targetField] = processedValue;
      });

      importRows.value.push({
        rowIndex: importRows.value.length + 2, // Excel 行号从 2 开始（1 是表头）
        raw: row.reduce<Record<string, unknown>>((acc, val, i) => {
          const header = excelHeaders.value[i];
          if (header !== undefined) {
            acc[header] = val;
          }
          return acc;
        }, {}),
        mapped,
        errors,
        selected: errors.length === 0, // 默认选中有效行
      });
    });

    // 更新选中行
    updateSelectedRows();
  }

  /**
   * @brief 数据类型转换
   */
  function convertDataType(value: unknown, dataType: string): unknown {
    if (value === null || value === undefined || value === '') {
      return null;
    }

    switch (dataType) {
      case 'number':
        return Number(value);
      case 'date':
        return new Date(value as string | number).toISOString();
      case 'boolean':
        if (typeof value === 'boolean') return value;
        if (typeof value === 'string') {
          return value.toLowerCase() === 'true' || value === '1' || value === '是';
        }
        return Boolean(value);
      default:
        // 其他类型转换为字符串
        if (value == null) return '';
        if (typeof value === 'object') return JSON.stringify(value);
        if (typeof value === 'number' || typeof value === 'boolean') return String(value);
        if (typeof value === 'bigint' || typeof value === 'symbol' || typeof value === 'function')
          return String(value);
        // 已经是字符串
        return value;
    }
  }

  /**
   * @brief 更新选中行
   */
  function updateSelectedRows() {
    selectedRows.value = new Set(
      importRows.value
        .filter((row) => row.errors.length === 0 && row.selected)
        .map((row) => row.rowIndex),
    );
  }

  /**
   * @brief 切换行选中状态
   */
  function toggleRowSelection(rowIndex: number) {
    const row = importRows.value.find((r) => r.rowIndex === rowIndex);
    if (row && row.errors.length === 0) {
      row.selected = !row.selected;
      updateSelectedRows();
    }
  }

  /**
   * @brief 全选/取消全选
   */
  function toggleSelectAll(select: boolean) {
    importRows.value.forEach((row) => {
      if (row.errors.length === 0) {
        row.selected = select;
      }
    });
    updateSelectedRows();
  }

  /**
   * @brief 执行导入
   */
  async function executeImport(
    apiFunction: (data: T[]) => Promise<{
      data?: { total: number; failed: number; errors: { row: number; message: string }[] };
    }>,
  ): Promise<ImportResult> {
    isImporting.value = true;
    importStep.value = 'importing';
    importProgress.value = 0;

    const selectedData = importRows.value
      .filter((row) => selectedRows.value.has(row.rowIndex))
      .map((row) => row.mapped) as T[];

    try {
      const response = await apiFunction(selectedData);
      // response.data 已经是 API 返回的结果
      const result = response.data || { total: 0, failed: 0, errors: [] };

      importProgress.value = 100;
      importStep.value = 'complete';

      $q.notify({
        type: 'positive',
        message: `成功导入 ${result.total - result.failed} 条数据`,
        caption: result.failed > 0 ? `失败 ${result.failed} 条` : '',
      });

      return {
        successCount: result.total - result.failed,
        failCount: result.failed,
        errors: result.errors.map((e: { row: number; message: string }) => ({
          row: e.row,
          field: '',
          message: e.message,
        })),
        data: selectedData,
      };
    } catch (error) {
      logger.error('【导入失败】', error);
      $q.notify({
        type: 'negative',
        message: '导入失败',
        caption: (error as Error).message,
      });
      throw error;
    } finally {
      isImporting.value = false;
    }
  }

  /**
   * @brief 下载错误报告
   */
  function downloadErrorReport() {
    const errorRows = importRows.value.filter((row) => row.errors.length > 0);

    if (errorRows.length === 0) {
      $q.notify({
        type: 'info',
        message: '没有错误数据',
      });
      return;
    }

    // 生成错误报告
    const errorReport = errorRows.map((row) => ({
      行号: row.rowIndex,
      ...row.raw,
      // 显式转换为字符串，避免 @typescript-eslint/no-base-to-string 警告
      错误信息: row.errors.map((e) => `${String(e.field)}: ${e.message}`).join('; '),
    }));

    // 使用 useExport 导出
    useExport().exportToCSV(errorReport, [], 'import_errors');
  }

  /**
   * @brief 重置导入状态
   */
  function reset() {
    excelData.value = [];
    excelHeaders.value = [];
    fieldMappings.value = [];
    importRows.value = [];
    selectedRows.value = new Set();
    importProgress.value = 0;
    importStep.value = 'upload';
    isImporting.value = false;
  }

  /**
   * @brief 获取预览表格列
   */
  function getPreviewColumns(): Column[] {
    return [
      {
        name: 'rowIndex',
        label: '行号',
        field: 'rowIndex',
        align: 'center' as const,
        style: 'width: 60px',
      },
      {
        name: 'status',
        label: '状态',
        field: 'status',
        align: 'center' as const,
        style: 'width: 80px',
      },
      ...fieldMappings.value.map((mapping) => ({
        name: mapping.targetField,
        label: mapping.targetField,
        field: mapping.targetField,
        align: 'left' as const,
      })),
    ];
  }

  return {
    // 状态
    isImporting,
    importProgress,
    importStep,

    // 数据
    excelHeaders,
    excelData,
    fieldMappings,
    importRows,
    selectedRows,

    // 方法
    parseFile,
    updateMapping,
    applyMapping,
    toggleRowSelection,
    toggleSelectAll,
    executeImport,
    downloadErrorReport,
    reset,
    getPreviewColumns,

    // 选项
    options: currentOptions,
  };
}
