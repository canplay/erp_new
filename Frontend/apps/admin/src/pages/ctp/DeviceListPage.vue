<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">{{ $t('parking.ctpTitle') }}</div>

    <!-- 统计卡片 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 160px">
        <q-card-section class="row items-center">
          <q-icon name="lock" size="36px" color="primary" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.deviceTotal') }}</div>
            <div class="text-h6">{{ store.deviceCount }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 160px">
        <q-card-section class="row items-center">
          <q-icon name="lock_open" size="36px" color="positive" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.deviceOnline') }}</div>
            <div class="text-h6">{{ store.onlineDevices.length }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 160px">
        <q-card-section class="row items-center">
          <q-icon name="wifi_off" size="36px" color="negative" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('parking.deviceOffline') }}</div>
            <div class="text-h6">{{ store.offlineDevices.length }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 操作栏 -->
    <div class="row q-mb-md items-center">
      <q-input v-model="search" :placeholder="$t('parking.searchPlaceholder')" dense outlined clearable
        style="max-width: 280px" class="q-mr-md" debounce="300" @update:model-value="onSearch" />
      <q-btn flat icon="refresh" :label="$t('parking.refresh')" @click="loadDevices" />
    </div>

    <!-- 设备列表 -->
    <q-table :rows="store.devices" :columns="responsiveColumns" row-key="id" :loading="store.loading"
      flat bordered :pagination="{ rowsPerPage: 15, page: store.page, rowsNumber: store.total }"
      @request="onRequest" @row-click="(_, row) => $router.push(`/ctp/device/${row.device_no}`)"
      :wrap-cells="false">

      <template #body-cell-status="{ row }">
        <q-chip :color="statusColor(row.status)" text-color="white" dense size="sm">{{ row.status }}</q-chip>
      </template>
      <template #body-cell-battery_level="{ row }">
        <span>{{ row.battery_level || '-' }}</span>
      </template>
      <template #body-cell-voltage="{ row }">
        <span>{{ row.voltage || '-' }}</span>
      </template>
      <template #body-cell-actions="{ row }">
        <q-btn flat dense icon="tune" size="sm" color="primary"
          @click.stop="$router.push(`/ctp/device/${row.device_no}`)" />
      </template>
      <template #loading>
        <SkeletonLoader type="table" :columns="columns.length" :rows="5" />
      </template>
      <template #no-data>
        <EmptyState icon="lock" :title="$t('empty.noData')" description="暂无设备数据" />
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useCtpStore } from '@/stores/ctp';
import SkeletonLoader from '@/components/SkeletonLoader.vue';
import EmptyState from '@/components/EmptyState.vue';

const { t } = useI18n();
const $q = useQuasar();

const store = useCtpStore();
const search = ref('');
const isMobile = ref($q.screen.lt.md);

const columns = [
  { name: 'device_no', label: t('parking.deviceNo'), field: 'device_no', align: 'left' as const, sortable: true },
  { name: 'status', label: t('parking.status'), field: 'status', align: 'center' as const },
  { name: 'battery_level', label: t('parking.battery'), field: 'battery_level', align: 'center' as const },
  { name: 'voltage', label: t('parking.voltage'), field: 'voltage', align: 'center' as const },
  { name: 'park_code', label: t('parking.parkCode'), field: 'park_code', align: 'center' as const },
  { name: 'updated_at', label: t('parking.lastUpdate'), field: 'updated_at', align: 'center' as const },
  { name: 'actions', label: t('parking.deviceActions'), field: 'actions', align: 'center' as const },
];

// 移动端显示更少列
const responsiveColumns = computed(() => {
  if (isMobile.value) {
    return columns.filter(c => !['voltage', 'park_code', 'updated_at'].includes(c.name));
  }
  return columns;
});

function statusColor(status: string): string {
  const map: Record<string, string> = { Locked: 'positive', Unlocked: 'warning', Offline: 'negative', Fault: 'negative' };
  return map[status] || 'grey';
}

async function loadDevices() {
  await store.fetchDevices({ page: store.page, page_size: store.page_size });
}

function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  store.page = props.pagination.page;
  store.page_size = props.pagination.rowsPerPage;
  void loadDevices();
}

function onSearch() {
  store.page = 1;
  void loadDevices();
}

onMounted(async () => { try { await loadDevices(); } catch { $q.notify({ type: 'negative', message: '加载设备列表失败' }); } });
</script>
