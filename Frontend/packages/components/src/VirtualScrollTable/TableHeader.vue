/**
 * @file VirtualScrollTable/TableHeader.vue
 * @description 虚拟滚动表格 - 表头组件（列标题、排序图标）
 * @date 2026-08-22
 */

<template>
  <div class="table-header" :style="{ width: tableWidth }">
    <div
      v-for="column in columns"
      :key="column.name"
      class="table-cell header-cell"
      :class="[column.align || 'left', { sortable: column.sortable }]"
      :style="{ width: column.width || 'auto', flex: column.width ? 'none' : 1 }"
      @click="column.sortable && handleSort(column)"
    >
      <span class="header-label">{{ column.label }}</span>
      <q-icon
        v-if="column.sortable"
        :name="getSortIcon(column.name)"
        size="14px"
        class="sort-icon"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface TableColumn {
  name: string;
  label: string;
  field: string | ((row: unknown) => unknown);
  align?: 'left' | 'center' | 'right';
  width?: string | number;
  sortable?: boolean;
  fixed?: 'left' | 'right';
}

const props = withDefaults(defineProps<{
  columns: TableColumn[];
  tableWidth: string;
  sortField?: string;
  sortOrder?: 'asc' | 'desc';
  sortFieldDefault?: string;
  sortOrderDefault?: 'asc' | 'desc';
}>(), {
  columns: () => [],
  sortField: undefined,
  sortOrder: undefined,
  sortFieldDefault: '',
  sortOrderDefault: undefined,
});

const emit = defineEmits<{
  sort: [field: string, order: 'asc' | 'desc'];
}>();

function getSortIcon(field: string): string {
  if (!props.sortField || props.sortField !== field) {
    return 'unfold_more';
  }
  return props.sortOrder === 'asc' ? 'expand_less' : 'expand_more';
}

function handleSort(column: TableColumn) {
  if (!column.sortable) return;
  const field = typeof column.field === 'function' ? column.name : column.field;
  let order: 'asc' | 'desc' = 'asc';
  if (props.sortField === field && props.sortOrder === 'asc') {
    order = 'desc';
  }
  emit('sort', field, order);
}
</script>

<style scoped>
.table-header {
  display: flex;
  background: #f5f5f5;
  border-bottom: 1px solid #e0e0e0;
  font-weight: 600;
  font-size: 14px;
  flex-shrink: 0;
}

.header-cell {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 12px 16px;
  user-select: none;
}

.header-cell.sortable {
  cursor: pointer;
}

.header-cell.sortable:hover {
  background: rgba(0, 0, 0, 0.03);
}

.sort-icon {
  opacity: 0.5;
}

.header-cell.sortable:hover .sort-icon {
  opacity: 1;
}
</style>
