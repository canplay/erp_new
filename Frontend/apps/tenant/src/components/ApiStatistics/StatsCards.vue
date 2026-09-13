/**
 * @file ApiStatistics/StatsCards.vue
 * @description API统计 - 概览统计卡片（总调用、成功、失败、QPS）
 * @date 2026-08-22
 */

<template>
  <div class="stats-cards">
    <!-- 总调用 -->
    <div class="col-12 col-sm-6 col-md-3">
      <q-card flat bordered class="stat-card stat-card--total">
        <q-card-section class="text-center">
          <q-icon name="lens" size="32px" color="primary" class="q-mb-sm" />
          <div class="text-h4 text-primary">{{ totalStats.totalCalls }}</div>
          <div class="text-caption">{{ totalCallsLabel }}</div>
          <div class="text-caption text-grey q-mt-xs">
            {{ timeRangeLabel }}
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 成功调用 -->
    <div class="col-12 col-sm-6 col-md-3">
      <q-card flat bordered class="stat-card stat-card--success">
        <q-card-section class="text-center">
          <q-icon name="check_circle" size="32px" color="positive" class="q-mb-sm" />
          <div class="text-h4 text-positive">{{ totalStats.successCalls }}</div>
          <div class="text-caption">{{ successCountLabel }}</div>
          <div class="text-caption text-positive q-mt-xs">
            {{ successRate }}%
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 失败调用 -->
    <div class="col-12 col-sm-6 col-md-3">
      <q-card flat bordered class="stat-card stat-card--failed">
        <q-card-section class="text-center">
          <q-icon name="error" size="32px" color="negative" class="q-mb-sm" />
          <div class="text-h4 text-negative">{{ totalStats.failedCalls }}</div>
          <div class="text-caption">{{ failureCountLabel }}</div>
          <div class="text-caption text-negative q-mt-xs">
            {{ totalStats.errorRate.toFixed(2) }}% 错误率
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- QPS -->
    <div class="col-12 col-sm-6 col-md-3">
      <q-card flat bordered class="stat-card stat-card--qps">
        <q-card-section class="text-center">
          <q-icon name="speed" size="32px" color="info" class="q-mb-sm" />
          <div class="text-h4 text-info">{{ totalStats.qps.toFixed(2) }}</div>
          <div class="text-caption">{{ qpsLabel }}</div>
          <div class="text-caption text-grey q-mt-xs">
            {{ avgResponseTimeLabel }} {{ totalStats.avg_response_time }}ms
          </div>
        </q-card-section>
      </q-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
interface TotalStats {
  totalCalls: number;
  successCalls: number;
  failedCalls: number;
  errorRate: number;
  qps: number;
  avg_response_time: number;
}

interface Props {
  totalStats: TotalStats;
  timeRangeLabel: string;
  totalCallsLabel?: string;
  successCountLabel?: string;
  failureCountLabel?: string;
  qpsLabel?: string;
  avgResponseTimeLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  totalCallsLabel: '总调用',
  successCountLabel: '成功',
  failureCountLabel: '失败',
  qpsLabel: 'QPS',
  avgResponseTimeLabel: '平均响应时间',
});

const successRate = computed(() => {
  if (props.totalStats.totalCalls === 0) return 0;
  return ((props.totalStats.successCalls / props.totalStats.totalCalls) * 100).toFixed(1);
});
</script>

<style scoped>
.stats-cards {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 16px;
}

.stat-card {
  border-radius: 8px;
  border-left: 4px solid;
  transition: all 0.3s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-card--total {
  border-color: #1976d2;
}

.stat-card--success {
  border-color: #4caf50;
}

.stat-card--failed {
  border-color: #f44336;
}

.stat-card--qps {
  border-color: #2196f3;
}
</style>
