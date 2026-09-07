/**
 * @file useApiStatistics.ts
 * @description ApiStatistics 业务逻辑 composable
 */

import { ref, computed, onMounted, watch } from 'vue';
import { useQuasar } from 'quasar';
import type { QTableProps } from 'quasar';
import type { ApiCallStatistics, ApiEndpointStatistics, ApiCategoryStatistics, ApiPerformanceBaseline, ApiTrendPoint, ApiResponseTimeDistribution } from '@/types/apiGovernance';
import { getApiCallStatistics, getApiEndpointStatistics, getApiCategoryStatistics, getApiPerformanceBaseline, getApiTrend, getApiResponseDistribution } from '@/api/apiGovernance';
import { logger } from '@/utils/logger';

export function useApiStatistics() {
  const $q = useQuasar();

  const METHOD_COLORS: Record<string, string> = { GET: 'info', POST: 'positive', PUT: 'warning', DELETE: 'negative', PATCH: 'secondary' };

  // ============ 状态 ============

  const timeRange = ref('24h');
  const loading = ref(false);

  const totalStats = ref<ApiCallStatistics>({
    totalCalls: 0, successCalls: 0, failedCalls: 0, errorRate: 0, avg_response_time: 0,
    p50ResponseTime: 0, p90ResponseTime: 0, p95_response_time: 0, p99ResponseTime: 0,
    max_response_time: 0, min_response_time: 0, totalDataSize: 0, qps: 0,
    timeRange: { start: Date.now() - 24 * 60 * 60 * 1000, end: Date.now() },
  });

  const endpointStats = ref<ApiEndpointStatistics[]>([]);
  const categoryStats = ref<ApiCategoryStatistics[]>([]);
  const baselineStats = ref<ApiPerformanceBaseline[]>([]);
  const trendData = ref<ApiTrendPoint[]>([]);
  const responseDistribution = ref<ApiResponseTimeDistribution[]>([]);

  // ============ 计算属性 ============

  const timeRangeLabel = computed(() => {
    const labels: Record<string, string> = { '1h': '最近 1 小时', '24h': '最近 24 小时', '7d': '最近 7 天', '30d': '最近 30 天' };
    return labels[timeRange.value] || timeRange.value;
  });

  const max_response_time = computed(() => {
    const max = Math.max(...trendData.value.map((p) => Math.max(p.avg_response_time, p.p95_response_time)));
    return max || 500;
  });

  const baselineAlerts = computed(() => baselineStats.value.filter((b) => b.status !== 'normal'));

  // ============ 表格列定义 ============

  const endpointColumns: QTableProps['columns'] = [
    { name: 'endpoint', label: '端点', field: 'path', align: 'left' },
    { name: 'callCount', label: '调用次数', field: 'callCount', align: 'center', sortable: true },
    { name: 'successCount', label: '成功', field: 'successCount', align: 'center' },
    { name: 'failedCount', label: '失败', field: 'failedCount', align: 'center' },
    { name: 'errorRate', label: '错误率', field: 'errorRate', align: 'center', sortable: true },
    { name: 'response_time', label: '平均响应', field: 'avg_response_time', align: 'center', sortable: true },
    { name: 'p95_response_time', label: 'P95', field: 'p95_response_time', align: 'center' },
    { name: 'lastCalledAt', label: '最后调用', field: 'lastCalledAt', align: 'center', sortable: true },
  ];

  // ============ 工具函数 ============

  function getMethodColor(method: string): string { return METHOD_COLORS[method] || 'grey'; }

  function getDistributionColor(max: number): string {
    if (max <= 100) return 'positive';
    if (max <= 300) return 'info';
    if (max <= 500) return 'warning';
    if (max <= 1000) return 'negative';
    return 'grey';
  }

  function getBaselineStatusColor(status: string): string {
    const colors: Record<string, string> = { normal: 'positive', degraded: 'warning', critical: 'negative' };
    return colors[status] || 'grey';
  }

  function formatTime(timestamp: number): string {
    const diff = Date.now() - timestamp;
    if (diff < 60000) return `${Math.floor(diff / 1000)}秒前`;
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    return new Date(timestamp).toLocaleTimeString('zh-CN');
  }

  function getStartTime(): string {
    const now = Date.now();
    const durations: Record<string, number> = { '1h': 60 * 60 * 1000, '24h': 24 * 60 * 60 * 1000, '7d': 7 * 24 * 60 * 60 * 1000, '30d': 30 * 24 * 60 * 60 * 1000 };
    const duration = durations[timeRange.value] ?? 24 * 60 * 60 * 1000;
    return new Date(now - duration).toISOString();
  }

  // ============ 数据加载 ============

  async function loadStatistics() {
    loading.value = true;
    const start_time = getStartTime();
    const end_time = new Date().toISOString();

    try {
      const [statsRes, endpointsRes, categoriesRes, baselineRes, trendRes, distributionRes] = await Promise.all([
        getApiCallStatistics({ start_time, end_time }).catch((error) => {
          logger.warn('【API调用统计获取失败】', error);
          return { data: null };
        }),
        getApiEndpointStatistics({ start_time, end_time }).catch((error) => {
          logger.warn('【API端点统计获取失败】', error);
          return { data: [] };
        }),
        getApiCategoryStatistics({ start_time, end_time }).catch((error) => {
          logger.warn('【API分类统计获取失败】', error);
          return { data: [] };
        }),
        getApiPerformanceBaseline({ start_time, end_time }).catch((error) => {
          logger.warn('【API性能基线获取失败】', error);
          return { data: [] };
        }),
        getApiTrend({ start_time, end_time, granularity: timeRange.value === '1h' ? 'hour' : 'day' }).catch((error) => {
          logger.warn('【API趋势获取失败】', error);
          return { data: [] };
        }),
        getApiResponseDistribution({ start_time, end_time }).catch((error) => {
          logger.warn('【API响应分布获取失败】', error);
          return { data: [] };
        }),
      ]);

      if (statsRes.data) totalStats.value = statsRes.data;
      if (Array.isArray(endpointsRes.data)) endpointStats.value = endpointsRes.data;
      if (Array.isArray(categoriesRes.data)) categoryStats.value = categoriesRes.data;
      if (Array.isArray(baselineRes.data)) baselineStats.value = baselineRes.data;
      if (Array.isArray(trendRes.data)) trendData.value = trendRes.data;
      if (Array.isArray(distributionRes.data)) responseDistribution.value = distributionRes.data;
    } catch (error) {
      logger.error('加载统计数据失败:', error);
      $q.notify({ type: 'negative', message: '加载统计数据失败' });
    } finally {
      loading.value = false;
    }
  }

  async function handleRefresh() {
    await loadStatistics();
    $q.notify({ type: 'positive', message: '统计数据已刷新' });
  }

  // ============ 生命周期 ============

  onMounted(() => { void loadStatistics(); });

  watch(timeRange, () => { void loadStatistics(); });

  // ============ 返回 ============

  return {
    timeRange, loading, totalStats, endpointStats, categoryStats, baselineStats, trendData, responseDistribution,
    timeRangeLabel, max_response_time, baselineAlerts, endpointColumns,
    getMethodColor, getDistributionColor, getBaselineStatusColor, formatTime,
    loadStatistics, handleRefresh,
  };
}
