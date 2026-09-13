<template>
  <div>
    <div class="row justify-between q-mb-sm">
      <div class="text-subtitle1">API 调用记录</div>
      <q-btn flat dense color="negative" icon="delete_sweep" :label="$t('common.clear')" @click="$emit('clear')" />
    </div>
    <q-table
      :rows="apiCallRecords"
      :columns="apiColumns"
      row-key="id"
      flat
      dense
      hide-pagination
      :rows-per-page-options="[0]"
      class="api-table"
    >
      <template v-slot:body-cell-method="props">
        <q-td :props="props"><q-badge :color="getMethodColor(props.value)" :label="props.value" /></q-td>
      </template>
      <template v-slot:body-cell-response_time="props">
        <q-td :props="props"><span :class="getResponseTimeClass(props.value)">{{ props.value }}ms</span></q-td>
      </template>
      <template v-slot:body-cell-timestamp="props">
        <q-td :props="props">{{ formatTime(props.value) }}</q-td>
      </template>
      <template v-slot:no-data>
        <div class="text-center q-pa-lg text-grey"><q-icon name="api" size="48px" /><div class="q-mt-sm">暂无 API 调用记录</div></div>
      </template>
    </q-table>
  </div>
</template>

<script setup lang="ts">
import type { QTableProps } from 'quasar';

defineProps<{
  apiCallRecords: { id: string; method: string; path: string; status: number; response_time: number; timestamp: number }[];
  apiColumns: QTableProps['columns'];
  getMethodColor: (method: string) => string;
  getResponseTimeClass: (time: number) => string;
  formatTime: (ts: number) => string;
}>();

defineEmits<{ clear: [] }>();
</script>
