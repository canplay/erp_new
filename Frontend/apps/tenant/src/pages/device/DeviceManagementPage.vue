<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('device.title') }}</div>

    <!-- 统计卡片 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="devices" size="40px" color="primary" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.totalDevices') }}</div>
            <div class="text-h6">{{ statistics.total }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="phone_android" size="40px" color="positive" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.mobileDevices') }}</div>
            <div class="text-h6">{{ statistics.mobile }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="computer" size="40px" color="info" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.desktopDevices') }}</div>
            <div class="text-h6">{{ statistics.desktop }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="warning" size="40px" color="negative" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('device.activeSessions') }}</div>
            <div class="text-h6">{{ statistics.active }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 操作栏 -->
    <div class="row q-mb-md items-center">
      <div class="col">
        <q-input
          v-model="searchKeyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="max-width: 300px"
          @update:model-value="loadDevices"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
      </div>
      <div class="row q-gutter-sm">
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="loadDevices" />
      </div>
    </div>

    <!-- 设备列表 -->
    <q-card flat bordered>
      <q-table
        :rows="deviceList"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
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
            {{ getDeviceTypeLabel(row.deviceType) }}
          </q-td>
        </template>

        <!-- 操作系统 -->
        <template #body-cell-os="{ row }">
          <q-td>
            <q-chip dense size="sm" :color="getOsColor(row.os)" text-color="white">
              {{ row.os || 'Unknown' }}
            </q-chip>
          </q-td>
        </template>

        <!-- 浏览器 -->
        <template #body-cell-browser="{ row }">
          <q-td>
            <span class="text-body2">{{ row.browser || '-' }}</span>
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
            <div v-if="row.lastActiveTime" class="text-caption text-grey">
              {{ $t('device.lastActive') }}: {{ row.lastActiveTime }}
            </div>
          </q-td>
        </template>

        <!-- 状态 -->
        <template #body-cell-status="{ row }">
          <q-td>
            <q-badge
              :color="row.isActive ? 'positive' : 'grey'"
              :label="row.isActive ? $t('device.online') : $t('device.offline')"
            />
            <q-badge
              v-if="row.isTrusted"
              color="primary"
              :label="$t('device.trusted')"
              class="q-ml-sm"
            />
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
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
          <q-btn
            flat
            dense
            round
            :icon="row.isTrusted ? 'verified_user' : 'verified'"
            :color="row.isTrusted ? 'positive' : 'grey'"
            @click="toggleTrust(row)"
          >
            <q-tooltip>{{ row.isTrusted ? $t('device.untrust') : $t('device.trust') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense round :icon="matDelete" color="negative" @click="deleteDevice(row)">
            <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
          </q-btn>
        </template>

        <!-- 加载状态 -->
        <template #loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 空状态 -->
        <template #no-data>
          <div class="full-width row flex-center text-grey-6 q-pa-lg">
            <q-icon name="devices" size="48px" class="q-mb-sm" />
            <div>{{ $t('device.noDevices') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 删除确认对话框 -->
    <ConfirmDialog
      ref="deleteDialogRef"
      :title="$t('common.confirmDelete')"
      :message="$t('device.deleteConfirmMessage')"
      icon="delete"
      confirm-color="negative"
      @confirm="confirmDelete"
    />
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file DeviceManagementPage.vue
 * @description 登录设备管理页面
 * @date 2026-04-06
 */

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import {
  getAllDevices,
  kickDevice as apiKickDevice,
  trustDevice,
  untrustDevice,
  deleteDevice as apiDeleteDevice,
  getDeviceStatistics,
  type LoginDevice,
} from '@/api/device';
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';

// Material Icons
const { t } = useI18n();
const matRefresh = 'refresh';
const matLogout = 'logout';
const matDelete = 'delete';


const $q = useQuasar();

// 状态
const loading = ref(false);
const searchKeyword = ref('');
const deviceList = ref<LoginDevice[]>([]);
const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });
const deleteDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);
const deleteTargetId = ref<number | null>(null);

// 统计数据
const statistics = ref({
  total: 0,
  mobile: 0,
  desktop: 0,
  active: 0,
});

// 表格列定义
const columns = computed(() => [
  { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
  { name: 'userName', label: t('device.user'), field: 'userName', align: 'left' as const },
  { name: 'deviceType', label: t('device.deviceType'), field: 'deviceType', align: 'left' as const },
  { name: 'os', label: t('device.os'), field: 'os', align: 'left' as const },
  { name: 'browser', label: t('device.browser'), field: 'browser', align: 'left' as const },
  { name: 'ipAddress', label: t('device.ipAddress'), field: 'ipAddress', align: 'left' as const },
  { name: 'login_time', label: t('device.login_time'), field: 'login_time', align: 'left' as const },
  { name: 'status', label: t('device.status'), field: 'status', align: 'center' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

// 方法

/**
 * @brief 加载设备列表
 */
async function loadDevices() {
  loading.value = true;
  try {
    const response = await getAllDevices({
      keyword: searchKeyword.value,
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    });
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: LoginDevice[]; data?: LoginDevice[]; total?: number };
    deviceList.value = respData.list || respData.data || [];
    pagination.value.total = respData.total || 0;
  } catch (error) {
    logger.error('【加载设备列表失败】', error);
    $q.notify({
      type: 'negative',
      message: t('device.loadFailed'),
    });
  } finally {
    loading.value = false;
  }
}

/**
 * @brief 加载统计数据
 */
async function loadStatistics() {
  try {
    const response = await getDeviceStatistics();
    // 类型断言：兼容新旧格式（已展开的字段）
    const respData = response as { data?: { total?: number; mobile?: number; desktop?: number; active?: number } };
    const stats = respData.data || { total: 0, mobile: 0, desktop: 0, active: 0 };
    statistics.value = {
      total: stats.total || 0,
      mobile: stats.mobile || 0,
      desktop: stats.desktop || 0,
      active: stats.active || 0,
    };
  } catch (error) {
    logger.error('【加载统计数据失败】', error);
  }
}

/**
 * @brief 分页请求
 */
function onRequest(props: { pagination: { page: number; rowsPerPage: number } }): void {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  void loadDevices();
}

/**
 * @brief 获取设备图标
 */
function getDeviceIcon(deviceType: string) {
  const icons: Record<string, string> = {
    desktop: 'computer',
    mobile: 'phone_android',
    tablet: 'tablet_android',
    other: 'devices',
  };
  return icons[deviceType] || 'devices';
}

/**
 * @brief 获取设备类型颜色
 */
function getDeviceTypeColor(deviceType: string) {
  const colors: Record<string, string> = {
    desktop: 'info',
    mobile: 'positive',
    tablet: 'warning',
    other: 'grey',
  };
  return colors[deviceType] || 'grey';
}

/**
 * @brief 获取设备类型标签
 */
function getDeviceTypeLabel(deviceType: string) {
  const labels: Record<string, string> = {
    desktop: t('device.desktop'),
    mobile: t('device.mobile'),
    tablet: t('device.tablet'),
    other: t('device.other'),
  };
  return labels[deviceType] || deviceType;
}

/**
 * @brief 获取操作系统颜色
 */
function getOsColor(os: string | undefined) {
  if (!os) return 'grey';
  const lowerOs = os.toLowerCase();
  if (lowerOs.includes('windows')) return 'blue';
  if (lowerOs.includes('mac') || lowerOs.includes('ios')) return 'grey-8';
  if (lowerOs.includes('linux')) return 'brown';
  if (lowerOs.includes('android')) return 'green';
  return 'grey';
}

/**
 * @brief 踢出设备
 */
async function kickDevice(row: LoginDevice): Promise<void> {
  try {
    await apiKickDevice(row.id);
    $q.notify({
      type: 'positive',
      message: t('device.kickSuccess'),
    });
    await Promise.all([loadDevices(), loadStatistics()]);
  } catch (error) {
    logger.error('【踢出设备失败】', error);
    $q.notify({
      type: 'negative',
      message: t('device.kickFailed'),
    });
  }
}

/**
 * @brief 切换信任状态
 */
async function toggleTrust(row: LoginDevice) {
  try {
    if (row.isTrusted) {
      await untrustDevice(row.id);
    } else {
      await trustDevice(row.id);
    }
    $q.notify({
      type: 'positive',
      message: row.isTrusted ? t('device.untrustSuccess') : t('device.trustSuccess'),
    });
    await loadDevices();
  } catch (error) {
    logger.error('【切换信任状态失败】', error);
    $q.notify({
      type: 'negative',
      message: t('device.toggleTrustFailed'),
    });
  }
}

/**
 * @brief 删除设备
 */
function deleteDevice(row: LoginDevice) {
  deleteTargetId.value = row.id;
  deleteDialogRef.value?.open();
}

/**
 * @brief 确认删除
 */
async function confirmDelete(): Promise<void> {
  if (deleteTargetId.value === null) return;
  try {
    await apiDeleteDevice(deleteTargetId.value);
    $q.notify({
      type: 'positive',
      message: t('device.deleteSuccess'),
    });
    await Promise.all([loadDevices(), loadStatistics()]);
  } catch (error) {
    logger.error('【删除设备失败】', error);
    $q.notify({
      type: 'negative',
      message: t('device.deleteFailed'),
    });
  }
}

// 生命周期
onMounted(() => {
  void Promise.all([loadDevices(), loadStatistics()]);
});
</script>
