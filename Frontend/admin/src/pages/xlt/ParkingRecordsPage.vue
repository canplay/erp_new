<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">车辆进出记录</div>

    <div class="row q-mb-md q-gutter-sm">
      <q-input v-model="filters.plate_no" :placeholder="$t('common.plateNo')" dense outlined style="max-width: 160px" />
      <q-input v-model="filters.park_code" :placeholder="$t('common.parkCode')" dense outlined style="max-width: 140px" />
      <q-btn icon="search" :label="$t('common.query')" color="primary" @click="doSearch" />
      <q-btn flat icon="refresh" :label="$t('common.refresh')" @click="loadRecords" />
    </div>

    <q-table :rows="store.records" :columns="columns" row-key="id" :loading="store.loading"
      flat bordered :pagination="{ rowsPerPage: 20, rowsNumber: store.total }"
      @request="onRequest">
      <template #body-cell-direction="{ row }">
        <q-chip :color="row.direction === 'entry' ? 'positive' : 'warning'" text-color="white" dense>
          {{ row.direction === 'entry' ? '入场' : '出场' }}
        </q-chip>
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue';
import { useXltStore } from '@/stores/xlt';
const store = useXltStore();
const filters = reactive({ plate_no: '', park_code: '' });

const columns = [
  { name: 'plate_no', label: '车牌号', field: 'plate_no', align: 'left' as const, sortable: true },
  { name: 'plate_color', label: '颜色', field: 'plate_color' },
  { name: 'direction', label: '方向', field: 'direction', align: 'center' as const },
  { name: 'event_time', label: '时间', field: 'event_time', align: 'center' as const },
  { name: 'vehicle_type', label: '车型', field: 'vehicle_type' },
  { name: 'amount', label: '金额', field: 'amount', align: 'right' as const },
];

async function loadRecords() { await store.fetchRecords(filters); }
function doSearch() { void loadRecords(); }
function onRequest() { void loadRecords(); }

onMounted(loadRecords);
</script>
