<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">{{ $t('parking.xltDashboardTitle') }}</div>

    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col">
        <q-card-section class="row items-center">
          <q-icon name="local_parking" size="36px" color="primary" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.currentParking') }}</div>
            <div class="text-h6">{{ store.currentParking }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col">
        <q-card-section class="row items-center">
          <q-icon name="directions_car" size="36px" color="positive" class="q-mr-md" />
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
          <q-icon name="devices" size="36px" color="info" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.onlineDevices') }}</div>
            <div class="text-h6">{{ store.devices.length }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <div class="row q-gutter-md">
      <q-card flat bordered class="col">
        <q-card-section><div class="text-h6">{{ $t('parking.passTrend') }}</div></q-card-section>
        <q-separator />
        <q-card-section style="height: 300px">
          <div class="text-grey text-center" style="padding-top: 120px">{{ $t('parking.chartPlaceholder') }}</div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col-4">
        <q-card-section><div class="text-h6">{{ $t('parking.deviceOnlineStatus') }}</div></q-card-section>
        <q-separator />
        <q-card-section>
          <q-list v-if="store.devices.length" dense>
            <q-item v-for="d in store.devices" :key="d.sn">
              <q-item-section avatar><q-icon :name="d.last_heartbeat ? 'check_circle' : 'radio_button_unchecked'" :color="d.last_heartbeat ? 'positive' : 'negative'" /></q-item-section>
              <q-item-section><q-item-label>{{ d.sn }}</q-item-label><q-item-label caption>{{ d.dev_info }}</q-item-label></q-item-section>
            </q-item>
          </q-list>
          <EmptyState v-else icon="devices" :title="$t('common.noResults')" :description="$t('common.noResults')" />
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">import { useI18n } from 'vue-i18n';

import { onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useXltStore } from '@/stores/xlt';
import EmptyState from '@/components/EmptyState.vue';
const { t } = useI18n();
const $q = useQuasar();
const store = useXltStore();
onMounted(async () => { try { await store.fetchDevices(); } catch { $q.notify({ type: 'negative', message: t('parking.loadDeviceDataFailed') }); } });
</script>
