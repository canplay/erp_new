<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('device.loginHistory') }}</div>

    <!-- 统计概览 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="history" size="40px" color="primary" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.totalLogins') }}</div>
            <div class="text-h6">{{ statistics.total }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="today" size="40px" color="positive" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.todayLogins') }}</div>
            <div class="text-h6">{{ statistics.today }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="location_on" size="40px" color="info" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.uniqueLocations') }}</div>
            <div class="text-h6">{{ statistics.uniqueLocations }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="devices" size="40px" color="warning" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.uniqueDevices') }}</div>
            <div class="text-h6">{{ statistics.uniqueDevices }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 时间范围选择 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section class="row q-gutter-md items-center">
        <q-btn-toggle
          v-model="timeRange"
          toggle-color="primary"
          :options="[
            { label: $t('device.today'), value: 'today' },
            { label: $t('device.last7Days'), value: 'week' },
            { label: $t('device.last30Days'), value: 'month' },
            { label: $t('device.all'), value: 'all' },
          ]"
          @update:model-value="loadHistory"
        />
        <q-space />
        <q-input
          v-model="searchKeyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="min-width: 200px"
          @update:model-value="loadHistory"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="resetFilters" />
      </q-card-section>
    </q-card>

    <!-- 登录历史列表 -->
    <q-card flat bordered>
      <q-table
        :rows="loginHistory"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 20 }"
        @request="onRequest"
      >
        <!-- 设备类型 -->
        <template #body-cell-deviceType="{ row }">
          <q-td>
            <q-icon
              :name="getDeviceIcon(row.deviceType)"
              :color="getDeviceTypeColor(row.deviceType)"
              size="24px"
            />
          </q-td>
        </template>

        <!-- IP地址 -->
        <template #body-cell-ipAddress="{ row }">
          <q-td>
            <span class="text-body2">{{ row.ipAddress }}</span>
            <q-tooltip v-if="row.ipLocation">{{ row.ipLocation }}</q-tooltip>
          </q-td>
        </template>

        <!-- 登录状态 -->
        <template #body-cell-status="{ row }">
          <q-td>
            <q-badge :color="row.success ? 'positive' : 'negative'" :label="row.success ? $t('device.success') : $t('device.failed')" />
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-btn flat dense round :icon="matVisibility" @click="viewDetail(row)">
            <q-tooltip>{{ $t('common.view') }}</q-tooltip>
          </q-btn>
        </template>

        <!-- 加载状态 -->
        <template #loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 空状态 -->
        <template #no-data>
          <div class="full-width row flex-center text-grey-6 q-pa-lg">
            <q-icon name="history" size="48px" class="q-mb-sm" />
            <div>{{ $t('device.noHistory') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 详情对话框 -->
    <q-dialog v-model="showDetailDialog" persistent>
      <q-card style="min-width: 500px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('device.loginDetail') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="showDetailDialog = false" />
        </q-card-section>

        <q-separator />

        <q-card-section v-if="currentLogin">
          <q-list dense>
            <q-item>
              <q-item-section>{{ $t('device.user') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.userName }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.status') }}:</q-item-section>
              <q-item-section side>
                <q-badge :color="currentLogin.success ? 'positive' : 'negative'" :label="currentLogin.success ? $t('device.success') : $t('device.failed')" />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.ipAddress') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.ipAddress }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.location') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.ipLocation || '-' }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.deviceType') }}:</q-item-section>
              <q-item-section side>{{ getDeviceTypeLabel(currentLogin.deviceType) }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.os') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.os || '-' }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.browser') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.browser || '-' }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.login_time') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.login_time }}</q-item-section>
            </q-item>
            <q-item v-if="currentLogin.failureReason">
              <q-item-section>{{ $t('device.failureReason') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.failureReason }}</q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file LoginHistoryPage.vue
 * @description 登录历史页面
 * @date 2026-05-05
 */

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { httpClient } from '@/utils/alova';

const { t } = useI18n();
const matRefresh = 'refresh';
const matVisibility = 'visibility';


interface LoginHistoryRecord {
  id: number;
  user_id: number;
  userName: string;
  deviceType: string;
  deviceName?: string;
  browser?: string;
  os?: string;
  ipAddress: string;
  ipLocation?: string;
  login_time: string;
  success: boolean;
  failureReason?: string;
}

const loading = ref(false);
const timeRange = ref('week');
const searchKeyword = ref('');
const loginHistory = ref<LoginHistoryRecord[]>([]);
const pagination = ref({ page: 1, rowsPerPage: 20, total: 0 });
const showDetailDialog = ref(false);
const currentLogin = ref<LoginHistoryRecord | null>(null);

const statistics = ref({
  total: 0,
  today: 0,
  uniqueLocations: 0,
  uniqueDevices: 0,
});

const columns = computed(() => [
  { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
  { name: 'userName', label: t('device.user'), field: 'userName', align: 'left' as const },
  { name: 'deviceType', label: t('device.deviceType'), field: 'deviceType', align: 'center' as const },
  { name: 'ipAddress', label: t('device.ipAddress'), field: 'ipAddress', align: 'left' as const },
  { name: 'os', label: t('device.os'), field: 'os', align: 'left' as const },
  { name: 'login_time', label: t('device.login_time'), field: 'login_time', align: 'left' as const },
  { name: 'status', label: t('device.status'), field: 'status', align: 'center' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

async function loadHistory() {
  loading.value = true;
  try {
    const params: Record<string, unknown> = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };
    if (timeRange.value !== 'all') params.range = timeRange.value;
    if (searchKeyword.value) params.keyword = searchKeyword.value;

    const response = await httpClient.get('/admin/devices/login-history', { params });
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: LoginHistoryRecord[]; data?: LoginHistoryRecord[]; total?: number };
    if (respData.list) {
      loginHistory.value = respData.list;
      pagination.value.total = respData.total || 0;
    } else if (respData.data) {
      loginHistory.value = respData.data;
    }
  } catch (error) {
    console.error('【加载登录历史失败】', error);
  } finally {
    loading.value = false;
  }
}

function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  void loadHistory();
}

function resetFilters() {
  timeRange.value = 'week';
  searchKeyword.value = '';
  void loadHistory();
}

function viewDetail(row: LoginHistoryRecord) {
  currentLogin.value = row;
  showDetailDialog.value = true;
}

function getDeviceIcon(deviceType?: string) {
  const icons: Record<string, string> = {
    desktop: 'computer',
    mobile: 'phone_android',
    tablet: 'tablet_android',
    other: 'devices',
  };
  return icons[deviceType || 'other'] || 'devices';
}

function getDeviceTypeColor(deviceType?: string) {
  const colors: Record<string, string> = {
    desktop: 'info',
    mobile: 'positive',
    tablet: 'warning',
    other: 'grey',
  };
  return colors[deviceType || 'other'] || 'grey';
}

function getDeviceTypeLabel(deviceType?: string) {
  const labels: Record<string, string> = {
    desktop: t('device.desktop'),
    mobile: t('device.mobile'),
    tablet: t('device.tablet'),
    other: t('device.other'),
  };
  return labels[deviceType || 'other'] || deviceType || '';
}

onMounted(() => {
  void loadHistory();
});
</script>
