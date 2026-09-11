<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">在场车辆查询</div>

    <div class="row q-mb-md q-gutter-sm">
      <q-input v-model="plateNo" :placeholder="$t('common.plateNo')" dense outlined style="max-width: 200px" />
      <q-input v-model="parkCode" :placeholder="$t('common.parkCode')" dense outlined style="max-width: 160px" />
      <q-btn icon="search" :label="$t('common.query')" color="primary" @click="doSearch" />
    </div>

    <q-table :rows="store.parkingVehicles" :columns="columns" row-key="plateNo"
      flat bordered :loading="store.loading">
      <template #body-cell-status="{ row }">
        <q-chip color="positive" text-color="white" dense>{{ row.status }}</q-chip>
      </template>
      <template #loading>
        <SkeletonLoader type="table" :columns="columns.length" :rows="5" />
      </template>
      <template #no-data>
        <EmptyState icon="time_to_leave" :title="$t('empty.noData')" description="暂无在场车辆" />
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useXltStore } from '@/stores/xlt';
import SkeletonLoader from '@/components/SkeletonLoader.vue';
import EmptyState from '@/components/EmptyState.vue';
const store = useXltStore();
const plateNo = ref('');
const parkCode = ref('PK001');

const columns = [
  { name: 'plateNo', label: '车牌号', field: 'plateNo', align: 'left' as const },
  { name: 'plateColor', label: '颜色', field: 'plateColor' },
  { name: 'entryTime', label: '入场时间', field: 'entryTime', align: 'center' as const },
  { name: 'vehicleType', label: '车型', field: 'vehicleType', align: 'center' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
];

function doSearch() { void store.fetchParkingVehicles(parkCode.value, plateNo.value || undefined); }
</script>
