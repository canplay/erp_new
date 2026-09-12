<template>
  <div class="health-dashboard">
    <!-- 页面标题和操作 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="monitor_heart" class="q-mr-sm" />
        {{ $t('monitor.healthDashboardTitle') }}
      </div>
      <q-space />
      <q-btn
        :color="isMonitoring ? 'negative' : 'primary'"
        :icon="isMonitoring ? 'stop' : 'play_arrow'"
        :label="isMonitoring ? $t('monitor.stopMonitoring') : $t('monitor.startMonitoring')"
        @click="toggleMonitoring"
      />
      <q-btn flat color="grey" icon="refresh" @click="handleRefresh" />
    </div>

    <!-- 概览卡片 -->
    <div class="row q-col-gutter-md q-mb-md">
      <!-- 健康状态 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="overview-card">
          <q-card-section class="row items-center">
            <q-icon name="favorite" size="40px" :color="overallHealthColor" class="q-mr-md" />
            <div>
              <div class="text-h6">{{ $t('common.systemHealth') }}</div>
              <div class="text-subtitle2" :class="`text-${overallHealthColor}`">
                {{ overallHealthStatus }}
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 活跃告警 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="overview-card">
          <q-card-section class="row items-center">
            <q-icon name="warning" size="40px" :color="activeAlertsCount > 0 ? 'negative' : 'positive'" class="q-mr-md" />
            <div>
              <div class="text-h6">{{ activeAlertsCount }}</div>
              <div class="text-subtitle2 text-grey">{{ $t('monitor.activeAlertsLabel') }}</div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- API 性能 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="overview-card">
          <q-card-section class="row items-center">
            <q-icon name="speed" size="40px" :color="apiStatusColor" class="q-mr-md" />
            <div>
              <div class="text-h6">{{ apiStats.avg_response_time }}ms</div>
              <div class="text-subtitle2 text-grey">{{ $t('monitor.avgResponseTime') }}</div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 错误率 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="overview-card">
          <q-card-section class="row items-center">
            <q-icon name="error" size="40px" :color="errorRateColor" class="q-mr-md" />
            <div>
              <div class="text-h6">{{ apiStats.errorRate }}%</div>
              <div class="text-subtitle2 text-grey">{{ $t('monitor.apiErrorRate') }}</div>
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 标签页 -->
    <q-tabs
      v-model="activeTab"
      class="q-mb-md"
      align="left"
      active-color="primary"
      indicator-color="primary"
    >
      <q-tab name="performance" icon="speed" :label="$t('common.coreMetrics')" />
      <q-tab name="websocket" icon="cable" label="WebSocket" />
      <q-tab name="api" icon="api" :label="$t('common.callRecords')" />
      <q-tab name="errors" icon="bug_report" :label="$t('common.frontendErrors')" />
      <q-tab name="health" icon="health_and_safety" :label="$t('common.systemHealth')" />
    </q-tabs>

    <q-tab-panels v-model="activeTab" animated>
      <!-- ========== 性能指标 ========== -->
      <q-tab-panel name="performance" class="q-pa-none">
        <HealthPerformanceTab
          :metrics="metrics"
          :api-stats="apiStats"
          :resource-usage="resourceUsage as any"
          :lcp-color-class="lcpColorClass"
          :cls-color-class="clsColorClass"
          :memory-usage-color="memoryUsageColor"
          :format-bytes="formatBytes"
        />
      </q-tab-panel>

      <!-- ========== WebSocket ========== -->
      <q-tab-panel name="websocket" class="q-pa-none">
        <HealthWebSocketTab
          :ws-metrics="wsMetrics"
          :ws-status-color="wsStatusColor"
          :ws-status-label="wsStatusLabel"
          :quality-score-color="qualityScoreColor"
          :quality-level-label="qualityLevelLabel"
          :connection-duration="connectionDuration"
          :last-heartbeat-ago="lastHeartbeatAgo"
          :quality-levels="qualityLevels"
        />
      </q-tab-panel>

      <!-- ========== API 调用 ========== -->
      <q-tab-panel name="api" class="q-pa-none">
        <HealthApiTab
          :api-call-records="apiCallRecords"
          :api-columns="apiColumns"
          :get-method-color="getMethodColor"
          :get-response-time-class="getResponseTimeClass"
          :format-time="formatTime"
          @clear="clearApiRecords"
        />
      </q-tab-panel>

      <!-- ========== 错误记录 ========== -->
      <q-tab-panel name="errors" class="q-pa-none">
        <HealthErrorsTab
          :error-records="errorRecords"
          :error-columns="errorColumns"
          :get-error-type-color="getErrorTypeColor"
          @resolve-all="handleResolveAll"
          @clear="clearErrors"
          @show-detail="showErrorDetail"
        />
      </q-tab-panel>

      <!-- ========== 健康检查 ========== -->
      <q-tab-panel name="health" class="q-pa-none">
        <HealthChecksTab
          :health-checks="healthChecks"
          :active-alerts="activeAlerts"
          :get-health-icon="getHealthIcon"
          :get-health-color="getHealthColor"
          :get-alert-icon="getAlertIcon"
          :get-alert-color="getAlertColor"
          @resolve-alert="resolveAlert"
        />
      </q-tab-panel>
    </q-tab-panels>

    <!-- 错误详情对话框 -->
    <q-dialog v-model="showErrorDialog">
      <q-card style="min-width: 600px; max-width: 800px;">
        <q-card-section>
          <div class="text-h6">{{ $t('monitor.errorDetails') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section v-if="selectedError">
          <q-list>
            <q-item>
              <q-item-section>{{ $t('monitor.errorType') }}</q-item-section>
              <q-item-section side>
                <q-badge :color="getErrorTypeColor(selectedError.type)" :label="selectedError.type" />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('monitor.errorMessage') }}</q-item-section>
              <q-item-section side class="text-negative">{{ selectedError.message }}</q-item-section>
            </q-item>
            <q-item v-if="selectedError.filename">
              <q-item-section>{{ $t('monitor.fileName') }}</q-item-section>
              <q-item-section side>{{ selectedError.filename }}</q-item-section>
            </q-item>
            <q-item v-if="selectedError.lineno">
              <q-item-section>{{ $t('monitor.location') }}</q-item-section>
              <q-item-section side>{{ $t('monitor.lineCol', { line: selectedError.lineno, col: selectedError.colno }) }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('common.time') }}</q-item-section>
              <q-item-section side>{{ formatTime(selectedError.timestamp) }}</q-item-section>
            </q-item>
          </q-list>

          <q-separator class="q-my-md" />

          <div class="text-subtitle2 q-mb-sm">{{ $t('monitor.stackInfo') }}</div>
          <q-card class="bg-grey-9" flat>
            <q-card-section>
              <pre class="text-white text-caption" style="white-space: pre-wrap; word-break: break-all;">{{ selectedError.stack || $t('monitor.noStackTrace') }}</pre>
            </q-card-section>
          </q-card>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * @file HealthDashboard.vue
 * @description 系统健康监控仪表盘
 * @date 2026-04-04
 */

import { onMounted } from 'vue';
import type { QTableProps } from 'quasar';
import { useHealthDashboard } from '@/composables/useHealthDashboard';
import HealthPerformanceTab from '@/components/HealthPerformanceTab.vue';
import HealthWebSocketTab from '@/components/HealthWebSocketTab.vue';
import HealthApiTab from '@/components/HealthApiTab.vue';
import HealthErrorsTab from '@/components/HealthErrorsTab.vue';
import HealthChecksTab from '@/components/HealthChecksTab.vue';

const hd = useHealthDashboard();

// 解构模板所需属性
const {
  activeTab, showErrorDialog, selectedError,
  isMonitoring, metrics, apiStats,
  wsMetrics, apiCallRecords, errorRecords, resourceUsage, healthChecks,
  activeAlerts, activeAlertsCount, qualityLevels,
  overallHealthColor, overallHealthStatus, apiStatusColor, errorRateColor, memoryUsageColor,
  lcpColorClass, clsColorClass,
  wsStatusColor, wsStatusLabel, qualityScoreColor, qualityLevelLabel,
  connectionDuration, lastHeartbeatAgo,
  startMonitoring, stopMonitoring,
  clearErrors, clearApiRecords,
  formatBytes, formatTime, getMethodColor,
  getHealthIcon, getHealthColor, getAlertIcon, getAlertColor,
  resolveAlert,
  handleRefresh, showErrorDetail, handleResolveAll,
} = hd;

// ============ 表格列定义 ============

const apiColumns: QTableProps['columns'] = [
  { name: 'method', label: '方法', field: 'method', align: 'center', style: 'width: 80px' },
  { name: 'path', label: '路径', field: 'path', align: 'left', sortable: true },
  { name: 'status', label: '状态', field: 'status', align: 'center', style: 'width: 80px' },
  { name: 'response_time', label: '响应时间', field: 'response_time', align: 'center', style: 'width: 100px', sortable: true },
  { name: 'timestamp', label: '时间', field: 'timestamp', align: 'center', style: 'width: 160px' },
];

const errorColumns: QTableProps['columns'] = [
  { name: 'type', label: '类型', field: 'type', align: 'center', style: 'width: 100px' },
  { name: 'message', label: '消息', field: 'message', align: 'left' },
  { name: 'resolved', label: '状态', field: 'resolved', align: 'center', style: 'width: 80px' },
  { name: 'timestamp', label: '时间', field: 'timestamp', align: 'center', style: 'width: 160px' },
  { name: 'actions', label: '操作', field: 'actions', align: 'center', style: 'width: 100px' },
];

// ============ 方法 (via useHealthDashboard) ============

function toggleMonitoring() {
  if (isMonitoring.value) {
    stopMonitoring();
  } else {
    startMonitoring(5000);
  }
}

function getResponseTimeClass(time: number): string {
  if (time > 2000) return 'text-negative text-weight-bold';
  if (time > 1000) return 'text-warning';
  return 'text-positive';
}

function getErrorTypeColor(type: string): string {
  const colors: Record<string, string> = {
    javascript: 'negative',
    resource: 'warning',
    promise: 'info',
    vue: 'secondary',
    network: 'grey',
  };
  return colors[type] || 'grey';
}

// 辅助函数（via useHealthDashboard）

// ============ 生命周期 ============

onMounted(() => {
  // 自动开始监控
  startMonitoring(5000);
});
</script>

<style scoped>
.health-dashboard {
  padding: 16px;
}

.overview-card {
  border-left: 4px solid;
  transition: all 0.3s;
}

.overview-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.metric-card {
  border-radius: 8px;
  transition: all 0.3s;
}

.metric-card:hover {
  transform: scale(1.02);
}

.stat-mini-card {
  border-radius: 8px;
}

.body--dark .q-linear-progress {
  background: rgba(255, 255, 255, 0.1);
}
</style>
