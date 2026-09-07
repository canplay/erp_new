<template>
  <div class="tree-selector">
    <q-btn
      outline
      no-caps
      class="tree-selector__trigger"
      :class="{ 'tree-selector__trigger--error': error }"
      @click="openDialog"
    >
      <q-icon name="account_tree" class="q-mr-sm" />
      <span v-if="selectedLabel" class="selected-label">{{ selectedLabel }}</span>
      <span v-else class="placeholder-text">{{ placeholder || $t('common.select') }}</span>
      <q-icon name="arrow_drop_down" class="q-ml-sm" />
    </q-btn>

    <div v-if="hint && !error" class="tree-selector__hint">{{ hint }}</div>
    <div v-if="error" class="tree-selector__error">{{ error }}</div>

    <TreeDialog
      v-model="showDialog"
      :data="data"
      :multiple="multiple"
      :searchable="searchable"
      :show-toolbar="showToolbar"
      :show-refresh="showRefresh"
      :show-footer="showFooter"
      :clearable="clearable"
      :dialog-width="dialogWidth"
      :title="title"
      :empty-description="emptyDescription"
      v-model:selected="internalValue"
      @refresh="handleRefresh"
      @confirm="handleConfirm"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import TreeDialog from './TreeDialog.vue'
import { useTreeSelector } from './useTreeSelector'

const { t: $t } = useI18n()

interface TreeNode {
  id: string | number
  label: string
  icon?: string
  iconColor?: string
  extra?: string
  children?: TreeNode[]
  disabled?: boolean
  [key: string]: unknown
}

interface Props {
  value?: string | number | (string | number)[]
  placeholder?: string
  title?: string
  hint?: string
  error?: string
  multiple?: boolean
  searchable?: boolean
  showToolbar?: boolean
  showRefresh?: boolean
  showFooter?: boolean
  clearable?: boolean
  data?: TreeNode[]
  dialogWidth?: string
  emptyDescription?: string
  defaultExpandAll?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '请选择',
  title: '选择',
  hint: '',
  error: '',
  multiple: false,
  searchable: true,
  showToolbar: true,
  showRefresh: true,
  showFooter: true,
  clearable: true,
  data: () => [],
  dialogWidth: '500px',
  emptyDescription: '',
  defaultExpandAll: false
})

const emit = defineEmits<{
  (e: 'update:value', value: string | number | (string | number)[]): void
  (e: 'update:modelValue', value: string | number | (string | number)[]): void
  (e: 'change', value: string | number | (string | number)[]): void
  (e: 'refresh'): void
}>()

const {
  showDialog,
  internalValue,
  selectedLabel,
  openDialog,
  handleRefresh,
  handleConfirm
} = useTreeSelector({
  data: props.data,
  multiple: props.multiple,
  defaultExpandAll: props.defaultExpandAll,
  onChange: emit as any
})
</script>

<style scoped>
.tree-selector {
  display: inline-block;
  width: 100%;
}

.tree-selector__trigger {
  width: 100%;
  justify-content: flex-start;
  border-color: #dcdfe6;
  padding: 8px 12px;
}

.tree-selector__trigger:hover {
  border-color: #c0c4cc;
}

.tree-selector__trigger--error {
  border-color: #f56c6c;
}

.tree-selector__trigger .placeholder-text {
  color: #c0c4cc;
}

.tree-selector__trigger .selected-label {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-selector__hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.tree-selector__error {
  font-size: 12px;
  color: #f56c6c;
  margin-top: 4px;
}

.body--dark .tree-selector__trigger {
  border-color: #3d3d3d;
  background: #1e1e1e;
}

.body--dark .tree-selector__trigger:hover {
  border-color: #606266;
}

.body--dark .tree-selector__trigger .placeholder-text {
  color: #606266;
}

.body--dark .tree-selector__trigger .selected-label {
  color: #b0b0b0;
}

.body--dark .tree-selector__hint {
  color: #757575;
}
</style>
