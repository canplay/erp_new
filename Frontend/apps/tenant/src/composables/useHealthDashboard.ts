/**
 * @file useHealthDashboard.ts
 * @description 系统健康监控逻辑 — 从 HealthDashboard.vue 拆分
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { usePerformanceMonitor } from '@erp-new-frontend-monorepo/composables/src/usePerformanceMonitor';;
import type { FrontendError, PerformanceAlert } from '@/types/monitor';

export function useHealthDashboard() {
  const $q = useQuasar();

  const {
    performanceMetrics,
    wsMetrics,
    apiCallRecords,
    errorRecords,
    resourceUsage,
    healthChecks,
    isMonitoring,
    apiStats,
    errorStats,
    startMonitoring,
    stopMonitoring,
    resolveError,
  clearErrors,
  clearApiRecords,
} = usePerformanceMonitor();

  // Alerts
  const alerts = ref<PerformanceAlert[]>([]);
  const activeAlertsCount = computed(() => alerts.value.filter((a) => a.status === 'active').length);

  // Tab
  const activeTab = ref('performance');
  const showErrorDialog = ref(false);
  const selectedError = ref<FrontendError | null>(null);

  // Quality levels — static data, not reactive
  const qualityLevels = [
    { label: '优秀', range: '90-100', value: 90, color: 'positive', icon: 'sentiment_very_satisfied' },
    { label: '良好', range: '70-89', value: 70, color: 'info', icon: 'sentiment_satisfied' },
    { label: '一般', range: '50-69', value: 50, color: 'warning', icon: 'sentiment_neutral' },
    { label: '较差', range: '20-49', value: 20, color: 'negative', icon: 'sentiment_dissatisfied' },
    { label: '断开', range: '0-19', value: 0, color: 'grey', icon: 'signal_wifi_off' },
  ] as const;

  // Computed - Metrics
  const metrics = computed(() => performanceMetrics.value || {
    FCP: 0, LCP: 0, CLS: 0, FID: 0,
    pageLoadTime: 0, DOMContentLoaded: 0, loadComplete: 0,
  });

  const activeAlerts = computed(() => alerts.value.filter((a) => a.status === 'active'));

  // Computed - Colors
  const overallHealthColor = computed(() => {
    const c = healthChecks.value.filter((h) => h.status === 'critical').length;
    const w = healthChecks.value.filter((h) => h.status === 'warning').length;
    if (c > 0) return 'negative';
    if (w > 0) return 'warning';
    return 'positive';
  });

  const overallHealthStatus = computed(() => {
    const c = healthChecks.value.filter((h) => h.status === 'critical').length;
    const w = healthChecks.value.filter((h) => h.status === 'warning').length;
    if (c > 0) return `${c} 项异常`;
    if (w > 0) return `${w} 项警告`;
    return '全部正常';
  });

  const apiStatusColor = computed(() => {
    if (apiStats.value.errorRate > 10) return 'negative';
    if (apiStats.value.errorRate > 5) return 'warning';
    return 'positive';
  });

  const errorRateColor = computed(() => apiStatusColor.value);
  const memoryUsageColor = computed(() => {
    const usage = resourceUsage.value?.memoryUsage || 0;
    if (usage > 80) return 'negative';
    if (usage > 60) return 'warning';
    return 'positive';
  });

  const lcpColorClass = computed(() => {
    if (metrics.value.LCP > 4000) return 'text-negative';
    if (metrics.value.LCP > 2500) return 'text-warning';
    return 'text-primary';
  });

  const clsColorClass = computed(() => {
    if (metrics.value.CLS > 0.25) return 'text-negative';
    if (metrics.value.CLS > 0.1) return 'text-warning';
    return 'text-primary';
  });

  // Computed - WebSocket
  const connectionDuration = computed(() => {
    if (!wsMetrics.value.connectedAt) return '-';
    return formatDuration(Date.now() - wsMetrics.value.connectedAt);
  });

  const lastHeartbeatAgo = computed(() => {
    if (!wsMetrics.value.lastHeartbeatAt) return '-';
    return formatDuration(Date.now() - wsMetrics.value.lastHeartbeatAt) + ' 前';
  });

  const wsStatusColor = computed(() => {
    const colors: Record<string, string> = { connected: 'positive', connecting: 'warning', reconnecting: 'warning', disconnected: 'grey', error: 'negative' };
    return colors[wsMetrics.value.status] || 'grey';
  });

  const wsStatusLabel = computed(() => {
    const labels: Record<string, string> = { connected: '已连接', connecting: '连接中', reconnecting: '重新连接中', disconnected: '已断开', error: '连接错误' };
    return labels[wsMetrics.value.status] || '未知';
  });

  const qualityScoreColor = computed(() => {
    const s = wsMetrics.value.qualityScore;
    if (s >= 90) return 'positive';
    if (s >= 70) return 'info';
    if (s >= 50) return 'warning';
    return 'negative';
  });

  const qualityLevelLabel = computed(() => {
    const s = wsMetrics.value.qualityScore;
    if (s >= 90) return '优秀';
    if (s >= 70) return '良好';
    if (s >= 50) return '一般';
    if (s >= 20) return '较差';
    return '断开';
  });

  // Utility functions
  function formatBytes(bytes: number | undefined): string {
    if (!bytes) return '0 B';
    let b = bytes;
    const units = ['B', 'KB', 'MB', 'GB'];
    let i = 0;
    while (b >= 1024 && i < units.length - 1) { b /= 1024; i++; }
    return `${b.toFixed(1)} ${units[i]}`;
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleString('zh-CN');
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${Math.floor(ms / 1000)}s`;
    if (ms < 3600000) return `${Math.floor(ms / 60000)}m`;
    return `${Math.floor(ms / 3600000)}h`;
  }

  function getMethodColor(method: string): string {
    const colors: Record<string, string> = { GET: 'info', POST: 'positive', PUT: 'warning', DELETE: 'negative', PATCH: 'secondary' };
    return colors[method] || 'grey';
  }

  function getHealthIcon(status: string): string {
    const icons: Record<string, string> = { healthy: 'check_circle', warning: 'warning', critical: 'error', unknown: 'help' };
    return icons[status] || 'help';
  }

  function getHealthColor(status: string): string {
    const colors: Record<string, string> = { healthy: 'positive', warning: 'warning', critical: 'negative', unknown: 'grey' };
    return colors[status] || 'grey';
  }

  function getAlertIcon(level: string): string {
    const icons: Record<string, string> = { info: 'info', warning: 'warning', critical: 'error' };
    return icons[level] || 'info';
  }

  function getAlertColor(level: string): string {
    const colors: Record<string, string> = { info: 'info', warning: 'warning', critical: 'negative' };
    return colors[level] || 'grey';
  }

  function resolveAlert(alertId: string) {
    const alert = alerts.value.find((a) => a.id === alertId);
    if (alert) alert.status = 'resolved';
  }

  function handleRefresh() {
    $q.notify({ type: 'positive', message: '数据已刷新' });
  }

  function showErrorDetail(error: FrontendError) {
    selectedError.value = error;
    showErrorDialog.value = true;
  }

  function handleResolveAll() {
    errorRecords.value.forEach((e) => { if (!e.resolved) e.resolved = true; });
    $q.notify({ type: 'positive', message: '所有错误已标记为已处理' });
  }

  return {
    // State
    activeTab, showErrorDialog, selectedError,
    alerts, activeAlertsCount, qualityLevels,
    // From composable
    metrics, activeAlerts, isMonitoring,
    performanceMetrics, wsMetrics, apiCallRecords, errorRecords, resourceUsage, healthChecks, apiStats, errorStats,
    // Computed colors
    overallHealthColor, overallHealthStatus, apiStatusColor, errorRateColor, memoryUsageColor,
    lcpColorClass, clsColorClass,
    wsStatusColor, wsStatusLabel, qualityScoreColor, qualityLevelLabel,
    connectionDuration, lastHeartbeatAgo,
    // Methods
    startMonitoring, stopMonitoring,
    resolveError, clearErrors, clearApiRecords,
    formatBytes, formatTime, formatDuration, getMethodColor,
    getHealthIcon, getHealthColor, getAlertIcon, getAlertColor,
    resolveAlert, handleRefresh, showErrorDetail, handleResolveAll,
  };
}
