<template>
  <q-table
    v-bind="$attrs"
    :rows="rows"
    :columns="displayedColumns"
    :pagination="paginationProps"
    @request="onRequest"
  >
    <!-- 排序指示器插槽 -->
    <template v-slot:header="slotProps">
      <q-tr :props="slotProps">
        <q-th
          v-for="col in slotProps.cols"
          :key="col.name"
          :props="slotProps"
          class="sortable-header"
          @click="toggleSort(col.name)"
        >
          {{ col.label }}
          <q-icon
            v-if="sortField === col.name"
            :name="sort_order === 'asc' ? 'arrow_upward' : 'arrow_downward'"
            size="xs"
            class="sort-icon"
          />
        </q-th>
      </q-tr>
    </template>

    <!-- 默认插槽透传 -->
    <template v-for="(_, name) in $slots" #[name]="slotData">
      <slot :name="name" v-bind="slotData || {}" />
    </template>
  </q-table>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface Column {
  name: string;
  label: string;
  field: string | ((row: unknown) => unknown);
  align?: 'left' | 'center' | 'right';
  sortable?: boolean;
  [key: string]: unknown;
}

interface PaginationProps {
  page: number;
  rowsPerPage: number;
  rowsNumber?: number;
  sortBy?: string | null;
  descending?: boolean;
}

const props = defineProps<{
  rows: unknown[];
  columns: Column[];
  pagination?: PaginationProps;
  defaultSortField?: string;
  defaultSortOrder?: 'asc' | 'desc';
}>();

const emit = defineEmits<{
  (e: 'request', props: { pagination: PaginationProps }): void;
  (e: 'sort', field: string, order: 'asc' | 'desc'): void;
}>();

// 排序状态
const sortField = ref(props.defaultSortField || '');
const sort_order = ref<'asc' | 'desc'>(props.defaultSortOrder || 'asc');

// 当前显示的列
const displayedColumns = computed(() => {
  return props.columns.filter((col) => col.sortable !== false);
});

// 当前的分页配置
const paginationProps = computed((): PaginationProps => {
  return {
    page: props.pagination?.page || 1,
    rowsPerPage: props.pagination?.rowsPerPage || 10,
    rowsNumber: props.pagination?.rowsNumber || 0,
    sortBy: sortField.value || null,
    descending: sort_order.value === 'desc',
  };
});

/**
 * @brief 切换排序
 */
function toggleSort(field: string) {
  if (sortField.value === field) {
    // 同一字段，切换排序方向
    sort_order.value = sort_order.value === 'asc' ? 'desc' : 'asc';
  } else {
    // 新字段，默认降序
    sortField.value = field;
    sort_order.value = 'desc';
  }

  // 触发排序事件
  emit('sort', sortField.value, sort_order.value);

  // 触发请求事件以更新数据
  emit('request', {
    pagination: {
      page: 1,
      rowsPerPage: props.pagination?.rowsPerPage || 10,
      rowsNumber: props.pagination?.rowsNumber || 0,
      sortBy: sortField.value,
      descending: sort_order.value === 'desc',
    },
  });
}

/**
 * @brief 处理请求事件
 */
function onRequest(requestProps: { pagination: PaginationProps }) {
  if (requestProps.pagination.sortBy) {
    sortField.value = requestProps.pagination.sortBy;
    sort_order.value = requestProps.pagination.descending ? 'desc' : 'asc';
  }
  emit('request', requestProps);
}
</script>

<style scoped>
.sortable-header {
  cursor: pointer;
  user-select: none;
}

.sortable-header:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.sort-icon {
  margin-left: 4px;
}
</style>
