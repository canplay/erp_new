<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('log.loginLog') }}</div>

    <!-- 搜索筛选 -->
    <AdvancedSearch
      v-model="filters"
      :show-status="true"
      :show-date-range="true"
      :show-date-shortcuts="true"
      :keyword-placeholder="$t('log.searchPlaceholder')"
      :status-label="$t('log.status')"
      :date-range-label="$t('common.dateRange')"
      :status-options="statusOptions"
      keyword-class="col-12 col-sm-6"
      @search="handleSearch"
      @reset="handleReset"
    />

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-between">
        <div class="text-subtitle1">
          {{ $t('log.totalRecords', { count: pagination.rowsNumber }) }}
        </div>
        <div class="q-gutter-sm">
          <q-btn flat :label="$t('common.refresh')" icon="refresh" @click="loadLogs" />
          <q-btn flat color="primary" :label="$t('common.export')" icon="download" @click="handleExport" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 日志表格 -->
    <q-card bordered>
      <q-table
        :rows="logs"
        :columns="columns"
        row-key="id"
        :loading="loading"
        :pagination="pagination"
        @request="onTableRequest"
      >
        <!-- 用户名 -->
        <template v-slot:body-cell-username="props">
          <q-td :props="props">
            <div class="flex items-center">
              <q-avatar size="28px" color="primary" text-color="white" class="q-mr-sm">
                {{ props.row.username?.substring(0, 1).toUpperCase() }}
              </q-avatar>
              <span>{{ props.row.username }}</span>
            </div>
          </q-td>
        </template>

        <!-- 状态 -->
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-chip
              :color="props.row.status === 1 ? 'positive' : 'negative'"
              text-color="white"
              dense
              :icon="props.row.status === 1 ? 'check_circle' : 'error'"
            >
              {{ props.row.status === 1 ? $t('log.success') : $t('log.failed') }}
            </q-chip>
          </q-td>
        </template>

        <!-- IP地址 -->
        <template v-slot:body-cell-ip="props">
          <q-td :props="props">
            <code class="text-grey-7">{{ props.row.ip || '-' }}</code>
          </q-td>
        </template>

        <!-- 用户代理 -->
        <template v-slot:body-cell-user_agent="props">
          <q-td :props="props">
            <q-tooltip>{{ props.row.user_agent || '-' }}</q-tooltip>
            <span class="text-grey-7 ellipsis-2-lines" style="max-width: 200px; display: block;">
              {{ props.row.user_agent || '-' }}
            </span>
          </q-td>
        </template>

        <!-- 错误信息 -->
        <template v-slot:body-cell-error_msg="props">
          <q-td :props="props">
            <q-tooltip v-if="props.row.error_msg">{{ props.row.error_msg }}</q-tooltip>
            <span class="text-negative" v-if="props.row.error_msg">
              {{ props.row.error_msg.substring(0, 50) }}...
            </span>
            <span v-else class="text-grey-5">-</span>
          </q-td>
        </template>

        <!-- 登录时间 -->
        <template v-slot:body-cell-login_time="props">
          <q-td :props="props">
            {{ formatDateTime(props.row.login_time) }}
          </q-td>
        </template>

        <!-- 加载状态 -->
        <template v-slot:loading>
          <q-spinner-dots />
        </template>

        <!-- 空状态 -->
        <template v-slot:no-data="{ message }">
          <div class="full-width row justify-center q-pa-xl text-grey-6">
            <q-icon name="history" size="64px" class="q-mb-md" />
            <div>{{ message || $t('table.noData') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { listLoginLogs, exportLoginLogs, type LoginLog } from '@/api/log';
import AdvancedSearch from '@erp-new-frontend-monorepo/components/src/AdvancedSearch/Main.vue';
import { useExport } from '@erp-new-frontend-monorepo/composables/src/useExport';;
import type { AdvancedFilters } from '@/types/advanced-search'

const { t } = useI18n();
const $q = useQuasar();

const { exportToCSV } = useExport();

// ============ 状态 ============
const loading = ref(false);
const logs = ref<LoginLog[]>([]);

const filters = ref<AdvancedFilters>({
  keyword: '',
  start_date: '',
  end_date: '',
  status: null,
});

const pagination = ref({
  page: 1,
  rowsPerPage: 10,
  rowsNumber: 0,
  sortBy: 'login_time',
  descending: true,
});

// ============ 计算属性 ============

/**
 * @brief 状态选项
 */
const statusOptions = computed(() => [
  { label: t('log.all'), value: '' },
  { label: t('log.success'), value: 1 },
  { label: t('log.failed'), value: 0 },
]);

/**
 * @brief 表格列定义
 */
const columns = computed(() => [
  { name: 'id', label: t('log.id'), field: 'id', align: 'left' as const, sortable: true },
  { name: 'username', label: t('log.username'), field: 'username', align: 'left' as const, sortable: true },
  { name: 'status', label: t('log.status'), field: 'status', align: 'center' as const, sortable: true },
  { name: 'ip', label: t('log.ip'), field: 'ip', align: 'left' as const, sortable: false },
  { name: 'location', label: t('log.location'), field: 'location', align: 'left' as const, sortable: false },
  { name: 'user_agent', label: t('log.user_agent'), field: 'user_agent', align: 'left' as const, sortable: false },
  { name: 'error_msg', label: t('log.error_msg'), field: 'error_msg', align: 'left' as const, sortable: false },
  { name: 'login_time', label: t('log.login_time'), field: 'login_time', align: 'left' as const, sortable: true },
]);

// ============ 方法 ============

/**
 * @brief 格式化日期时间
 */
function formatDateTime(timestamp?: number): string {
  if (!timestamp) return '-';
  // 后端返回 epoch 秒，需转毫秒
  const date = new Date(timestamp < 1e12 ? timestamp * 1000 : timestamp);
  return date.toLocaleString();
}

/**
 * @brief 加载日志
 */
async function loadLogs(): Promise<void> {
  loading.value = true;
  try {
    const params: {
      page: number;
      page_size: number;
      keyword?: string;
      status?: number;
      start_date?: string;
      end_date?: string;
    } = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };

    if (filters.value.keyword) {
      params.keyword = filters.value.keyword;
    }
    if (filters.value.status != null && filters.value.status !== '') {
      params.status = Number(filters.value.status);
    }
    if (filters.value.start_date) {
      params.start_date = filters.value.start_date;
    }
    if (filters.value.end_date) {
      params.end_date = filters.value.end_date;
    }

    const response = await listLoginLogs(params);
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: LoginLog[]; total?: number; data?: { list?: LoginLog[]; total?: number } };
    const result = respData.list ? { list: respData.list, total: respData.total || 0 } : respData.data;
    logs.value = result?.list || [];
    pagination.value.rowsNumber = result?.total || 0;
  } catch (error) {
    console.error('【加载日志失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
    throw error;
  } finally {
    loading.value = false;
  }
}

/**
 * @brief 表格请求
 */
interface TableRequestProps {
  pagination: {
    page: number;
    rowsPerPage: number;
    sortBy?: string;
    descending?: boolean;
  };
}

function onTableRequest(props: TableRequestProps): void {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  pagination.value.sortBy = props.pagination.sortBy || 'login_time';
  pagination.value.descending = props.pagination.descending ?? true;
  void loadLogs();
}

/**
 * @brief 搜索
 */
function handleSearch() {
  pagination.value.page = 1;
  void loadLogs();
}

/**
 * @brief 重置
 */
function handleReset() {
  filters.value = {
    keyword: '',
    start_date: '',
    end_date: '',
    status: null,
  };
  pagination.value.page = 1;
  void loadLogs();
}

/**
 * @brief 导出
 */
async function handleExport(): Promise<void> {
  try {
    const params: {
      keyword?: string;
      status?: number;
      start_date?: string;
      end_date?: string;
    } = {};
    if (filters.value.keyword) params.keyword = filters.value.keyword;
    if (filters.value.status !== null && filters.value.status !== '') {
      params.status = Number(filters.value.status);
    }
    if (filters.value.start_date) params.start_date = filters.value.start_date;
    if (filters.value.end_date) params.end_date = filters.value.end_date;

    const response = await exportLoginLogs(params);
    const blob = new Blob([response.data as BlobPart], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    const timestamp = new Date().toISOString().slice(0, 10);
    link.download = `login_logs_${timestamp}.csv`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);

    $q.notify({ type: 'positive', message: t('common.success') });
  } catch (error) {
    console.error('【导出失败】', error);
    // 回退到前端导出
    void exportToCSV(logs.value, [], 'login-logs.csv');
  }
}

// ============ 生命周期 ============
onMounted(() => {
  void loadLogs();
});
</script>

<style scoped>
/* 暗色主题适配 */
.body--dark .q-card {
  background: #1e1e1e;
  border-color: #2d2d2d;
}

.body--dark .q-table {
  background: #1e1e1e;
  color: #ffffff;
}

code {
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
}

.body--dark code {
  background: rgba(255, 255, 255, 0.1);
}
</style>
