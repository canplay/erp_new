<template>
  <q-page class="q-pa-md">
    <q-breadcrumbs class="q-mb-md">
      <q-breadcrumbs-el :label="$t('parking.ctpTitle')" to="/ctp" />
      <q-breadcrumbs-el :label="deviceNo" />
    </q-breadcrumbs>

    <SkeletonLoader v-if="store.loading" type="card" :rows="6" />
    <template v-else-if="device">
      <div class="row q-gutter-md">
        <!-- 基本信息 -->
        <q-card flat bordered class="col-4">
          <q-card-section>
            <div class="text-h6">{{ $t('ctp.basicInfo') }}</div>
          </q-card-section>
          <q-separator />
          <q-card-section>
            <q-list dense>
              <q-item><q-item-section><q-item-label>{{ $t('ctp.deviceNo') }}</q-item-label><q-item-label caption>{{ device.device_no }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('ctp.statusLabel') }}</q-item-label><q-item-label caption><q-chip :color="statusColor(device.status)" text-color="white" dense>{{ device.status }}</q-chip></q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('ctp.batteryLevel') }}</q-item-label><q-item-label caption>{{ device.battery_level || '-' }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('ctp.voltageLabel') }}</q-item-label><q-item-label caption>{{ device.voltage || '-' }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('ctp.parkCode') }}</q-item-label><q-item-label caption>{{ device.park_code || '-' }}</q-item-label></q-item-section></q-item>
            </q-list>
          </q-card-section>
        </q-card>

        <!-- 状态字解析 -->
        <q-card flat bordered class="col-4">
          <q-card-section>
            <div class="text-h6">{{ $t('ctp.statusParse') }}</div>
          </q-card-section>
          <q-separator />
          <q-card-section>
            <template v-if="device.status_parsed">
              <q-list dense>
                <q-item><q-item-section><q-item-label>{{ $t('ctp.lockState') }}</q-item-label><q-item-label caption>{{ device.status_parsed.lock_state }}</q-item-label></q-item-section></q-item>
                <q-item><q-item-section><q-item-label>{{ $t('ctp.leftCoil') }}</q-item-label><q-item-label caption>{{ device.status_parsed.left_coil }}</q-item-label></q-item-section></q-item>
                <q-item><q-item-section><q-item-label>{{ $t('ctp.rightCoil') }}</q-item-label><q-item-label caption>{{ device.status_parsed.right_coil }}</q-item-label></q-item-section></q-item>
                <q-item><q-item-section><q-item-label>{{ $t('ctp.alarmLabel') }}</q-item-label><q-item-label caption>{{ device.status_parsed.alarm ? $t('ctp.yes') : $t('ctp.no') }}</q-item-label></q-item-section></q-item>
                <q-item><q-item-section><q-item-label>{{ $t('ctp.resetFlag') }}</q-item-label><q-item-label caption>{{ device.status_parsed.reset_flag ? $t('ctp.yes') : $t('ctp.no') }}</q-item-label></q-item-section></q-item>
              </q-list>
            </template>
            <div v-else class="text-grey">{{ $t('ctp.noStatusData') }}</div>
          </q-card-section>
        </q-card>

        <!-- 控制面板 -->
        <q-card flat bordered class="col-4">
          <q-card-section>
            <div class="text-h6">{{ $t('ctp.controlPanel') }}</div>
          </q-card-section>
          <q-separator />
          <q-card-section class="q-gutter-sm">
            <q-btn color="positive" icon="lock_open" :label="$t('ctp.boardUp')" class="full-width"
              @click="sendControl('up')" :disable="sending" />
            <q-btn color="negative" icon="lock" :label="$t('ctp.boardDown')" class="full-width"
              @click="sendControl('down')" :disable="sending" />
            <q-btn color="info" icon="sync" :label="$t('ctp.syncStatus')" class="full-width"
              @click="sendControl('syn')" :disable="sending" />
          </q-card-section>
        </q-card>
      </div>

      <!-- 统计数据 -->
      <q-card v-if="device.status_two_parsed" flat bordered class="q-mt-md">
        <q-card-section>
          <div class="text-h6">{{ $t('ctp.countStatistics') }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section class="row q-gutter-md">
          <q-chip color="primary" text-color="white" icon="directions_car">{{ $t('ctp.entryCount') }} {{ device.status_two_parsed.total_entry_count }}</q-chip>
          <q-chip color="warning" text-color="white" icon="exit_to_app">{{ $t('ctp.exitCount') }} {{ device.status_two_parsed.total_exit_count }}</q-chip>
          <q-chip color="negative" text-color="white" icon="report_problem">{{ $t('ctp.theftCount') }} {{ device.status_two_parsed.total_theft_count }}</q-chip>
          <q-chip color="info" text-color="white" icon="local_parking">{{ $t('ctp.currentVehicles') }} {{ device.status_two_parsed.current_occupancy }}</q-chip>
        </q-card-section>
      </q-card>
    </template>
    <div v-else class="text-grey text-center q-mt-xl">
      <q-icon name="lock" size="48px" />
      <div class="q-mt-sm">{{ $t('empty.noData') }}</div>
      <div>{{ $t('ctp.deviceNotFound') }}</div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useCtpStore } from '@/stores/device';
import type { CmdType } from '@/api/ctp';
import SkeletonLoader from '@erp-new-frontend-monorepo/components/src/SkeletonLoader.vue';

const route = useRoute();
const store = useCtpStore();
const deviceNo = computed(() => route.params.deviceNo as string);
const device = computed(() => store.currentDevice);
const sending = ref(false);

function statusColor(status: string): string {
  const map: Record<string, string> = { Locked: 'positive', Unlocked: 'warning', Offline: 'negative', Fault: 'negative' };
  return map[status] || 'grey';
}

async function sendControl(cmd: CmdType) {
  sending.value = true;
  try {
    await store.sendControl({ factory_id: '1', device_no: deviceNo.value, cmd_type: cmd });
    await store.fetchDevice(deviceNo.value);
  } finally {
    sending.value = false;
  }
}

onMounted(() => store.fetchDevice(deviceNo.value));
</script>
