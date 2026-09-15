<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">{{ $t('parking.livePassMonitor') }}</div>

    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col-8">
        <q-card-section>
          <div class="text-h6">{{ $t('parking.todayPassTrend') }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section style="height: 360px">
          <ECharts v-if="chartData.length" :config="liveChartConfig || {}" />
          <div v-else class="text-grey text-center" style="padding-top: 140px">{{ $t('parking.noPassData') }}</div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col">
        <q-card-section>
          <div class="text-h6">{{ $t('parking.liveSummary') }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section>
          <q-list dense>
            <q-item><q-item-section><q-item-label>{{ $t('parking.todayTotalPass') }}</q-item-label><q-item-label caption class="text-h5 text-primary">{{ store.total }}</q-item-label></q-item-section></q-item>
            <q-separator />
            <q-item><q-item-section><q-item-label>{{ $t('parking.entry') }}</q-item-label><q-item-label caption class="text-h5 text-positive">{{ store.todayEntryCount }}</q-item-label></q-item-section></q-item>
            <q-separator />
            <q-item><q-item-section><q-item-label>{{ $t('parking.exit') }}</q-item-label><q-item-label caption class="text-h5 text-warning">{{ store.todayExitCount }}</q-item-label></q-item-section></q-item>
            <q-separator />
            <q-item><q-item-section><q-item-label>{{ $t('parking.highConfidence') }}</q-item-label><q-item-label caption class="text-h5 text-info">{{ store.highConfidence }}</q-item-label></q-item-section></q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </div>

    <!-- 最近通行 -->
    <q-card flat bordered>
      <q-card-section><div class="text-h6">{{ $t('parking.recentPassRecords') }}</div></q-card-section>
      <q-separator />
      <q-table :rows="store.records.slice(0, 10)" :columns="columns" row-key="id" flat hide-pagination
        :loading="store.loading">
        <template #loading>
          <SkeletonLoader type="table" :columns="columns.length" :rows="5" />
        </template>
        <template #no-data>
          <EmptyState icon="videocam" :title="$t('empty.noRecords')" :description="$t('parking.noRecentPassRecords')" />
        </template>
        <template #body-cell-direction="{ row }">
          <q-chip :color="row.direction === 'entry' ? 'positive' : 'warning'" text-color="white" dense>
            {{ row.direction === 'entry' ? $t('parking.entry') : $t('parking.exit') }}
          </q-chip>
        </template>
      </q-table>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">import { useI18n } from 'vue-i18n';

import { computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useLprStore } from '@/stores/device';
import ECharts from '@erp-new-frontend-monorepo/components/src/ECharts/Main.vue';
import EmptyState from '@erp-new-frontend-monorepo/components/src/EmptyState.vue';
import SkeletonLoader from '@erp-new-frontend-monorepo/components/src/SkeletonLoader.vue';

const { t } = useI18n();
const $q = useQuasar();
const store = useLprStore();

// 生成模拟小时通行数据（0~23时）
const chartData = computed(() => {
  const hours = Array.from({ length: 24 }, (_, i) => `${String(i).padStart(2, '0')}:00`);
  const entryData = hours.map(() => Math.floor(Math.random() * 30 + 5));
  const exitData = hours.map(() => Math.floor(Math.random() * 25 + 3));
  return hours.map((label, i) => ({
    label,
    entry: entryData[i],
    exit: exitData[i],
  }));
});

const liveChartConfig = computed<any>(() => ({
  type: 'line',
  title: '',
  height: '320px',
  showLegend: true,
  showTooltip: true,
  areaFill: false,
  showGrid: true,
  data: chartData.value.flatMap(d => [
    { label: d.label, value: d.entry ?? 0, series: '入场' },
    { label: d.label, value: d.exit ?? 0, series: '出场' },
  ]),
}));

const columns = [
  { name: 'plate_no', label: t('parking.plateNo'), field: 'plate_no' },
  { name: 'direction', label: t('parking.direction'), field: 'direction' },
  { name: 'device_name', label: t('parking.device'), field: 'device_name' },
  { name: 'pass_time', label: t('parking.time'), field: 'pass_time' },
  { name: 'confidence', label: t('parking.confidence'), field: 'confidence' },
];

onMounted(async () => {
  try {
    await store.fetchRecords({ page_size: 10 });
  } catch {
    $q.notify({ type: 'negative', message: t('parking.loadDataFailed') });
  }
});
</script>
