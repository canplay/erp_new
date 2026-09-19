/**
 * @file usePerformanceMonitor.ts
 * @description 性能监控 Composable
 * @date 2026-04-04
 */

import { ref, computed, onMounted, onUnmounted } from 'vue';
import type {
  PerformanceMetrics,
  WebSocketMetrics,
  ApiCallRecord,
  FrontendError,
  ResourceUsage,
  HealthCheckResult,
} from '@/types/monitor';

/**
 * @brief 性能监控 Composable
 */
export function usePerformanceMonitor() {
  const performanceMetrics = ref<PerformanceMetrics | null>(null);
  const wsMetrics = ref<WebSocketMetrics>({
    status: 'disconnected',
    connectedAt: null,
    lastHeartbeatAt: null,
    reconnectAttempts: 0,
    messagesSent: 0,
    messagesReceived: 0,
    errorCount: 0,
    avg_response_time: 0,
    qualityScore: 0,
  });
  const apiCallRecords = ref<ApiCallRecord[]>([]);
  const errorRecords = ref<FrontendError[]>([]);
  const resourceUsage = ref<ResourceUsage | null>(null);
  const healthChecks = ref<HealthCheckResult[]>([]);
  const isMonitoring = ref(false);
  let monitorTimer: ReturnType<typeof setInterval> | null = null;

  // 计算属性
  const apiStats = computed(() => {
    const records = apiCallRecords.value;
    if (records.length === 0) {
      return { total: 0, success: 0, failed: 0, avg_response_time: 0, p95_response_time: 0, errorRate: 0 };
    }
    const success = records.filter((r) => r.success).length;
    const failed = records.length - success;
    const response_times = records.map((r) => r.response_time).sort((a, b) => a - b);
    const avg_response_time = response_times.reduce((a, b) => a + b, 0) / response_times.length;
    const p95Index = Math.floor(response_times.length * 0.95);
    return {
      total: records.length,
      success,
      failed,
      avg_response_time: Math.round(avg_response_time),
      p95_response_time: response_times[p95Index] || 0,
      errorRate: Math.round((failed / records.length) * 100),
    };
  });

  const errorStats = computed(() => {
    const errors = errorRecords.value;
    const unresolved = errors.filter((e) => !e.resolved).length;
    const byType: Record<string, number> = {};
    errors.forEach((e) => { byType[e.type] = (byType[e.type] || 0) + 1; });
    return { total: errors.length, unresolved, byType };
  });

  // 性能指标收集
  function collectPerformanceMetrics(): PerformanceMetrics {
    const timing = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    const paint = performance.getEntriesByType('paint');
    const fcpEntry = paint.find((entry) => entry.name === 'first-contentful-paint');
    const FCP = fcpEntry ? fcpEntry.startTime : 0;
    const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
    const LCP = lcpEntries.length > 0 ? (lcpEntries[lcpEntries.length - 1] as { startTime: number }).startTime : 0;
    const layoutShiftEntries = performance.getEntriesByType('layout-shift');
    const CLS = (layoutShiftEntries as { value: number }[]).reduce((sum, entry) => sum + entry.value, 0);

    const metrics: PerformanceMetrics = {
      FCP: Math.round(FCP),
      LCP: Math.round(LCP),
      CLS: Math.round(CLS * 1000) / 1000,
      FID: 0,
      pageLoadTime: timing ? Math.round(timing.loadEventEnd - timing.startTime) : 0,
      DOMContentLoaded: timing ? Math.round(timing.domContentLoadedEventEnd - timing.startTime) : 0,
      loadComplete: timing ? Math.round(timing.loadEventEnd - timing.startTime) : 0,
      timestamp: Date.now(),
    };
    performanceMetrics.value = metrics;
    return metrics;
  }

  function collectResourceUsage(): ResourceUsage {
    const memory = (performance as { memory?: { usedJSHeapSize: number; jsHeapSizeLimit: number } }).memory;
    const memoryUsed = memory?.usedJSHeapSize || 0;
    const memoryTotal = memory?.jsHeapSizeLimit || 0;
    const memoryUsage = memoryTotal > 0 ? (memoryUsed / memoryTotal) * 100 : 0;
    const connection = (navigator as { connection?: { effectiveType?: string } }).connection;

    return {
      cpuUsage: 0,
      memoryUsage: Math.round(memoryUsage * 100) / 100,
      memoryTotal,
      memoryUsed,
      storageUsage: 0,
      storageTotal: 0,
      storageUsed: 0,
      networkStatus: navigator.onLine ? 'online' : 'offline',
      networkType: connection?.effectiveType || 'unknown',
      timestamp: Date.now(),
    };
  }

  function recordApiCall(record: Omit<ApiCallRecord, 'id' | 'timestamp'>) {
    const fullRecord: ApiCallRecord = { ...record, id: `api_${Date.now()}`, timestamp: Date.now() };
    apiCallRecords.value.unshift(fullRecord);
    if (apiCallRecords.value.length > 100) apiCallRecords.value = apiCallRecords.value.slice(0, 100);
  }

  function recordError(error: Omit<FrontendError, 'id' | 'timestamp' | 'resolved'>) {
    const fullError: FrontendError = { ...error, id: `err_${Date.now()}`, timestamp: Date.now(), resolved: false };
    errorRecords.value.unshift(fullError);
    if (errorRecords.value.length > 50) errorRecords.value = errorRecords.value.slice(0, 50);
  }

  function performHealthCheck(): Promise<HealthCheckResult[]> {
    const results: HealthCheckResult[] = [];
    const apiHealth: HealthCheckResult = {
      name: 'API 响应时间',
      status: apiStats.value.avg_response_time > 2000 ? 'critical' : apiStats.value.avg_response_time > 1000 ? 'warning' : 'healthy',
      message: `平均响应时间: ${apiStats.value.avg_response_time}ms`,
      response_time: apiStats.value.avg_response_time,
    };
    results.push(apiHealth);

    const wsHealth: HealthCheckResult = {
      name: 'WebSocket 连接',
      status: wsMetrics.value.status === 'connected' ? 'healthy' : 'warning',
      message: wsMetrics.value.status === 'connected' ? '已连接' : `状态: ${wsMetrics.value.status}`,
    };
    results.push(wsHealth);

    const errorHealth: HealthCheckResult = {
      name: '错误率',
      status: errorStats.value.unresolved > 10 ? 'critical' : errorStats.value.unresolved > 5 ? 'warning' : 'healthy',
      message: `未处理错误: ${errorStats.value.unresolved}`,
    };
    results.push(errorHealth);

    healthChecks.value = results;
    return Promise.resolve(results);
  }

  function updateWsMetrics(updates: Partial<WebSocketMetrics>) {
    Object.assign(wsMetrics.value, updates);
  }

  function resolveError(errorId: string) {
    const error = errorRecords.value.find((e) => e.id === errorId);
    if (error) error.resolved = true;
  }

  function clearErrors() { errorRecords.value = []; }
  function clearApiRecords() { apiCallRecords.value = []; }

  function startMonitoring(interval = 5000) {
    if (isMonitoring.value) return;
    isMonitoring.value = true;
    collectPerformanceMetrics();
    collectResourceUsage();
    void performHealthCheck();
    monitorTimer = setInterval(() => {
      collectPerformanceMetrics();
      collectResourceUsage();
    }, interval);
  }

  function stopMonitoring() {
    isMonitoring.value = false;
    if (monitorTimer) { clearInterval(monitorTimer); monitorTimer = null; }
  }

  onMounted(() => {
    collectPerformanceMetrics();
    collectResourceUsage();
  });

  onUnmounted(() => { stopMonitoring(); });

  return {
    performanceMetrics, wsMetrics, apiCallRecords, errorRecords, resourceUsage, healthChecks, isMonitoring,
    apiStats, errorStats,
    collectPerformanceMetrics, collectResourceUsage, performHealthCheck,
    recordApiCall, recordError, updateWsMetrics, resolveError,
    clearErrors, clearApiRecords, startMonitoring, stopMonitoring,
  };
}
