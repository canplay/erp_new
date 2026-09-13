/**
 * @file TreeSelector.vue
 * @description 树形选择器组件 - 组合 TreeSearch + TreeNode
 * @date 2026-05-06
 */

<template>
  <div class="tree-selector">
    <!-- 选择触发器 -->
    <q-btn
      outline
      no-caps
      class="tree-selector__trigger"
      :class="{ 'tree-selector__trigger--error': error }"
      @click="showDialog = true"
    >
      <q-icon name="account_tree" class="q-mr-sm" />
      <span v-if="selectedLabel" class="selected-label">{{ selectedLabel }}</span>
      <span v-else class="placeholder-text">{{ placeholder || $t('common.select') }}</span>
      <q-icon name="arrow_drop_down" class="q-ml-sm" />
    </q-btn>

    <!-- 帮助文本 -->
    <div v-if="hint && !error" class="tree-selector__hint">{{ hint }}</div>
    <div v-if="error" class="tree-selector__error">{{ error }}</div>

    <!-- 树形选择对话框 -->
    <q-dialog v-model="showDialog" persistent>
      <q-card class="tree-selector__dialog" :style="{ width: dialogWidth || '500px' }">
        <!-- 对话框标题 -->
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6">{{ title || $t('common.select') }}</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <!-- 搜索和工具栏 -->
        <TreeSearch
          :searchable="searchable"
          :show-toolbar="showToolbar"
          :show-refresh="showRefresh"
          :expand-all="expandAll"
          :selected-count="selectedCount"
          :search-placeholder="$t('common.search')"
          :expand-all-label="$t('tree.expandAll')"
          :collapse-all-label="$t('tree.collapseAll')"
          :refresh-label="$t('common.refresh')"
          :selected-label="$t('tree.selected')"
          @toggle-expand="toggleExpandAll"
          @refresh="handleRefresh"
        />

        <!-- 树形节点 -->
        <q-card-section class="tree-selector__tree-container">
          <TreeNode
            :nodes="filteredTreeData"
            :selected="internalValue"
            :expanded-keys="expandedKeys"
            :ticked="internalTicked"
            :multiple="multiple"
            :filter="searchQuery"
            :no-results-label="$t('tree.noResults')"
            :no-data-title="$t('tree.noData')"
            :empty-description="emptyDescription"
          />
        </q-card-section>

        <!-- 底部操作 -->
        <q-card-actions v-if="showFooter" align="right" class="q-pa-md">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn
            v-if="clearable && selectedCount > 0"
            flat
            color="negative"
            :label="$t('common.clearSelection')"
            @click="handleClear"
          />
          <q-btn
            color="primary"
            :label="$t('common.confirm')"
            @click="handleConfirm"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import TreeSearch from './TreeSearch.vue';
import TreeNode from './TreeNode.vue';

const { t: $t } = useI18n();

interface TreeNode {
  id: string | number;
  label: string;
  icon?: string;
  iconColor?: string;
  extra?: string;
  children?: TreeNode[];
  disabled?: boolean;
  [key: string]: unknown;
}

interface Props {
  value?: string | number | (string | number)[];
  placeholder?: string;
  title?: string;
  hint?: string;
  error?: string;
  multiple?: boolean;
  searchable?: boolean;
  showToolbar?: boolean;
  showRefresh?: boolean;
  showFooter?: boolean;
  clearable?: boolean;
  data?: TreeNode[];
  dialogWidth?: string;
  emptyDescription?: string;
  defaultExpandAll?: boolean;
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
});

const emit = defineEmits<{
  (e: 'update:value', value: string | number | (string | number)[]): void;
  (e: 'update:modelValue', value: string | number | (string | number)[]): void;
  (e: 'change', value: string | number | (string | number)[]): void;
  (e: 'refresh'): void;
}>();

const showDialog = ref(false);
const searchQuery = ref('');
const expandedKeys = ref<(string | number)[]>([]);
const expandAll = ref(false);

const internalValue = computed({
  get: () => {
    if (props.multiple) {
      return Array.isArray(props.value) ? props.value : [];
    }
    return props.value;
  },
  set: (val) => {
    emitValue(val);
  }
});

const internalTicked = computed({
  get: () => {
    if (props.multiple && Array.isArray(props.value)) {
      return props.value;
    }
    return [];
  },
  set: (val) => {
    emitValue(val);
  }
});

const selectedLabel = computed(() => {
  if (!props.value) return '';

  if (props.multiple && Array.isArray(props.value)) {
    if (props.value.length === 0) return '';
    if (props.value.length === 1) {
      const node = findNode(props.data, props.value[0]!);
      return node?.label || '';
    }
    return `已选择 ${props.value.length} 项`;
  }

  const node = findNode(props.data, props.value as string | number);
  return node?.label || '';
});

const selectedCount = computed(() => {
  if (props.multiple && Array.isArray(props.value)) {
    return props.value.length;
  }
  return props.value ? 1 : 0;
});

const filteredTreeData = computed(() => {
  if (!searchQuery.value) {
    return props.data;
  }

  const query = searchQuery.value.toLowerCase();

  function filterNode(node: TreeNode): TreeNode | null {
    const labelMatch = node.label.toLowerCase().includes(query);
    const children = node.children
      ?.map(child => filterNode(child))
      ?.filter(child => child !== null);

    if (labelMatch || (children && children.length > 0)) {
      return { ...node, ...(children !== undefined ? { children } : {}) };
    }

    return null;
  }

  return props.data.map(node => filterNode(node)).filter(node => node !== null);
});

function findNode(nodes: TreeNode[], id: string | number): TreeNode | null {
  for (const node of nodes) {
    if (node.id === id) return node;
    if (node.children) {
      const found = findNode(node.children, id);
      if (found) return found;
    }
  }
  return null;
}

function emitValue(val: string | number | (string | number)[] | undefined): void {
  const finalVal = (val === undefined || val === null) ? (props.multiple ? [] : '') : val;
  emit('update:modelValue', finalVal);
  emit('update:value', finalVal);
  emit('change', finalVal);
}

function toggleExpandAll(): void {
  expandAll.value = !expandAll.value;

  if (expandAll.value) {
    expandedKeys.value = getAllNodeIds(props.data);
  } else {
    expandedKeys.value = [];
  }
}

function getAllNodeIds(nodes: TreeNode[]): (string | number)[] {
  const ids: (string | number)[] = [];

  function traverse(nodeList: TreeNode[]): void {
    for (const node of nodeList) {
      ids.push(node.id);
      if (node.children) {
        traverse(node.children);
      }
    }
  }

  traverse(nodes);
  return ids;
}

function handleClear(): void {
  emitValue(props.multiple ? [] : '');
  searchQuery.value = '';
}

function handleConfirm(): void {
  showDialog.value = false;
}

function handleRefresh(): void {
  emit('refresh');
}

watch(() => props.defaultExpandAll, (val) => {
  if (val) {
    expandedKeys.value = getAllNodeIds(props.data);
    expandAll.value = true;
  }
}, { immediate: true });

watch(() => props.data, (val) => {
  if (props.defaultExpandAll && val.length > 0) {
    expandedKeys.value = getAllNodeIds(val);
    expandAll.value = true;
  }
});
</script>

<style scoped>
.tree-selector {
  display: inline-block;
  width: 100%;
}

/* 触发器 */
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

/* 提示文本 */
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

/* 对话框 */
.tree-selector__dialog {
  max-height: 70vh;
  display: flex;
  flex-direction: column;
}

.tree-selector__tree-container {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

/* 暗色主题 */
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
</style>
