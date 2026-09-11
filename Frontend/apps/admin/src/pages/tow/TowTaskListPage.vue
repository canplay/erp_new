<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">拖车任务管理</div>

    <div class="row q-mb-md q-gutter-sm">
      <q-input v-model="search" :placeholder="$t('common.searchPlaceholder')" dense outlined style="max-width: 260px" class="q-mr-md" />
      <q-btn icon="search" :label="$t('common.search')" color="primary" @click="doSearch" />
      <q-btn flat icon="refresh" :label="$t('common.refresh')" @click="loadCars" />
    </div>

    <q-table :rows="store.cars" :columns="responsiveColumns" row-key="id" :loading="store.loading"
      flat bordered :pagination="{ rowsPerPage: 20, page: store.page, rowsNumber: store.total }"
      @request="onRequest" @row-click="(_, row) => $router.push(`/tow/car/${row.id}`)" :wrap-cells="false">
      <template #body-cell-delete="{ row }">
        <q-chip :color="row.delete ? 'negative' : 'positive'" text-color="white" dense>
          {{ row.delete ? '已删除' : '正常' }}
        </q-chip>
      </template>
      <template #loading>
        <SkeletonLoader type="table" :columns="columns.length" :rows="5" />
      </template>
      <template #no-data>
        <EmptyState icon="local_shipping" :title="$t('empty.noData')" description="暂无拖车任务" />
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useTowStore } from '@/stores/tow';
import SkeletonLoader from '@/components/SkeletonLoader.vue';
import EmptyState from '@/components/EmptyState.vue';
const $q = useQuasar();
const store = useTowStore();
const search = ref('');
const isMobile = computed(() => $q.screen.lt.md);

const columns = [
  { name: 'id', label: 'ID', field: 'id', align: 'left' as const },
  { name: 'license', label: '车牌号', field: 'license', align: 'left' as const, sortable: true },
  { name: 'car_type', label: '车型', field: 'car_type', align: 'center' as const },
  { name: 'car_color', label: '颜色', field: 'car_color', align: 'center' as const },
  { name: 'dc_date', label: '拖车日期', field: 'dc_date', align: 'center' as const },
  { name: 'dc_party_name', label: '当事人', field: 'dc_party_name', align: 'center' as const },
  { name: 'dc_address', label: '拖车地址', field: 'dc_address' },
  { name: 'delete', label: '状态', field: 'delete', align: 'center' as const },
];

// 移动端显示精简列
const responsiveColumns = computed(() => {
  if (isMobile.value) {
    return columns.filter(c => !['car_color', 'dc_address', 'delete'].includes(c.name));
  }
  return columns;
});

async function loadCars() { await store.fetchCars({ page: store.page, page_size: store.page_size }); }
function doSearch() { store.page = 1; void loadCars(); }
function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  store.page = props.pagination.page;
  store.page_size = props.pagination.rowsPerPage;
  void loadCars();
}

onMounted(async () => { try { await loadCars(); } catch { $q.notify({ type: 'negative', message: '加载车辆数据失败' }); } });
</script>
