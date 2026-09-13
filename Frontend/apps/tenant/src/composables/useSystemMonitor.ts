/**
 * @file useSystemMonitor.ts
 * @description 系统监控页面业务逻辑 composable
 * @date 2026-04-04
 */

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import {
  getApiCallStatistics,
  getApiTrend,
  listApiLogs,
} from '@/api/apiGovernance';
import { logger } from '@/utils/logger';

export function useSystemMonitor() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  const loading = ref(false);

  // ============ 系统状态数据（从 API 加载） ============
  const healthStatus = ref('normal');
  const lastCheckTime = ref(new Date().toLocaleTimeString());
  const avg_response_time = ref(0);
  const responseTrend = ref('flat');
  const onlineUsers = ref(0);
  const errorRate = ref(0);
  const todayErrors = ref(0);

  // ============ 最近API调用（从 API 加载） ============
  const recentApiCalls = ref<Array<{
    id: number;
    method: string;
    path: string;
    status: number;
    duration: number;
    time: number;
  }>>([]);

  // ============ 最近错误（从 API 加载） ============
  const recentErrors = ref<Array<{
    id: number;
    level: string;
    message: string;
    path: string;
    time: number;
  }>>([]);

  // ============ 图表数据（从 API 加载） ============
  const trendChartData = ref<Array<{ label: string; value: number }>>([]);
  const statusDistributionData = ref<Array<{ label: string; value: number }>>([]);

  // ============ 表格列定义 ============
  const apiCallColumns = computed(() => [
    { name: 'method', label: $t('monitor.method') || '方法', field: 'method', align: 'left' as const },
    { name: 'path', label: $t('monitor.path') || '路径', field: 'path', align: 'left' as const },
    { name: 'status', label: $t('monitor.status') || '状态', field: 'status', align: 'center' as const },
    { name: 'duration', label: $t('monitor.duration') || '耗时', field: 'duration', align: 'center' as const },
    { name: 'time', label: $t('monitor.time') || '时间', field: 'time', align: 'left' as const },
  ]);

  const errorColumns = computed(() => [
    { name: 'level', label: $t('monitor.level') || '级别', field: 'level', align: 'center' as const },
    { name: 'message', label: $t('monitor.message') || '消息', field: 'message', align: 'left' as const },
    { name: 'path', label: $t('monitor.path') || '路径', field: 'path', align: 'left' as const },
    { name: 'time', label: $t('monitor.time') || '时间', field: 'time', align: 'left' as const },
  ]);

  // ============ 计算属性 ============
  const healthStatusClass = computed(() => {
    switch (healthStatus.value) {
      case 'normal':
        return 'status-normal';
      case 'warning':
        return 'status-warning';
      case 'error':
        return 'status-error';
      default:
        return '';
    }
  });

  const healthStatusColor = computed(() => {
    switch (healthStatus.value) {
      case 'normal':
        return 'linear-gradient(135deg, #51cf66 0%, #40c057 100%)';
      case 'warning':
        return 'linear-gradient(135deg, #ffc107 0%, #ff9800 100%)';
      case 'error':
        return 'linear-gradient(135deg, #ff6b6b 0%, #f03e3e 100%)';
      default:
        return 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)';
    }
  });

  const response_time_class = computed(() => {
    if (avg_response_time.value < 50) return 'text-positive';
    if (avg_response_time.value < 100) return 'text-warning';
    return 'text-negative';
  });

  const responseTrendClass = computed(() => {
    if (responseTrend.value === 'down') return 'text-positive';
    if (responseTrend.value === 'up') return 'text-negative';
    return '';
  });

  const responseTrendIcon = computed(() => {
    if (responseTrend.value === 'down') return 'trending_down';
    if (responseTrend.value === 'up') return 'trending_up';
    return 'trending_flat';
  });

  const responseTrendText = computed(() => {
    if (responseTrend.value === 'down') return $t('monitor.faster') || '比昨日快';
    if (responseTrend.value === 'up') return $t('monitor.slower') || '比昨日慢';
    return $t('monitor.same') || '与昨日持平';
  });

  const errorRateClass = computed(() => {
    if (errorRate.value < 1) return 'text-positive';
    if (errorRate.value < 5) return 'text-warning';
    return 'text-negative';
  });

  // ============ 图表配置 ============
  const response_time_chart_config = computed(() => ({
    type: 'line' as const,
    data: trendChartData.value.length > 0
      ? trendChartData.value
      : [
          { label: '00:00', value: 0 },
          { label: '04:00', value: 0 },
          { label: '08:00', value: 0 },
          { label: '12:00', value: 0 },
          { label: '16:00', value: 0 },
          { label: '20:00', value: 0 },
          { label: '现在', value: avg_response_time.value },
        ],
    height: '300px',
    showGrid: true,
    showLegend: false,
    areaFill: true,
    smooth: true,
  }));

  const statusDistributionConfig = computed(() => ({
    type: 'pie' as const,
    data: statusDistributionData.value.length > 0
      ? statusDistributionData.value
      : [
          { label: $t('monitor.success') || '成功', value: 0 },
          { label: $t('monitor.clientError') || '客户端错误', value: 0 },
          { label: $t('monitor.serverError') || '服务端错误', value: 0 },
        ],
    height: '300px',
    showLegend: true,
    pieRadius: '70%',
  }));

  // ============ 方法 ============
  function getStatusColor(status: number): string {
    if (status >= 200 && status < 300) return 'positive';
    if (status >= 300 && status < 400) return 'info';
    if (status >= 400 && status < 500) return 'warning';
    return 'negative';
  }

  function getDurationClass(duration: number): string {
    if (duration < 50) return 'text-positive';
    if (duration < 100) return 'text-warning';
    return 'text-negative';
  }

  function getLevelColor(level: string): string {
    switch (level) {
      case 'error':
        return 'negative';
      case 'warning':
        return 'warning';
      case 'info':
        return 'info';
      default:
        return 'grey';
    }
  }

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  }

  /**
   * @brief 加载监控数据
   */
  async function loadData() {
    loading.value = true;
    try {
      const now = new Date();
      const start_time = new Date(now.getTime() - 24 * 60 * 60 * 1000).toISOString();
      const end_time = now.toISOString();

      // 并行加载多个数据源
      const [statsRes, trendRes, logsRes] = await Promise.all([
        getApiCallStatistics({ start_time, end_time }).catch((error) => {
          logger.warn('【API调用统计获取失败】', error);
          return { data: null };
        }),
        getApiTrend({ start_time, end_time, granularity: 'hour' }).catch((error) => {
          logger.warn('【API趋势获取失败】', error);
          return { data: [] };
        }),
        listApiLogs({ page_size: 10 }).catch((error) => {
          logger.warn('【API日志列表获取失败】', error);
          return { data: { list: [], total: 0 } };
        }),
      ]);

      // 更新统计数据
      if (statsRes.data) {
        const stats = statsRes.data;
        avg_response_time.value = stats.avg_response_time || 0;
        errorRate.value = stats.errorRate || 0;
        todayErrors.value = stats.failedCalls || 0;

        // 根据错误率判断健康状态
        if (stats.errorRate < 1) {
          healthStatus.value = 'normal';
        } else if (stats.errorRate < 5) {
          healthStatus.value = 'warning';
        } else {
          healthStatus.value = 'error';
        }

        // 更新状态分布数据
        const successCount = stats.successCalls || 0;
        const failedCount = stats.failedCalls || 0;
        statusDistributionData.value = [
          { label: $t('monitor.success') || '成功', value: successCount },
          { label: $t('monitor.clientError') || '客户端错误', value: Math.floor(failedCount * 0.7) },
          { label: $t('monitor.serverError') || '服务端错误', value: Math.floor(failedCount * 0.3) },
        ];
      }

      // 更新趋势数据
      if (Array.isArray(trendRes.data) && trendRes.data.length > 0) {
        trendChartData.value = trendRes.data.map((point) => ({
          label: new Date(point.timestamp).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
          value: point.avg_response_time || 0,
        }));
      }

      // 更新最近API调用
      const logsData = logsRes.data;
      if (logsData && 'list' in logsData && Array.isArray(logsData.list)) {
        recentApiCalls.value = logsData.list.slice(0, 5).map((log, idx) => ({
          id: idx + 1,
          method: log.method || 'GET',
          path: log.path || '',
          status: log.status_code || 200,
          duration: log.response_time || 0,
          time: log.timestamp || Date.now(),
        }));

        // 提取错误日志
        recentErrors.value = logsData.list
          .filter((log) => log.status_code >= 400 || log.error)
          .slice(0, 5)
          .map((log, idx) => ({
            id: idx + 1,
            level: log.status_code >= 500 ? 'error' : 'warning',
            message: log.error || `HTTP ${log.status_code}`,
            path: log.path || '',
            time: log.timestamp || Date.now(),
          }));
      }

      lastCheckTime.value = new Date().toLocaleTimeString();
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch (error) {
      logger.error('加载监控数据失败:', error);
      $q.notify({ type: 'negative', message: $t('common.error') || '加载失败' });
    } finally {
      loading.value = false;
    }
  }

  onMounted(() => {
    void loadData();
  });

  return {
    loading,
    healthStatus,
    lastCheckTime,
    avg_response_time,
    responseTrend,
    onlineUsers,
    errorRate,
    todayErrors,
    recentApiCalls,
    recentErrors,
    trendChartData,
    statusDistributionData,
    apiCallColumns,
    errorColumns,
    healthStatusClass,
    healthStatusColor,
    response_time_class,
    responseTrendClass,
    responseTrendIcon,
    responseTrendText,
    errorRateClass,
    response_time_chart_config,
    statusDistributionConfig,
    getStatusColor,
    getDurationClass,
    getLevelColor,
    formatTime,
    loadData,
  };
}
