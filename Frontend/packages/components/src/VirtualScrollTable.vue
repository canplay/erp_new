/**
 * @file VirtualScrollTable.vue
 * @description 虚拟滚动表格组件 - 组合 Header/Body/Footer 子组件
 * @date 2026-04-03
 */

<template>
  <div class="virtual-scroll-table" :class="{ 'table-dark': isDark }">
    <TableHeader
      :columns="columns"
      :table-width="tableWidth"
      :sort-field="sortConfig?.field"
      :sort-order="sortConfig?.order"
      @sort="handleSort"
    />

    <TableBody
      ref="tableBodyRef"
      :columns="columns"
      :display-data="displayData"
      :row-height="rowHeight"
      :virtual-scroll-height="virtualScrollHeight"
      :selection="selection"
      :selection-type="selectionType"
      :selected="selected"
      :empty-text="emptyText"
      :row-key="rowKey"
      @update:selected="onSelectedChange"
      @row-click="(row: Record<string, unknown>, index: number) => emit('row-click', row, index)"
    />

    <TableFooter
      :total-rows="totalRows"
      :selected-count="selectedKeys.size"
      :current-page="currentPage"
      :page-size="page_size"
      :total-pages="totalPages"
      :selection="selection"
      :show-bottom-bar="showBottomBar"
      :show-pagination="showPagination"
      @page-change="onPageChange"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import TableHeader from './TableHeader.vue';
import TableBody from './TableBody.vue';
import TableFooter from './TableFooter.vue';

const $q = useQuasar();

// ============ 类型定义 ============

/**
 * @brief 列定义接口
 */
export interface TableColumn {
  /** 列名 */
  name: string;
  /** 显示标题 */
  label: string;
  /** 字段名 */
  field: string | ((row: unknown) => unknown);
  /** 对齐方式 */
  align?: 'left' | 'center' | 'right';
  /** 宽度 */
  width?: string | number;
  /** 是否可排序 */
  sortable?: boolean;
  /** 固定列 */
  fixed?: 'left' | 'right';
}

interface SortConfig {
  field: string;
  order: 'asc' | 'desc';
}

// ============ Props & Emits ============

interface Props {
  columns: TableColumn[];
  data?: Record<string, unknown>[];
  rowHeight?: number;
  virtualScrollHeight?: string;
  selection?: boolean;
  selectionType?: 'single' | 'multiple';
  selected?: Record<string, unknown>[];
  emptyText?: string;
  showBottomBar?: boolean;
  showPagination?: boolean;
  rowKey?: string;
  defaultPageSize?: number;
}

const props = withDefaults(defineProps<Props>(), {
  columns: () => [],
  data: () => [],
  rowHeight: 48,
  virtualScrollHeight: '400px',
  selection: false,
  selectionType: 'multiple',
  selected: () => [],
  emptyText: '暂无数据',
  showBottomBar: true,
  showPagination: true,
  rowKey: 'id',
  defaultPageSize: 20,
});

const emit = defineEmits<{
  (e: 'update:selected', value: Record<string, unknown>[]): void;
  (e: 'row-click', row: Record<string, unknown>, index: number): void;
  (e: 'sort', field: string, order: 'asc' | 'desc'): void;
  (e: 'page-change', page: number, page_size: number): void;
}>();

// ============ 响应式状态 ============
const tableBodyRef = ref<InstanceType<typeof TableBody>>();
const currentPage = ref(1);
const page_size = ref(props.defaultPageSize);
const sortConfig = ref<SortConfig | null>(null);
const selectedKeys = ref<Set<unknown>>(new Set());

// ============ 计算属性 ============
const isDark = computed(() => $q.dark.isActive);

const tableWidth = computed(() => {
  const fixedWidth = props.columns
    .filter((col) => col.width)
    .reduce((sum, col) => sum + (typeof col.width === 'number' ? col.width : parseInt(String(col.width)) || 100), 0);
  const flexCount = props.columns.filter((col) => !col.width).length;
  return `calc(${fixedWidth}px + ${flexCount * 100}%)`;
});

const totalRows = computed(() => props.data.length);
const totalPages = computed(() => Math.ceil(totalRows.value / page_size.value));

const sortedData = computed(() => {
  const sort = sortConfig.value;
  if (!sort) return props.data;

  return [...props.data].sort((a, b) => {
    const field = sort.field;
    const aVal = a[field];
    const bVal = b[field];

    if (aVal === bVal) return 0;
    if (aVal == null) return 1;
    if (bVal == null) return -1;

    const comparison = aVal < bVal ? -1 : 1;
    return sort.order === 'asc' ? comparison : -comparison;
  });
});

const paginatedData = computed(() => {
  if (!props.showPagination) return sortedData.value;
  const start = (currentPage.value - 1) * page_size.value;
  const end = start + page_size.value;
  return sortedData.value.slice(start, end);
});

const displayData = computed(() => {
  return props.showPagination ? paginatedData.value : sortedData.value;
});

// ============ 监听 ============
watch(
  () => props.selected,
  (newSelected) => {
    selectedKeys.value = new Set(newSelected.map((row) => row[props.rowKey]));
  },
  { immediate: true, deep: true }
);

// ============ 方法 ============
function onSelectedChange(rows: Record<string, unknown>[]) {
  emit('update:selected', rows);
}

function onPageChange(page: number, pageSize: number) {
  currentPage.value = page;
  page_size.value = pageSize;
  emit('page-change', page, pageSize);
}

function handleSort(field: string, order: 'asc' | 'desc') {
  if (sortConfig.value && sortConfig.value.field === field) {
    sortConfig.value.order = sortConfig.value.order === 'asc' ? 'desc' : 'asc';
  } else {
    sortConfig.value = { field, order: 'asc' };
  }
  emit('sort', field, order);
}

// ============ 暴露方法给父组件 ============
defineExpose({
  scrollToRow: (index: number) => {
    const body = tableBodyRef.value as { scrollToRow?: (index: number) => void } | null;
    body?.scrollToRow?.(index);
  },
  clearSelection: () => {
    const body = tableBodyRef.value as { clearSelection?: () => void } | null;
    body?.clearSelection?.();
  },
  toggleSelectAll: () => {
    const body = tableBodyRef.value as { toggleSelectAll?: () => void } | null;
    body?.toggleSelectAll?.();
  },
});
</script>

<style scoped>
.virtual-scroll-table {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
  background: #ffffff;
  display: flex;
  flex-direction: column;
  min-height: 200px;
}
</style>
