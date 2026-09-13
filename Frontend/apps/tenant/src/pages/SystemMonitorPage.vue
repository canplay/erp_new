<template>
  <q-page class="q-pa-md system-monitor">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">{{ $t('monitor.title') || '系统监控' }}</div>
      <q-space />
      <q-btn
        flat
        color="primary"
        icon="refresh"
        :label="$t('common.refresh')"
        @click="loadData"
        :loading="loading"
      />
    </div>

    <!-- 系统状态卡片 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <!-- 系统健康状态 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="monitor-card" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="monitor-label">{{ $t('monitor.systemHealth') || '系统健康' }}</div>
              <div class="monitor-value" :class="healthStatusClass">
                {{ healthStatus }}
              </div>
              <div class="monitor-trend">
                <q-icon name="schedule" size="14px" />
                <span>{{ $t('monitor.lastCheck') || '最后检查' }}: {{ lastCheckTime }}</span>
              </div>
            </div>
            <div class="monitor-icon" :style="{ background: healthStatusColor }">
              <q-icon name="favorite" />
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- API响应时间 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="monitor-card" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="monitor-label">{{ $t('monitor.apiResponse') || 'API响应' }}</div>
              <div class="monitor-value" :class="response_time_class">
                {{ avg_response_time }}ms
              </div>
              <div class="monitor-trend" :class="responseTrendClass">
                <q-icon :name="responseTrendIcon" size="14px" />
                <span>{{ responseTrendText }}</span>
              </div>
            </div>
            <div class="monitor-icon" style="background: linear-gradient(135deg, #51cf66 0%, #40c057 100%)">
              <q-icon name="speed" />
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 在线用户 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="monitor-card" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="monitor-label">{{ $t('monitor.onlineUsers') || '在线用户' }}</div>
              <div class="monitor-value" style="color: #339af0">
                {{ onlineUsers }}
              </div>
              <div class="monitor-trend">
                <q-icon name="people" size="14px" />
                <span>{{ $t('monitor.activeSessions') || '活跃会话' }}</span>
              </div>
            </div>
            <div class="monitor-icon" style="background: linear-gradient(135deg, #339af0 0%, #228be6 100%)">
              <q-icon name="people" />
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 错误率 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="monitor-card" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="monitor-label">{{ $t('monitor.errorRate') || '错误率' }}</div>
              <div class="monitor-value" :class="errorRateClass">
                {{ errorRate }}%
              </div>
              <div class="monitor-trend">
                <q-icon name="bug_report" size="14px" />
                <span>{{ $t('monitor.todayErrors') || '今日错误' }}: {{ todayErrors }}</span>
              </div>
            </div>
            <div class="monitor-icon" style="background: linear-gradient(135deg, #ff6b6b 0%, #f03e3e 100%)">
              <q-icon name="error" />
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 图表区域 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <!-- API响应时间趋势 -->
      <div class="col-12 col-md-8">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('monitor.response_time_trend') || 'API响应时间趋势' }}</div>
            <ECharts :config="response_time_chart_config" />
          </q-card-section>
        </q-card>
      </div>

      <!-- 系统状态分布 -->
      <div class="col-12 col-md-4">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('monitor.apiStatusDistribution') || 'API状态分布' }}</div>
            <ECharts :config="statusDistributionConfig" />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 详细数据表格 -->
    <div class="row q-col-gutter-md">
      <!-- 最近API调用 -->
      <div class="col-12 col-lg-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('monitor.recentApiCalls') || '最近API调用' }}</div>
            <q-table
              :rows="recentApiCalls"
              :columns="apiCallColumns"
              row-key="id"
              flat
              bordered
              :rows-per-page-options="[5]"
              hide-pagination
            >
              <template v-slot:body-cell-status="props">
                <q-td :props="props">
                  <q-badge :color="getStatusColor(props.value)" :label="props.value" />
                </q-td>
              </template>
              <template v-slot:body-cell-duration="props">
                <q-td :props="props">
                  <span :class="getDurationClass(props.value)">{{ props.value }}ms</span>
                </q-td>
              </template>
              <template v-slot:body-cell-time="props">
                <q-td :props="props">{{ formatTime(props.value) }}</q-td>
              </template>
            </q-table>
          </q-card-section>
        </q-card>
      </div>

      <!-- 最近错误日志 -->
      <div class="col-12 col-lg-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('monitor.recentErrors') || '最近错误' }}</div>
            <q-table
              :rows="recentErrors"
              :columns="errorColumns"
              row-key="id"
              flat
              bordered
              :rows-per-page-options="[5]"
              hide-pagination
            >
              <template v-slot:body-cell-level="props">
                <q-td :props="props">
                  <q-badge :color="getLevelColor(props.value)" :label="props.value" />
                </q-td>
              </template>
              <template v-slot:body-cell-time="props">
                <q-td :props="props">{{ formatTime(props.value) }}</q-td>
              </template>
            </q-table>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-overlay">
      <q-spinner-dots size="50px" color="primary" />
    </div>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file SystemMonitorPage.vue
 * @description 系统监控页面 - 审计与监控面板
 * @date 2026-04-04
 */

import ECharts from '@erp-new-frontend-monorepo/components/src/ECharts/Main.vue';
import { useSystemMonitor } from '@erp-new-frontend-monorepo/composables/src/useSystemMonitor';;

const {
  loading,
  healthStatus,
  lastCheckTime,
  avg_response_time,
  onlineUsers,
  errorRate,
  todayErrors,
  recentApiCalls,
  recentErrors,
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
} = useSystemMonitor();
</script>

<style scoped>
.system-monitor {
  padding: 24px;
}

.monitor-card {
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.monitor-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.monitor-label {
  font-size: 14px;
  color: #666;
  margin-bottom: 4px;
}

.monitor-value {
  font-size: 28px;
  font-weight: 700;
  color: #333;
  margin-bottom: 4px;
}

.monitor-trend {
  font-size: 12px;
  color: #999;
  display: flex;
  align-items: center;
  gap: 4px;
}

.monitor-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 24px;
}

.status-normal {
  color: #40c057;
}

.status-warning {
  color: #ff9800;
}

.status-error {
  color: #f03e3e;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9999;
}
</style>
