/**
 * @file VirtualScrollTable/TableFooter.vue
 * @description 虚拟滚动表格 - 底部分页栏（选中状态、每页行数、分页器）
 * @date 2026-08-22
 */

<template>
  <div v-if="showBottomBar" class="table-footer">
    <div class="footer-info">
      <span v-if="selection && selectedCount > 0">
        {{ $t('batchActions.selected', { count: selectedCount }) }}
      </span>
      <span v-else>
        {{ $t('table.total', { total: totalRows }) }} {{ $t('table.records', { count: totalRows }) }}
      </span>
    </div>
    <div class="footer-pagination" v-if="showPagination">
      <span class="text-grey-6">{{ $t('table.rowsPerPage') }}:</span>
      <q-select
        v-model="localPageSize"
        :options="page_size_options"
        dense
        borderless
        emit-value
        map-options
        class="q-mx-sm"
        style="min-width: 80px"
      />
      <q-pagination
        v-model="localPage"
        :max="totalPages"
        :max-pages="5"
        boundary-numbers
        direction-links
        dense
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';

const { t: $t } = useI18n();

const props = withDefaults(defineProps<{
  totalRows: number;
  selectedCount: number;
  currentPage: number;
  pageSize: number;
  totalPages: number;
  selection: boolean;
  showBottomBar: boolean;
  showPagination: boolean;
  page_size_options?: number[];
}>(), {
  totalRows: 0,
  selectedCount: 0,
  currentPage: 1,
  pageSize: 20,
  totalPages: 0,
  selection: false,
  showBottomBar: true,
  showPagination: true,
  page_size_options: () => [10, 20, 50, 100],
});

const emit = defineEmits<{
  'page-change': [page: number, pageSize: number];
}>();

const localPage = ref(props.currentPage);
const localPageSize = ref(props.pageSize);

import { watch } from 'vue';
watch([localPage, localPageSize], () => {
  emit('page-change', localPage.value, localPageSize.value);
});
</script>

<style scoped>
.table-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid #e0e0e0;
  background: #fafafa;
  flex-shrink: 0;
}

.footer-info {
  font-size: 14px;
  color: #666;
}

.footer-pagination {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
