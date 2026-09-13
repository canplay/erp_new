<template>
  <div>
    <!-- 设备下钻 -->
    <q-dialog v-model="deviceDialog">
      <q-card style="width: 720px; max-width: 92vw">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ i18nT('raw.sanl_dev_list', { title: deviceDialogTitle }) }}</div>
          <q-space />
          <q-btn flat round dense icon="close" v-close-popup />
        </q-card-section>
        <q-separator />
        <q-card-section class="q-pt-sm">
          <PageTable
            :rows="deviceDialogRows"
            :columns="deviceColumns"
            :total="deviceDialogRows.length"
            :rows-per-page="10"
            flat
            bordered
          >
            <template #body-cell-healthScore="{ row }">
              <q-td>
                <HealthScoreBadge :score="row.healthScore" :level="row.healthLevel" />
              </q-td>
            </template>
          </PageTable>
        </q-card-section>
      </q-card>
    </q-dialog>

    <!-- 隐患下钻 -->
    <q-dialog v-model="dangerDialog">
      <q-card style="width: 760px; max-width: 92vw">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ i18nT('raw.sanl_dan_list', { title: dangerDialogTitle }) }}</div>
          <q-space />
          <q-btn flat round dense icon="close" v-close-popup />
        </q-card-section>
        <q-separator />
        <q-card-section class="q-pt-sm">
          <PageTable
            :rows="dangerDialogRows"
            :columns="dangerColumns"
            :total="dangerDialogRows.length"
            :rows-per-page="10"
            flat
            bordered
          >
            <template #body-cell-level="{ row }">
              <q-td><DangerLevelTag :level="row.level" /></q-td>
            </template>
            <template #body-cell-status="{ row }">
              <q-td>
                <q-badge :color="dangerStatusColor(row.status)">{{
                  dangerStatusText(row.status)
                }}</q-badge>
              </q-td>
            </template>
            <template #body-cell-createTime="{ row }">
              <q-td>{{ formatDateTime(row.createTime) }}</q-td>
            </template>
          </PageTable>
        </q-card-section>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref } from 'vue';
import type { Device, HiddenDanger } from '@erp-new-frontend-monorepo/types';
import { HealthScoreBadge, DangerLevelTag, PageTable } from '@erp-new-frontend-monorepo/components';
import {
  formatDateTime,
  dangerStatusText,
  dangerStatusColor,
} from '@erp-new-frontend-monorepo/utils';

const deviceDialog = ref(false);
const deviceDialogTitle = ref('');
const deviceDialogRows = ref<Device[]>([]);

const dangerDialog = ref(false);
const dangerDialogTitle = ref('');
const dangerDialogRows = ref<HiddenDanger[]>([]);

const deviceColumns = [
  { name: 'code', label: i18nT('raw.s667f13'), field: 'code', align: 'left' as const },
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name', align: 'left' as const },
  {
    name: 'healthScore',
    label: i18nT('raw.s471784'),
    field: 'healthScore',
    align: 'center' as const,
  },
];

const dangerColumns = [
  {
    name: 'description',
    label: i18nT('raw.s9e58f1'),
    field: 'description',
    align: 'left' as const,
  },
  { name: 'level', label: i18nT('raw.s509e53'), field: 'level', align: 'center' as const },
  { name: 'status', label: i18nT('raw.s9fb403'), field: 'status', align: 'center' as const },
  {
    name: 'createTime',
    label: i18nT('raw.s70f7f2'),
    field: 'createTime',
    align: 'center' as const,
  },
];

function openDeviceDialog(title: string, rows: Device[]) {
  deviceDialogTitle.value = title;
  deviceDialogRows.value = rows;
  deviceDialog.value = true;
}

function openDangerDialog(title: string, rows: HiddenDanger[]) {
  dangerDialogTitle.value = title;
  dangerDialogRows.value = rows;
  dangerDialog.value = true;
}

defineExpose({
  openDeviceDialog,
  openDangerDialog,
});
</script>
