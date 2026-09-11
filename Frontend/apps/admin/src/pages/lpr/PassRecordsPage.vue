<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">{{ $t('parking.passRecordsTitle') }}</div>

    <!-- 统计卡片 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col">
        <q-card-section class="row items-center">
          <q-icon name="directions_car" size="36px" color="primary" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.todayEntry') }}</div>
            <div class="text-h6">{{ store.todayEntryCount }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col">
        <q-card-section class="row items-center">
          <q-icon name="exit_to_app" size="36px" color="warning" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.todayExit') }}</div>
            <div class="text-h6">{{ store.todayExitCount }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col">
        <q-card-section class="row items-center">
          <q-icon name="check_circle" size="36px" color="positive" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.highConfidence') }}</div>
            <div class="text-h6">{{ store.highConfidence }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 搜索栏 -->
    <div class="row q-mb-md items-center q-gutter-sm">
      <q-input v-model="filters.plate_no" :placeholder="$t('common.plateNo')" dense outlined style="max-width: 180px" debounce="300" />
      <q-select v-model="filters.direction" :options="dirOptions" :placeholder="$t('parking.direction')" dense outlined style="max-width: 120px" clearable />
      <q-select v-model="filters.status" :options="statusOptions" :placeholder="$t('parking.statusLabel')" dense outlined style="max-width: 140px" clearable />
      <q-input v-model="filters.start_time" :label="$t('common.startTime')" type="date" dense outlined style="max-width: 160px" />
      <q-input v-model="filters.end_time" :label="$t('common.endTime')" type="date" dense outlined style="max-width: 160px" />
      <q-btn icon="search" :label="$t('common.search')" color="primary" @click="doSearch" />
      <q-btn flat icon="refresh" :label="$t('common.refresh')" @click="loadRecords" />
    </div>

    <!-- 记录表格 -->
    <q-table :rows="store.records" :columns="responsiveColumns" row-key="id" :loading="store.loading"
      flat bordered :pagination="{ rowsPerPage: 20, page: store.page, rowsNumber: store.total }"
      @request="onRequest" :wrap-cells="false">

      <template #loading>
        <SkeletonLoader type="table" :columns="columns.length" :rows="5" />
      </template>

      <template #no-data>
        <EmptyState icon="directions_car" :title="$t('empty.noRecords')" :description="$t('parking.noPassRecords')" />
      </template>

      <template #body-cell-confidence="{ row }">
        <q-chip :color="row.confidence >= 0.9 ? 'positive' : row.confidence >= 0.7 ? 'warning' : 'negative'"
          text-color="white" dense>{{ (row.confidence * 100).toFixed(0) }}%</q-chip>
      </template>
      <template #body-cell-direction="{ row }">
        <q-chip :color="row.direction === 'entry' ? 'positive' : 'warning'" text-color="white" dense>
          {{ row.direction === 'entry' ? $t('parking.entry') : $t('parking.exit') }}
        </q-chip>
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useLprStore } from '@/stores/lpr';
import EmptyState from '@/components/EmptyState.vue';
import SkeletonLoader from '@/components/SkeletonLoader.vue';

const { t } = useI18n();
const $q = useQuasar();
const store = useLprStore();
const filters = reactive({ plate_no: '', direction: '', status: '', start_time: '', end_time: '' });
const isMobile = computed(() => $q.screen.lt.md);
const dirOptions = [{ label: t('parking.entry'), value: 'entry' }, { label: t('parking.exit'), value: 'exit' }];
const statusOptions = [{ label: t('common.pending'), value: 'pending' }, { label: t('common.processed'), value: 'processed' }, { label: t('common.failed'), value: 'failed' }];

const columns = [
  { name: 'plate_no', label: t('parking.plateNo'), field: 'plate_no', align: 'left' as const, sortable: true },
  { name: 'plate_color', label: t('parking.plateColor'), field: 'plate_color', align: 'center' as const },
  { name: 'direction', label: t('parking.direction'), field: 'direction', align: 'center' as const },
  { name: 'device_name', label: t('parking.device'), field: 'device_name', align: 'center' as const },
  { name: 'park_code', label: t('parking.parkCode'), field: 'park_code', align: 'center' as const },
  { name: 'pass_time', label: t('parking.passTime'), field: 'pass_time', align: 'center' as const },
  { name: 'confidence', label: t('parking.confidence'), field: 'confidence', align: 'center' as const },
  { name: 'status', label: t('parking.statusLabel'), field: 'status', align: 'center' as const },
];

// 移动端显示精简列
const responsiveColumns = computed(() => {
  if (isMobile.value) {
    return columns.filter(c => !['plate_color', 'park_code', 'confidence', 'status'].includes(c.name));
  }
  return columns;
});

async function loadRecords() {
  await store.fetchRecords({
    page: store.page, page_size: store.page_size,
    ...(filters.plate_no && { plate_no: filters.plate_no }),
    ...(filters.direction && { direction: filters.direction as 'entry' | 'exit' }),
    ...(filters.status && { status: filters.status as 'pending' | 'processed' | 'failed' }),
    ...(filters.start_time && { start_time: filters.start_time }),
    ...(filters.end_time && { end_time: filters.end_time }),
  });
}

function doSearch() { store.page = 1; void loadRecords(); }
function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  store.page = props.pagination.page;
  store.page_size = props.pagination.rowsPerPage;
  void loadRecords();
}

onMounted(async () => { try { await loadRecords(); } catch { $q.notify({ type: 'negative', message: t('parking.loadPassRecordsFailed') }); } });
</script>
