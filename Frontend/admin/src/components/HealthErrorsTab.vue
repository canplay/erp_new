<template>
  <div>
    <div class="row justify-between q-mb-sm">
      <div class="text-subtitle1">前端错误记录</div>
      <div><q-btn flat dense color="negative" icon="check" :label="$t('common.allProcessed')" @click="$emit('resolveAll')" class="q-mr-sm" /><q-btn flat dense color="grey" icon="delete_sweep" :label="$t('common.clear')" @click="$emit('clear')" /></div>
    </div>
    <q-table
      :rows="errorRecords"
      :columns="errorColumns"
      row-key="id"
      flat
      dense
      hide-pagination
      :rows-per-page-options="[0]"
    >
      <template v-slot:body-cell-type="props">
        <q-td :props="props"><q-badge :color="getErrorTypeColor(props.value)" :label="props.value" /></q-td>
      </template>
      <template v-slot:body-cell-resolved="props">
        <q-td :props="props"><q-icon :name="props.value ? 'check_circle' : 'pending'" :color="props.value ? 'positive' : 'warning'" size="20px" /></q-td>
      </template>
      <template v-slot:body-cell-actions="props">
        <q-td :props="props"><q-btn flat dense icon="visibility" size="sm" color="primary" @click="$emit('showDetail', props.row)" /></q-td>
      </template>
      <template v-slot:no-data>
        <div class="text-center q-pa-lg text-grey"><q-icon name="check_circle" size="48px" /><div class="q-mt-sm">暂无错误记录</div></div>
      </template>
    </q-table>
  </div>
</template>

<script setup lang="ts">
import type { QTableProps } from 'quasar';
import type { FrontendError } from '@/types/monitor';

defineProps<{
  errorRecords: FrontendError[];
  errorColumns: QTableProps['columns'];
  getErrorTypeColor: (type: string) => string;
}>();

defineEmits<{ resolveAll: []; clear: []; showDetail: [error: FrontendError] }>();
</script>
