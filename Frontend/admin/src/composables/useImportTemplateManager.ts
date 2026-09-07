/**
 * @file useImportTemplateManager.ts
 * @description 导入模板管理 composable
 * @date 2026-07-08
 */

import { ref, computed, reactive } from 'vue';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import type { QTableProps } from 'quasar';
import type { ImportTemplate, ImportTemplateColumn } from '@/types/importExport';
import { DEFAULT_IMPORT_TEMPLATES } from '@/types/importExport';
import ExcelJS from 'exceljs';

export function useImportTemplateManager() {
  const $q = useQuasar();

  // ============ 常量 ============

  const entity_type_options = [
    { label: '用户', value: 'user' },
    { label: '客户', value: 'customer' },
    { label: '订单', value: 'order' },
    { label: '商品', value: 'product' },
  ];

  const dataTypeOptions = [
    { label: '文本', value: 'string' },
    { label: '数字', value: 'number' },
    { label: '日期', value: 'date' },
    { label: '布尔值', value: 'boolean' },
    { label: '下拉选择', value: 'select' },
  ];

  // ============ 状态 ============

  const loading = ref(false);
  const searchQuery = ref('');
  const entity_type_filter = ref<string | null>(null);

  const templates = ref<ImportTemplate[]>([
    ...DEFAULT_IMPORT_TEMPLATES.map((tpl, i) => ({
      ...tpl,
      id: `template_${i + 1}`,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    })),
  ]);

  const showPreviewDialog = ref(false);
  const showEditDialog = ref(false);
  const isEditMode = ref(false);
  const previewTemplateData = ref<ImportTemplate | null>(null);
  const previewSampleData = ref<Record<string, unknown>[]>([]);

  // ============ 表格列定义 ============

  const columns: QTableProps['columns'] = [
    { name: 'name', label: '模板名称', field: 'name', align: 'left', sortable: true },
    { name: 'entity_type', label: '实体类型', field: 'entity_type', align: 'center' },
    { name: 'columns', label: '列数', field: 'columns', align: 'center' },
    { name: 'created_at', label: '创建时间', field: 'created_at', align: 'left' },
    { name: 'actions', label: '操作', field: 'actions', align: 'center' },
  ];

  const columnConfigColumns: QTableProps['columns'] = [
    { name: 'header', label: '表头', field: 'header', align: 'left' },
    { name: 'field', label: '字段', field: 'field', align: 'left' },
    { name: 'dataType', label: '类型', field: 'dataType', align: 'center' },
    { name: 'required', label: '必填', field: 'required', align: 'center' },
    { name: 'options', label: '选项', field: 'options', align: 'left' },
  ];

  const previewColumns = computed(() => {
    if (!previewTemplateData.value?.columns?.length) return [];
    return [
      { name: 'index', label: '#', field: 'index', align: 'center' as const },
      ...previewTemplateData.value.columns.map((col) => ({
        name: col.field,
        label: col.header,
        field: col.field,
        align: 'left' as const,
      })),
    ];
  });

  // ============ 表单数据 ============

  const formData = reactive<{
    id?: string;
    name: string;
    entity_type: string;
    description: string;
    columns: ImportTemplateColumn[];
  }>({
    name: '',
    entity_type: 'user',
    description: '',
    columns: [],
  });

  // ============ 计算属性 ============

  const filteredTemplates = computed(() => {
    let result = templates.value;

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(
        (tpl) =>
          tpl.name.toLowerCase().includes(query) ||
          tpl.description?.toLowerCase().includes(query),
      );
    }

    if (entity_type_filter.value) {
      result = result.filter((tpl) => tpl.entity_type === entity_type_filter.value);
    }

    return result;
  });

  // ============ 方法 ============

  /**
   * @brief 获取实体类型标签
   */
  function getEntityLabel(entity_type: string): string {
    const option = entity_type_options.find((o) => o.value === entity_type);
    return option?.label || entity_type;
  }

  /**
   * @brief 预览模板
   */
  function previewTemplate(template: ImportTemplate) {
    previewTemplateData.value = template;

    // 生成示例数据
    previewSampleData.value = Array.from({ length: 3 }, (_, i) => {
      const row: Record<string, unknown> = { index: i + 1 };
      template.columns?.forEach((col) => {
        if (col.defaultValue !== undefined) {
          row[col.field] = col.defaultValue;
        } else if (col.dataType === 'select' && col.options?.length) {
          row[col.field] = col.options[0]!.label;
        } else if (col.dataType === 'number') {
          row[col.field] = Math.floor(Math.random() * 1000);
        } else {
          row[col.field] = `示例${col.header}`;
        }
      });
      return row;
    });

    showPreviewDialog.value = true;
  }

  /**
   * @brief 下载模板
   */
  async function downloadTemplate(template: ImportTemplate | null) {
    if (!template) return;

    try {
      const workbook = new ExcelJS.Workbook();
      const worksheet = workbook.addWorksheet(template.name);

      // 添加表头
      const headerRow = worksheet.addRow(template.columns.map((c) => c.header));

      // 设置表头样式
      headerRow.eachCell((cell) => {
        cell.font = {
          color: { argb: 'FFFFFFFF' },
          bold: true,
        };
        cell.fill = {
          type: 'pattern',
          pattern: 'solid',
          fgColor: { argb: 'FF1976D2' },
        };
      });

      // 添加示例数据
      const sampleData = Array.from({ length: template.sampleRowCount || 5 }, () => {
        const row: unknown[] = [];
        template.columns.forEach((col) => {
          if (col.defaultValue !== undefined) {
            row.push(col.defaultValue);
          } else if (col.dataType === 'select' && col.options?.length) {
            row.push(col.options[0]!.value);
          } else if (col.dataType === 'number') {
            row.push(Math.floor(Math.random() * 1000));
          } else if (col.dataType === 'boolean') {
            row.push(true);
          } else {
            row.push('');
          }
        });
        return row;
      });

      worksheet.addRows(sampleData);

      // 设置列宽
      worksheet.columns.forEach((col, index) => {
        col.width = template.columns[index]?.width || 15;
      });

      // 生成文件
      const buffer = await workbook.xlsx.writeBuffer();
      const blob = new Blob([buffer], {
        type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      });

      // 下载
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${template.name}.xlsx`;
      link.click();
      URL.revokeObjectURL(url);

      $q.notify({
        type: 'positive',
        message: '模板下载成功',
      });
    } catch (error) {
      logger.error('【下载模板失败】', error);
      $q.notify({
        type: 'negative',
        message: '模板下载失败',
      });
    }
  }

  /**
   * @brief 打开创建对话框
   */
  function openCreateDialog() {
    isEditMode.value = false;
    Object.assign(formData, {
      id: undefined,
      name: '',
      entity_type: 'user',
      description: '',
      columns: [],
    });
    showEditDialog.value = true;
  }

  /**
   * @brief 打开编辑对话框
   */
  function openEditDialog(template: ImportTemplate) {
    isEditMode.value = true;
    Object.assign(formData, {
      id: template.id,
      name: template.name,
      entity_type: template.entity_type,
      description: template.description || '',
      columns: [...(template.columns || [])],
    });
    showEditDialog.value = true;
  }

  /**
   * @brief 添加列
   */
  function addColumn() {
    formData.columns.push({
      field: '',
      header: '',
      required: false,
      dataType: 'string',
    });
  }

  /**
   * @brief 移除列
   */
  function removeColumn(index: number) {
    formData.columns.splice(index, 1);
  }

  /**
   * @brief 保存模板
   */
  function handleSave() {
    if (!formData.name || !formData.entity_type) {
      $q.notify({
        type: 'warning',
        message: '请填写必填字段',
      });
      return;
    }

    if (formData.columns.length === 0) {
      $q.notify({
        type: 'warning',
        message: '请至少添加一列',
      });
      return;
    }

    const now = new Date().toISOString();

    if (isEditMode.value && formData.id) {
      // 更新
      const index = templates.value.findIndex((tpl) => tpl.id === formData.id);
      if (index > -1) {
        const existing = templates.value[index]!;
        templates.value[index] = {
          ...existing,
          id: existing.id,
          name: formData.name,
          entity_type: formData.entity_type,
          description: formData.description,
          columns: [...formData.columns] as ImportTemplate['columns'],
          sampleRowCount: existing.sampleRowCount,
          is_default: existing.is_default,
          created_by: existing.created_by,
          created_at: existing.created_at,
          updated_at: now,
        };
      }
    } else {
      // 创建
      templates.value.unshift({
        id: `template_${Date.now()}`,
        name: formData.name,
        entity_type: formData.entity_type,
        description: formData.description,
        columns: [...formData.columns],
        sampleRowCount: 5,
        is_default: false,
        created_by: 'admin',
        created_at: now,
        updated_at: now,
      });
    }

    showEditDialog.value = false;

    $q.notify({
      type: 'positive',
      message: isEditMode.value ? '模板已更新' : '模板已创建',
    });
  }

  /**
   * @brief 删除模板
   */
  function deleteTemplate(template: ImportTemplate) {
    $q.dialog({
      title: '确认删除',
      message: `确定要删除模板「${template.name}」吗？此操作不可恢复。`,
      ok: {
        label: '删除',
        color: 'negative',
      },
      cancel: {
        label: '取消',
        flat: true,
      },
    }).onOk(() => {
      templates.value = templates.value.filter((tpl) => tpl.id !== template.id);
      $q.notify({
        type: 'positive',
        message: '模板已删除',
      });
    });
  }

  /**
   * @brief 初始化数据
   */
  function initData() {
    loading.value = true;
    // 当前为本地数据，无需异步加载；预留接口扩展点
    loading.value = false;
  }

  return {
    // 常量
    entity_type_options,
    dataTypeOptions,
    // 状态
    loading,
    searchQuery,
    entity_type_filter,
    templates,
    showPreviewDialog,
    showEditDialog,
    isEditMode,
    previewTemplateData,
    previewSampleData,
    // 表格列
    columns,
    columnConfigColumns,
    previewColumns,
    // 表单
    formData,
    // 计算属性
    filteredTemplates,
    // 方法
    getEntityLabel,
    previewTemplate,
    downloadTemplate,
    openCreateDialog,
    openEditDialog,
    addColumn,
    removeColumn,
    handleSave,
    deleteTemplate,
    initData,
  };
}
