<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('device.abnormalLogin') }}</div>

    <!-- 统计卡片 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="warning" size="40px" color="negative" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.abnormalCount') }}</div>
            <div class="text-h6">{{ statistics.abnormalCount }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="schedule" size="40px" color="warning" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.recentAbnormal') }}</div>
            <div class="text-h6">{{ statistics.recentCount }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="vpn_key" size="40px" color="info" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.unknownLocation') }}</div>
            <div class="text-h6">{{ statistics.unknownLocationCount }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 筛选栏 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section class="row q-gutter-md items-center">
        <q-select
          v-model="filters.riskLevel"
          :options="riskLevelOptions"
          :label="$t('device.riskLevel')"
          dense
          outlined
          clearable
          emit-value
          map-options
          style="min-width: 150px"
          @update:model-value="loadAbnormalLogins"
        />
        <q-input
          v-model="filters.keyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="min-width: 200px"
          @update:model-value="loadAbnormalLogins"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
        <q-space />
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="resetFilters" />
      </q-card-section>
    </q-card>

    <!-- 异常登录列表 -->
    <q-card flat bordered>
      <q-table
        :rows="abnormalLogins"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
        @request="onRequest"
      >
        <!-- 风险等级 -->
        <template #body-cell-riskLevel="{ row }">
          <q-td>
            <q-badge :color="getRiskLevelColor(row.riskLevel)" :label="getRiskLevelLabel(row.riskLevel)" />
          </q-td>
        </template>

        <!-- 登录类型 -->
        <template #body-cell-loginType="{ row }">
          <q-td>
            <q-badge :color="getLoginTypeColor(row.loginType)" :label="getLoginTypeLabel(row.loginType)" />
          </q-td>
        </template>

        <!-- IP地址 -->
        <template #body-cell-ipAddress="{ row }">
          <q-td>
            <span class="text-body2">{{ row.ipAddress }}</span>
            <q-tooltip v-if="row.ipLocation">{{ row.ipLocation }}</q-tooltip>
          </q-td>
        </template>

        <!-- 登录时间 -->
        <template #body-cell-login_time="{ row }">
          <q-td>
            <div class="text-body2">{{ row.login_time }}</div>
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-btn flat dense round :icon="matVisibility" @click="viewDetail(row)">
            <q-tooltip>{{ $t('common.view') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.isActive"
            flat
            dense
            round
            :icon="matLogout"
            color="negative"
            @click="kickDevice(row)"
          >
            <q-tooltip>{{ $t('device.kick') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense round :icon="matBlock" color="warning" @click="blockDevice(row)">
            <q-tooltip>{{ $t('device.block') }}</q-tooltip>
          </q-btn>
        </template>

        <!-- 加载状态 -->
        <template #loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 空状态 -->
        <template #no-data>
          <div class="full-width row flex-center text-grey-6 q-pa-lg">
            <q-icon name="security" size="48px" class="q-mb-sm" />
            <div>{{ $t('device.noAbnormal') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 详情对话框 -->
    <q-dialog v-model="showDetailDialog" persistent>
      <q-card style="min-width: 600px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('device.abnormalDetail') }}</div>
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
              <q-item-section>{{ $t('device.riskLevel') }}:</q-item-section>
              <q-item-section side>
                <q-badge :color="getRiskLevelColor(currentLogin.riskLevel)" :label="getRiskLevelLabel(currentLogin.riskLevel)" />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('device.loginType') }}:</q-item-section>
              <q-item-section side>
                <q-badge :color="getLoginTypeColor(currentLogin.loginType)" :label="getLoginTypeLabel(currentLogin.loginType)" />
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
            <q-item v-if="currentLogin.reason">
              <q-item-section>{{ $t('device.abnormalReason') }}:</q-item-section>
              <q-item-section side>{{ currentLogin.reason }}</q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" v-close-popup />
          <q-btn
            v-if="currentLogin?.isActive"
            color="negative"
            :label="$t('device.kickDevice')"
            @click="kickCurrentDevice"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file AbnormalLoginPage.vue
 * @description 异常登录记录页面
 * @date 2026-05-05
 */

import { ref, reactive, onMounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { useDeviceStore } from '@/stores/device';
import type { LoginDevice, AbnormalLoginRecord } from '@/api/device';

// Material Icons
const { t } = useI18n();
const matRefresh = 'refresh';
const matVisibility = 'visibility';
const matLogout = 'logout';
const matBlock = 'block';


const $q = useQuasar();
const deviceStore = useDeviceStore();

// 状态
const loading = ref(false);
const abnormalLogins = ref<LoginDevice[]>([]);
const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });
const showDetailDialog = ref(false);
const currentLogin = ref<AbnormalLoginRecord | null>(null);

// 筛选条件
const filters = reactive({
  riskLevel: null,
  keyword: '',
});

// 统计数据
const statistics = ref({
  abnormalCount: 0,
  recentCount: 0,
  unknownLocationCount: 0,
});

// 选项配置
const riskLevelOptions = [
  { label: t('device.riskLow'), value: 'low' },
  { label: t('device.riskMedium'), value: 'medium' },
  { label: t('device.riskHigh'), value: 'high' },
  { label: t('device.riskCritical'), value: 'critical' },
];

// 表格列定义
const columns = computed(() => [
  { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
  { name: 'userName', label: t('device.user'), field: 'userName', align: 'left' as const },
  { name: 'riskLevel', label: t('device.riskLevel'), field: 'riskLevel', align: 'center' as const },
  { name: 'loginType', label: t('device.loginType'), field: 'loginType', align: 'center' as const },
  { name: 'ipAddress', label: t('device.ipAddress'), field: 'ipAddress', align: 'left' as const },
  { name: 'login_time', label: t('device.login_time'), field: 'login_time', align: 'left' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

// 方法
async function loadAbnormalLogins() {
  loading.value = true;
  try {
    const params: Record<string, unknown> = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };
    if (filters.riskLevel) params.riskLevel = filters.riskLevel;
    if (filters.keyword) params.keyword = filters.keyword;

    await deviceStore.fetchAbnormalLogins(params);
    abnormalLogins.value = deviceStore.abnormalLogins;
    pagination.value.total = deviceStore.pagination.total;

    // 更新统计
    statistics.value = {
      abnormalCount: deviceStore.abnormalLogins.length,
      recentCount: deviceStore.abnormalLogins.filter((l) => {
        // 后端返回 epoch 秒，需转毫秒
        const ts = Number(l.login_time);
        const loginDate = new Date(ts < 1e12 ? ts * 1000 : ts);
        const now = new Date();
        const diff = now.getTime() - loginDate.getTime();
        return diff < 24 * 60 * 60 * 1000; // 24小时内
      }).length,
      unknownLocationCount: deviceStore.abnormalLogins.filter((l) => !l.ipLocation).length,
    };
  } finally {
    loading.value = false;
  }
}

function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  void loadAbnormalLogins();
}

function resetFilters() {
  filters.riskLevel = null;
  filters.keyword = '';
  void loadAbnormalLogins();
}

function viewDetail(row: LoginDevice) {
  currentLogin.value = row;
  showDetailDialog.value = true;
}

async function kickDevice(row: LoginDevice) {
  try {
    await deviceStore.kickDeviceById(row.id);
    $q.notify({ type: 'positive', message: t('device.kickSuccess') });
    void loadAbnormalLogins();
  } catch {
    $q.notify({ type: 'negative', message: t('device.kickFailed') });
  }
}

async function kickCurrentDevice() {
  if (!currentLogin.value) return;
  await kickDevice(currentLogin.value);
  showDetailDialog.value = false;
}

function blockDevice(row: LoginDevice) {
  $q.dialog({
    title: t('device.blockConfirm'),
    message: t('device.blockConfirmMessage', { ip: row.ipAddress }),
    cancel: true,
    persistent: true,
  }).onOk(() => {
    $q.notify({ type: 'info', message: t('device.blockSuccess') });
  });
}

function getRiskLevelColor(level?: string) {
  const colors: Record<string, string> = {
    low: 'positive',
    medium: 'warning',
    high: 'orange',
    critical: 'negative',
  };
  return colors[level || 'low'] || 'grey';
}

function getRiskLevelLabel(level?: string) {
  const labels: Record<string, string> = {
    low: t('device.riskLow'),
    medium: t('device.riskMedium'),
    high: t('device.riskHigh'),
    critical: t('device.riskCritical'),
  };
  return labels[level || 'low'] || level || '';
}

function getLoginTypeColor(type?: string) {
  const colors: Record<string, string> = {
    new_location: 'info',
    new_device: 'warning',
    suspicious: 'negative',
    brute_force: 'negative',
  };
  return colors[type || ''] || 'grey';
}

function getLoginTypeLabel(type?: string) {
  const labels: Record<string, string> = {
    new_location: t('device.loginTypeNewLocation'),
    new_device: t('device.loginTypeNewDevice'),
    suspicious: t('device.loginTypeSuspicious'),
    brute_force: t('device.loginTypeBruteForce'),
  };
  return labels[type || ''] || type || '';
}

function getDeviceTypeLabel(type?: string) {
  const labels: Record<string, string> = {
    desktop: t('device.desktop'),
    mobile: t('device.mobile'),
    tablet: t('device.tablet'),
    other: t('device.other'),
  };
  return labels[type || 'other'] || type || '';
}

onMounted(() => {
  void loadAbnormalLogins();
});
</script>
