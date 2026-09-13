/**
 * @file useOperationLog.ts
 * @description 操作日志页面业务逻辑 composable
 */

import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { listOperationLogs, exportOperationLogs, type OperationLog } from '@/api/log';
import { useExport } from '@/composables/useExport';

export interface OperationLogFilters {
  keyword?: string;
  start_date?: string;
  end_date?: string;
  user?: string;
  module?: string;
  action?: string;
}

export function useOperationLog() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const { exportToCSV } = useExport();
const exportData = exportToCSV;

  // ============ 状态 ============
  const loading = ref(false);
  const logs = ref<OperationLog[]>([]);
  const showDetailDialog = ref(false);
  const currentLog = ref<OperationLog | null>(null);

  const filters = ref<OperationLogFilters>({
    keyword: '',
    start_date: '',
    end_date: '',
    user: '',
    module: '',
    action: '',
  });

  const pagination = ref({
    page: 1,
    rowsPerPage: 10,
    rowsNumber: 0,
    sortBy: 'created_at',
    descending: true,
  });

  // ============ 计算属性 ============

  const userOptions = computed(() => [
    { label: $t('log.allUsers'), value: '' },
    { label: 'admin', value: 'admin' },
    { label: 'system', value: 'system' },
  ]);

  const moduleOptions = computed(() => [
    { label: $t('log.allModules'), value: '' },
    { label: $t('log.moduleUser'), value: 'user' },
    { label: $t('log.moduleRole'), value: 'role' },
    { label: $t('log.moduleDepartment'), value: 'department' },
    { label: $t('log.moduleSystem'), value: 'system' },
  ]);

  const actionOptions = computed(() => [
    { label: $t('log.allActions'), value: '' },
    { label: $t('log.actionCreate'), value: 'create' },
    { label: $t('log.actionUpdate'), value: 'update' },
    { label: $t('log.actionDelete'), value: 'delete' },
    { label: $t('log.actionLogin'), value: 'login' },
    { label: $t('log.actionLogout'), value: 'logout' },
  ]);

  const columns = computed(() => [
    { name: 'id', label: $t('log.id'), field: 'id', align: 'left' as const, sortable: true },
    { name: 'operator', label: $t('log.operator'), field: 'operator', align: 'left' as const, sortable: true },
    { name: 'module', label: $t('log.module'), field: 'module', align: 'center' as const, sortable: true },
    { name: 'action', label: $t('log.action'), field: 'action', align: 'center' as const, sortable: true },
    { name: 'resource', label: $t('log.resource'), field: 'resource', align: 'left' as const, sortable: false },
    { name: 'ip', label: $t('log.ip'), field: 'ip', align: 'left' as const, sortable: false },
    { name: 'detail', label: $t('log.detail'), field: 'detail', align: 'center' as const },
    { name: 'created_at', label: $t('log.time'), field: 'created_at', align: 'left' as const, sortable: true },
  ]);

  // ============ 方法 ============

  function getActionColor(action: string): string {
    const colors: Record<string, string> = {
      create: 'green',
      update: 'orange',
      delete: 'red',
      login: 'blue',
      logout: 'grey',
      export: 'purple',
    };
    return colors[action] || 'grey';
  }

  function formatDateTime(timestamp?: number | string): string {
    if (!timestamp) return '-';
    const ts = typeof timestamp === 'string' ? Number(timestamp) : timestamp;
    // 后端返回 epoch 秒，需转毫秒
    const date = new Date(ts < 1e12 ? ts * 1000 : ts);
    return date.toLocaleString();
  }

  function formatJson(obj?: Record<string, unknown> | string): string {
    if (!obj) return '-';
    try {
      if (typeof obj === 'string') {
        return JSON.stringify(JSON.parse(obj), null, 2);
      }
      return JSON.stringify(obj, null, 2);
    } catch {
      return typeof obj === 'object' ? JSON.stringify(obj) : String(obj);
    }
  }

  async function loadLogs(): Promise<void> {
    loading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        keyword?: string;
        operator?: string;
        module?: string;
        action?: string;
        start_date?: string;
        end_date?: string;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.rowsPerPage,
      };

      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.user) params.operator = filters.value.user;
      if (filters.value.module) params.module = filters.value.module;
      if (filters.value.action) params.action = filters.value.action;
      if (filters.value.start_date) params.start_date = filters.value.start_date;
      if (filters.value.end_date) params.end_date = filters.value.end_date;

      const response = await listOperationLogs(params);
      const respData = response as { list?: OperationLog[]; total?: number; data?: { list?: OperationLog[]; total?: number } };
      const result = respData.list ? { list: respData.list, total: respData.total || 0 } : respData.data;
      logs.value = result?.list || [];
      pagination.value.rowsNumber = result?.total || 0;
    } catch (error) {
      console.error('【加载日志失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    } finally {
      loading.value = false;
    }
  }

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
    pagination.value.sortBy = props.pagination.sortBy || 'created_at';
    pagination.value.descending = props.pagination.descending ?? true;
    void loadLogs();
  }

  function handleSearch() {
    pagination.value.page = 1;
    void loadLogs();
  }

  function handleReset() {
    filters.value = {
      keyword: '',
      start_date: '',
      end_date: '',
      user: '',
      module: '',
      action: '',
    };
    pagination.value.page = 1;
    void loadLogs();
  }

  function viewDetail(log: OperationLog) {
    currentLog.value = log;
    showDetailDialog.value = true;
  }

  async function handleExport(): Promise<void> {
    try {
      const params: {
        keyword?: string;
        operator?: string;
        module?: string;
        action?: string;
        start_date?: string;
        end_date?: string;
      } = {};
      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.user) params.operator = filters.value.user;
      if (filters.value.module) params.module = filters.value.module;
      if (filters.value.action) params.action = filters.value.action;
      if (filters.value.start_date) params.start_date = filters.value.start_date;
      if (filters.value.end_date) params.end_date = filters.value.end_date;

      const response = await exportOperationLogs(params);
      const blob = new Blob([response.data as BlobPart], { type: 'text/csv' });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      const timestamp = new Date().toISOString().slice(0, 10);
      link.download = `operation_logs_${timestamp}.csv`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch (error) {
      console.error('【导出失败】', error);
      void exportData(logs.value, [], 'operation_logs.csv');
    }
  }

  // ============ 生命周期 ============
  onMounted(() => {
    void loadLogs();
  });

  return {
    // 状态
    loading,
    logs,
    showDetailDialog,
    currentLog,
    filters,
    pagination,
    // 计算属性
    userOptions,
    moduleOptions,
    actionOptions,
    columns,
    // 方法
    getActionColor,
    formatDateTime,
    formatJson,
    loadLogs,
    onTableRequest,
    handleSearch,
    handleReset,
    viewDetail,
    handleExport,
  };
}
