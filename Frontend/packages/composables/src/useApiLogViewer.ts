/**
 * @file useApiLogViewer.ts
 * @description ApiLogViewer 业务逻辑 composable
 */

import { ref, computed, reactive, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import type { QTableProps } from 'quasar';
import type { ApiLogEntry, ApiCallStatistics, ApiLogFilter } from '@/types/apiGovernance';
import { listApiLogs, getApiCallStatistics } from '@/api/apiGovernance';
import { logger } from '@/utils/logger';

export function useApiLogViewer() {
  const $q = useQuasar();

  // ============ 常量 ============

  const methodOptions = [
    { label: 'GET', value: 'GET' },
    { label: 'POST', value: 'POST' },
    { label: 'PUT', value: 'PUT' },
    { label: 'DELETE', value: 'DELETE' },
    { label: 'PATCH', value: 'PATCH' },
  ];

  const statusOptions = [
    { label: '2xx 成功', value: 2 },
    { label: '4xx 客户端错误', value: 4 },
    { label: '5xx 服务器错误', value: 5 },
  ];

  // ============ 状态 ============

  const loading = ref(false);
  const showDetailDialog = ref(false);
  const selectedLog = ref<ApiLogEntry | null>(null);

  const filter = reactive({
    pathKeyword: '',
    errorsOnly: false,
  }) as ApiLogFilter;

  const stats = ref<ApiCallStatistics>({
    totalCalls: 0,
    successCalls: 0,
    failedCalls: 0,
    errorRate: 0,
    avg_response_time: 0,
    p50ResponseTime: 0,
    p90ResponseTime: 0,
    p95_response_time: 0,
    p99ResponseTime: 0,
    max_response_time: 0,
    min_response_time: 0,
    totalDataSize: 0,
    qps: 0,
    timeRange: { start: Date.now() - 24 * 60 * 60 * 1000, end: Date.now() },
  });

  const logs = ref<ApiLogEntry[]>([]);
  const totalCount = ref(0);

  // ============ 计算属性 ============

  const filteredLogs = computed(() =>
    logs.value.filter((log) => {
      if (filter.method && log.method !== filter.method) return false;
      if (filter.pathKeyword && !log.path.toLowerCase().includes(filter.pathKeyword.toLowerCase())) return false;
      if (filter.status_code && Math.floor(log.status_code / 100) !== filter.status_code) return false;
      if (filter.min_response_time && log.response_time < filter.min_response_time) return false;
      if (filter.max_response_time && log.response_time > filter.max_response_time) return false;
      if (filter.errorsOnly && log.status_code < 400) return false;
      if (filter.start_time && new Date(filter.start_time).getTime() > log.timestamp) return false;
      if (filter.end_time && new Date(filter.end_time).getTime() < log.timestamp) return false;
      return true;
    }),
  );

  const errorRateClass = computed(() => {
    if (stats.value.errorRate > 10) return 'text-negative';
    if (stats.value.errorRate > 5) return 'text-warning';
    return 'text-positive';
  });

  // ============ 表格列定义 ============

  const columns: QTableProps['columns'] = [
    { name: 'method', label: '方法', field: 'method', align: 'center', style: 'width: 80px' },
    { name: 'path', label: '路径', field: 'path', align: 'left' },
    { name: 'status_code', label: '状态', field: 'status_code', align: 'center', style: 'width: 80px' },
    { name: 'response_time', label: '响应', field: 'response_time', align: 'center', style: 'width: 100px', sortable: true },
    { name: 'user', label: '用户', field: 'username', align: 'center', style: 'width: 120px' },
    { name: 'timestamp', label: '时间', field: 'timestamp', align: 'center', style: 'width: 180px', sortable: true },
  ];

  // ============ 工具函数 ============

  function getMethodColor(method: string): string {
    const colors: Record<string, string> = { GET: 'info', POST: 'positive', PUT: 'warning', DELETE: 'negative', PATCH: 'secondary' };
    return colors[method] || 'grey';
  }

  function getStatusColor(status_code: number): string {
    if (status_code >= 200 && status_code < 300) return 'positive';
    if (status_code >= 400 && status_code < 500) return 'warning';
    if (status_code >= 500) return 'negative';
    return 'grey';
  }

  function getResponseTimeClass(time: number): string {
    if (time > 2000) return 'text-negative text-weight-bold';
    if (time > 1000) return 'text-warning';
    return 'text-positive';
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleString('zh-CN');
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    let i = 0;
    let b = bytes;
    while (b >= 1024 && i < units.length - 1) { b /= 1024; i++; }
    return `${b.toFixed(1)} ${units[i]}`;
  }

  function formatJson(obj: Record<string, unknown> | string | undefined): string {
    if (!obj) return '';
    if (typeof obj === 'string') { try { return JSON.stringify(JSON.parse(obj), null, 2); } catch { return obj; } }
    return JSON.stringify(obj, null, 2);
  }

  // ============ 数据加载 ============

  async function loadLogs() {
    loading.value = true;
    try {
      const response = await listApiLogs({
        page: 1, page_size: 100,
        ...(filter.pathKeyword ? { pathKeyword: filter.pathKeyword } : {}),
        ...(filter.method ? { method: filter.method } : {}),
        ...(filter.status_code !== undefined ? { status_code: filter.status_code } : {}),
        ...(filter.min_response_time !== undefined ? { min_response_time: filter.min_response_time } : {}),
        ...(filter.max_response_time !== undefined ? { max_response_time: filter.max_response_time } : {}),
        ...(filter.start_time ? { start_time: new Date(filter.start_time).toISOString() } : {}),
        ...(filter.end_time ? { end_time: new Date(filter.end_time).toISOString() } : {}),
        ...(filter.errorsOnly !== undefined ? { errorsOnly: filter.errorsOnly } : {}),
      });
      const respData = response.data as { list?: ApiLogEntry[]; total?: number };
      if (respData && 'list' in respData) {
        logs.value = respData.list || [];
        totalCount.value = respData.total || 0;
      }
    } catch (error) {
      logger.error('加载日志失败:', error);
      $q.notify({ type: 'negative', message: '加载日志失败' });
    } finally {
      loading.value = false;
    }
  }

  async function loadStatistics() {
    try {
      const response = await getApiCallStatistics({
        ...(filter.start_time ? { start_time: new Date(filter.start_time).toISOString() } : {}),
        ...(filter.end_time ? { end_time: new Date(filter.end_time).toISOString() } : {}),
      });
      if (response.data) stats.value = response.data;
    } catch (error) {
      logger.error('加载统计数据失败:', error);
    }
  }

  async function handleRefresh() {
    await Promise.all([loadLogs(), loadStatistics()]);
    $q.notify({ type: 'positive', message: '日志已刷新' });
  }

  async function handleFilter() {
    await Promise.all([loadLogs(), loadStatistics()]);
    $q.notify({ type: 'positive', message: `筛选完成，共 ${totalCount.value} 条记录` });
  }

  function handleClearFilter() {
    const f = filter as Record<string, unknown>;
    f.method = undefined;
    filter.pathKeyword = '';
    f.status_code = undefined;
    f.min_response_time = undefined;
    f.max_response_time = undefined;
    f.start_time = undefined;
    f.end_time = undefined;
    filter.errorsOnly = false;
  }

  function handleRowClick(_evt: Event, row: ApiLogEntry) {
    selectedLog.value = row;
    showDetailDialog.value = true;
  }

  function handleExport() {
    const data = filteredLogs.value.map((log) => ({
      时间: formatTime(log.timestamp), 方法: log.method, 路径: log.path,
      状态: log.status_code, 响应时间: `${log.response_time}ms`, 用户: log.username || '-',
    }));

    const csv = [Object.keys(data[0] || {}).join(','), ...data.map((row) => Object.values(row).join(','))].join('\n');
    const blob = new Blob(['\ufeff' + csv], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `api_logs_${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    $q.notify({ type: 'positive', message: '导出成功' });
  }

  // ============ 生命周期 ============

  onMounted(() => { void Promise.all([loadLogs(), loadStatistics()]); });

  // ============ 返回 ============

  return {
    methodOptions, statusOptions,
    loading, showDetailDialog, selectedLog, filter, stats, logs, totalCount,
    filteredLogs, errorRateClass, columns,
    getMethodColor, getStatusColor, getResponseTimeClass, formatTime, formatBytes, formatJson,
    loadLogs, loadStatistics, handleRefresh, handleFilter, handleClearFilter, handleRowClick, handleExport,
  };
}
