/**
 * @file useFormBuilder.ts
 * @description 表单生成器业务逻辑 composable
 * @date 2026-07-08
 */

import { ref, reactive, computed } from 'vue';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import type { FormConfig, FormFieldConfig } from '@/types/formBuilder';

// ---------- 常量 ----------

const componentCategories = [
  {
    name: '基础组件',
    components: [
      { type: 'input', label: '文本输入', icon: 'text_fields', color: 'primary' },
      { type: 'textarea', label: '多行文本', icon: 'notes', color: 'primary' },
      { type: 'number', label: '数字输入', icon: 'pin', color: 'primary' },
    ],
  },
  {
    name: '选择组件',
    components: [
      { type: 'select', label: '下拉选择', icon: 'arrow_drop_down_circle', color: 'info' },
      { type: 'radio', label: '单选组', icon: 'radio_button_checked', color: 'info' },
      { type: 'switch', label: '开关', icon: 'toggle_on', color: 'info' },
    ],
  },
  {
    name: '日期时间',
    components: [
      { type: 'date', label: '日期', icon: 'calendar_today', color: 'warning' },
      { type: 'datetime', label: '日期时间', icon: 'event', color: 'warning' },
    ],
  },
];

const fieldTypeOptions = [
  { label: '文本输入', value: 'input' },
  { label: '多行文本', value: 'textarea' },
  { label: '数字输入', value: 'number' },
  { label: '下拉选择', value: 'select' },
  { label: '单选组', value: 'radio' },
  { label: '开关', value: 'switch' },
  { label: '日期', value: 'date' },
  { label: '日期时间', value: 'datetime' },
];

const FIELD_COLORS: Record<string, string> = {
  input: 'primary',
  textarea: 'primary',
  number: 'primary',
  select: 'info',
  radio: 'info',
  switch: 'info',
  date: 'warning',
  datetime: 'warning',
};

// ---------- 工具函数 ----------

function createField(type: string, label: string): FormFieldConfig {
  return {
    id: `field_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    type: type as FormFieldConfig['type'],
    name: `field_${Math.random().toString(36).substr(2, 6)}`,
    label,
    placeholder: `请输入${label}`,
    span: 24,
  };
}

function getFieldColor(type: string): string {
  return FIELD_COLORS[type] || 'grey';
}

// ---------- composable ----------

export function useFormBuilder() {
  const $q = useQuasar();

  // --- 响应式状态 ---
  const componentSearch = ref('');
  const showPreviewDialog = ref(false);
  const selectedFieldId = ref<string | null>(null);

  const formConfig = reactive<FormConfig>({
    id: `form_${Date.now()}`,
    name: '我的表单',
    layout: 'vertical',
    groups: [
      {
        id: `group_${Date.now()}`,
        name: '基本信息',
        fields: [],
      },
    ],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  });

  // --- computed ---
  const selectedField = computed(() => {
    if (!selectedFieldId.value) return null;
    for (const group of formConfig.groups) {
      const field = group.fields.find((f) => f.id === selectedFieldId.value);
      if (field) return field;
    }
    return null;
  });

  // --- 方法 ---
  function handleDragStart(event: DragEvent, comp: { type: string; label: string }) {
    if (event.dataTransfer) {
      event.dataTransfer.setData('componentType', comp.type);
      event.dataTransfer.setData('componentLabel', comp.label);
    }
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    const componentType = event.dataTransfer?.getData('componentType');
    const componentLabel = event.dataTransfer?.getData('componentLabel');
    if (componentType && componentLabel && formConfig.groups.length > 0) {
      const newField = createField(componentType, componentLabel);
      formConfig.groups[0]!.fields.push(newField);
    }
  }

  function handleAddGroup() {
    formConfig.groups.push({
      id: `group_${Date.now()}`,
      name: `分组 ${formConfig.groups.length + 1}`,
      fields: [],
    });
  }

  function handleEditGroup(index: number) {
    const group = formConfig.groups[index];
    if (!group) return;
    void $q.dialog({
      title: '编辑分组',
      message: '请输入分组名称',
      prompt: { model: group.name, type: 'text' },
      cancel: true,
    }).onOk((name: string) => {
      group.name = name;
    });
  }

  function handleDeleteGroup(index: number) {
    void $q.dialog({
      title: '确认删除',
      message: '确定要删除这个分组吗？',
      cancel: true,
    }).onOk(() => {
      formConfig.groups.splice(index, 1);
    });
  }

  function handleAddField(groupIndex: number) {
    void $q.dialog({
      title: '添加字段',
      message: '选择字段类型',
      options: { type: 'radio', model: 'input', items: fieldTypeOptions.slice(0, 4) },
      cancel: true,
    }).onOk((type: string) => {
      const label = fieldTypeOptions.find((o) => o.value === type)?.label || '字段';
      const group = formConfig.groups[groupIndex];
      if (group) {
        group.fields.push(createField(type, label));
      }
    });
  }

  function handleSelectField(field: FormFieldConfig) {
    selectedFieldId.value = field.id;
  }

  function handleCopyField(groupIndex: number, fieldIndex: number) {
    const group = formConfig.groups[groupIndex];
    if (!group) return;
    const original = group.fields[fieldIndex];
    if (!original) return;
    const copy: FormFieldConfig = {
      ...original,
      type: original.type,
      id: `field_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      name: `${original.name}_copy`,
      label: `${original.label} (副本)`,
    };
    group.fields.splice(fieldIndex + 1, 0, copy);
  }

  function handleDeleteField(groupIndex: number, fieldIndex: number) {
    void $q.dialog({
      title: '确认删除',
      message: '确定要删除这个字段吗？',
      cancel: true,
    }).onOk(() => {
      const group = formConfig.groups[groupIndex];
      if (group) {
        group.fields.splice(fieldIndex, 1);
      }
    });
  }

  function handleSave() {
    formConfig.updated_at = new Date().toISOString();
    $q.notify({ type: 'positive', message: '表单配置已保存' });
  }

  function handlePreview() {
    showPreviewDialog.value = true;
  }

  function handleFormSubmit(values: Record<string, unknown>) {
    logger.info('【表单提交】', values);
    $q.notify({ type: 'positive', message: '表单提交成功' });
  }

  function handleFormReset() {
    logger.info('【表单重置】');
  }

  function handleUndo() {
    $q.notify({ type: 'info', message: '撤销' });
  }

  function handleRedo() {
    $q.notify({ type: 'info', message: '重做' });
  }

  function handleCopyConfig() {
    void navigator.clipboard.writeText(JSON.stringify(formConfig, null, 2));
    $q.notify({ type: 'positive', message: '配置已复制到剪贴板' });
  }

  function handleImportConfig() {
    void $q.dialog({
      title: '导入配置',
      message: '请粘贴 JSON 配置',
      prompt: { model: '', type: 'textarea' },
      cancel: true,
    }).onOk((configStr: string) => {
      try {
        Object.assign(formConfig, JSON.parse(configStr));
        $q.notify({ type: 'positive', message: '配置导入成功' });
      } catch {
        $q.notify({ type: 'negative', message: '配置格式错误' });
      }
    });
  }

  function handleExportConfig() {
    const blob = new Blob([JSON.stringify(formConfig, null, 2)], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `form_config_${Date.now()}.json`;
    a.click();
    $q.notify({ type: 'positive', message: '配置已导出' });
  }

  // --- 初始化 ---
  function initData() {
    // FormBuilder 无需异步加载数据，保留接口供未来扩展
  }

  return {
    // 状态
    componentSearch,
    showPreviewDialog,
    selectedFieldId,
    formConfig,

    // 常量
    componentCategories,
    fieldTypeOptions,

    // computed
    selectedField,

    // 工具函数
    getFieldColor,

    // 方法
    initData,
    handleDragStart,
    handleDrop,
    handleAddGroup,
    handleEditGroup,
    handleDeleteGroup,
    handleAddField,
    handleSelectField,
    handleCopyField,
    handleDeleteField,
    handleSave,
    handlePreview,
    handleFormSubmit,
    handleFormReset,
    handleUndo,
    handleRedo,
    handleCopyConfig,
    handleImportConfig,
    handleExportConfig,
  };
}
