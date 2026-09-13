/**
 * @file VirtualScrollTable/TableBody.vue
 * @description 虚拟滚动表格 - 表体组件（虚拟滚动行、空状态）
 * @date 2026-08-22
 */

<template>
  <div v-if="displayData.length > 0" ref="virtualScrollRef" class="table-body">
    <q-virtual-scroll
      :items="displayData"
      :virtual-scroll-item-size="rowHeight"
      class="virtual-scroll"
      :style="{ height: virtualScrollHeight }"
    >
      <template #default="{ item, index }">
        <div
          class="table-row"
          :class="{ 'row-selected': selectedKeys.has(getRowKey(item)) }"
          :style="{ height: rowHeight + 'px' }"
          @click="handleRowClick(item, index)"
        >
          <!-- 选择框列 -->
          <div v-if="selection" class="table-cell selection-cell">
            <q-checkbox
              :model-value="selectedKeys.has(getRowKey(item))"
              dense
              @update:model-value="(val: boolean) => toggleSelection(item, val)"
              @click.stop
            />
          </div>

          <!-- 数据列 -->
          <div
            v-for="column in columns"
            :key="column.name"
            class="table-cell"
            :class="column.align || 'left'"
            :style="{ width: column.width || 'auto', flex: column.width ? 'none' : 1 }"
          >
            <slot :name="`body-cell-${column.name}`" :row="item" :column="column" :index="index">
              {{ item[column.field as keyof typeof item] }}
            </slot>
          </div>
        </div>
      </template>
    </q-virtual-scroll>
  </div>

  <!-- 空状态 -->
  <div v-else class="empty-state">
    <q-icon name="inbox" size="48px" color="grey-5" />
    <div class="text-grey-6 q-mt-md">{{ emptyText }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface TableColumn {
  name: string;
  label: string;
  field: string | ((row: unknown) => unknown);
  align?: 'left' | 'center' | 'right';
  width?: string | number;
}

interface Props {
  columns: TableColumn[];
  displayData: Record<string, unknown>[];
  rowHeight: number;
  virtualScrollHeight: string;
  selection: boolean;
  selectionType: 'single' | 'multiple';
  selected: Record<string, unknown>[];
  emptyText: string;
  rowKey: string;
}

const props = withDefaults(defineProps<Props>(), {
  columns: () => [],
  displayData: () => [],
  rowHeight: 48,
  virtualScrollHeight: '400px',
  selection: false,
  selectionType: 'multiple',
  selected: () => [],
  emptyText: '暂无数据',
  rowKey: 'id',
});

const emit = defineEmits<{
  'update:selected': [value: Record<string, unknown>[]];
  'row-click': [row: Record<string, unknown>, index: number];
}>();

const virtualScrollRef = ref<{ scrollTo: (index: number) => void } | null>(null);
const selectedKeys = ref<Set<unknown>>(new Set());

import { watch } from 'vue';
watch(
  () => props.selected,
  (newSelected) => {
    selectedKeys.value = new Set(newSelected.map((row) => row[props.rowKey]));
  },
  { immediate: true, deep: true }
);

function getRowKey(row: Record<string, unknown>): unknown {
  return row[props.rowKey];
}

function toggleSelection(row: Record<string, unknown>, selected: boolean) {
  const key = getRowKey(row);
  if (selected) {
    selectedKeys.value.add(key);
  } else {
    selectedKeys.value.delete(key);
  }
  updateSelected();
}

function updateSelected() {
  const selectedRows = props.displayData.filter((row) => selectedKeys.value.has(getRowKey(row)));
  emit('update:selected', selectedRows);
}

function handleRowClick(row: Record<string, unknown>, index: number) {
  if (props.selection) {
    const key = getRowKey(row);
    const isSelected = selectedKeys.value.has(key);
    if (props.selectionType === 'single') {
      selectedKeys.value.clear();
      if (!isSelected) {
        selectedKeys.value.add(key);
      }
    } else {
      if (isSelected) {
        selectedKeys.value.delete(key);
      } else {
        selectedKeys.value.add(key);
      }
    }
    updateSelected();
  }
  emit('row-click', row, index);
}

function scrollToRow(index: number) {
  virtualScrollRef.value?.scrollTo(index);
}

function clearSelection() {
  selectedKeys.value.clear();
  updateSelected();
}

function toggleSelectAll() {
  if (selectedKeys.value.size === props.displayData.length) {
    selectedKeys.value.clear();
  } else {
    props.displayData.forEach((row) => {
      selectedKeys.value.add(getRowKey(row));
    });
  }
  updateSelected();
}

defineExpose({
  scrollToRow,
  clearSelection,
  toggleSelectAll,
});
</script>

<style scoped>
.table-body {
  flex: 1;
  overflow: hidden;
}

.table-row {
  display: flex;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.15s ease;
}

.table-row:hover {
  background: rgba(0, 0, 0, 0.02);
}

.table-row.row-selected {
  background: rgba(25, 118, 210, 0.08);
}

.table-cell {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  min-height: 48px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.table-cell.left {
  justify-content: flex-start;
}

.table-cell.center {
  justify-content: center;
}

.table-cell.right {
  justify-content: flex-end;
}

.selection-cell {
  padding-left: 16px;
  padding-right: 8px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 0;
}
</style>
