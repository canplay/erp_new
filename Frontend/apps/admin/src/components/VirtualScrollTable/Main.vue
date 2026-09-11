// @ts-nocheck
<template>
  <div class="virtual-scroll-table">
    <div class="virtual-scroll-table__header">
      <div class="text-h6">{{ title }}</div>
      <div class="row q-gutter-sm">
        <slot name="header-actions" />
      </div>
    </div>

    <div class="virtual-scroll-table__toolbar">
      <slot name="toolbar" />
    </div>

    <div class="virtual-scroll-table__content">
      <VirtualHead :columns="columns" @sort-change="handleSortChange" />

      <div class="virtual-scroll-table__body">
        <VirtualRow
          :rows="visibleRows as any"
          :row-height="rowHeight"
          :total-height="containerHeight"
          :render-row="renderRow"
        />
      </div>

      <div v-if="isLoading" class="virtual-scroll-table__loading">
        <q-spinner-colorful size="40px" color="primary" />
      </div>
    </div>

    <div class="virtual-scroll-table__footer">
      <q-pagination
        v-model="currentPage"
        :max="totalPages"
        dense
        outline
        color="primary"
        @change="handlePageChange"
      />
      <div class="text-caption text-grey-6 q-ml-md">
        {{ $t('table.showing') }} {{ totalCount }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import VirtualHead from './VirtualHead.vue'
import VirtualRow from './VirtualRow.vue'
import { useVirtualScrollTable } from './useVirtualScrollTable'

const { t: $t } = useI18n()

interface Column {
  name: string
  label: string
  field: string
  sortable?: boolean
  align?: 'left' | 'center' | 'right'
}

interface Props {
  title?: string
  columns: Column[]
  rows: unknown[]
  rowHeight?: number
  containerHeight?: string | number
  pageSize?: number
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '数据表格',
  rowHeight: 48,
  containerHeight: '400px',
  pageSize: 20,
  loading: false
})

const emit = defineEmits<{
  (e: 'sort-change', payload: { field: string; order: string }): void
  (e: 'page-change', page: number): void
  (e: 'row-click', row: unknown): void
}>()

const currentPage = ref(1)

const {
  totalPages,
  totalCount,
  visibleRows,
  isLoading,
  handleSortChange,
  handlePageChange,
  renderRow
} = useVirtualScrollTable({
  rows: props.rows,
  pageSize: props.pageSize,
  loading: props.loading,
  onSortChange: emit,
  onPageChange: emit
})

watch(currentPage, (page) => {
  handlePageChange(page)
})
</script>

<style scoped>
.virtual-scroll-table {
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  overflow: hidden;
}

.virtual-scroll-table__header {
  padding: 16px;
  background: #fafafa;
  border-bottom: 1px solid rgba(0, 0, 0, 0.12);
}

.virtual-scroll-table__toolbar {
  padding: 8px 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.12);
}

.virtual-scroll-table__content {
  height: v-bind('props.containerHeight');
  position: relative;
}

.virtual-scroll-table__loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.virtual-scroll-table__footer {
  padding: 16px;
  background: #fafafa;
  border-top: 1px solid rgba(0, 0, 0, 0.12);
  display: flex;
  align-items: center;
}

.body--dark .virtual-scroll-table__header,
.body--dark .virtual-scroll-table__footer {
  background: #1e1e1e;
}
</style>
