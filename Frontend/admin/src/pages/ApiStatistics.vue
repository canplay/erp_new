// @ts-nocheck
/**
 * @file ApiStatistics.vue
 * @description API统计页面 - 组合 StatsCards + StatsCharts
 * @date 2026-04-04
 */

<template>
  <div class="api-statistics">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="bar_chart" class="q-mr-sm" />
        {{ $t('apiStatistics.title') }}
      </div>
      <q-space />
      <q-btn-group flat>
        <q-btn flat icon="refresh" @click="handleRefresh" />
        <q-btn-dropdown flat icon="schedule" :label="$t('common.dateRange')">
          <q-list>
            <q-item clickable v-close-popup @click="timeRange = '1h'">
              <q-item-section>{{ $t('apiStatistics.timeRangeOptions.last1Hour') }}</q-item-section>
            </q-item>
            <q-item clickable v-close-popup @click="timeRange = '24h'">
              <q-item-section>{{ $t('apiStatistics.timeRangeOptions.last24Hours') }}</q-item-section>
            </q-item>
            <q-item clickable v-close-popup @click="timeRange = '7d'">
              <q-item-section>{{ $t('apiStatistics.timeRangeOptions.last7Days') }}</q-item-section>
            </q-item>
            <q-item clickable v-close-popup @click="timeRange = '30d'">
              <q-item-section>{{ $t('apiStatistics.timeRangeOptions.last30Days') }}</q-item-section>
            </q-item>
          </q-list>
        </q-btn-dropdown>
      </q-btn-group>
    </div>

    <!-- 概览统计卡片 -->
    <StatsCards
      :total-stats="totalStats"
      :time-range-label="timeRangeLabel"
      :total-calls-label="$t('apiStatistics.totalCalls')"
      :success-count-label="$t('apiStatistics.successCount')"
      :failure-count-label="$t('apiStatistics.failureCount')"
      :qps-label="$t('apiStatistics.qpsLabel')"
      :avg-response-time-label="$t('apiStatistics.averageResponseTime')"
    />

    <!-- 图表和表格 -->
    <StatsCharts
      :trend-data="trendData"
      :response-distribution="responseDistribution"
      :category-stats="categoryStats"
      :baseline-alerts="baselineAlerts"
      :endpoint-stats="endpointStats"
      :endpoint-columns="endpointColumns"
      :max-response-time="max_response_time"
      :time-range-label="timeRangeLabel"
      :trend-title="$t('apiStatistics.responseTimeTrend')"
      :legend-avg="$t('apiStatistics.legendAvgResponse')"
      :legend-p95="$t('apiStatistics.legendP95Response')"
      :dist-title="$t('apiStatistics.responseTimeDistribution')"
      :category-title="$t('apiStatistics.apiCategoryStats')"
      :call-count-suffix="$t('apiStatistics.callCountSuffix')"
      :baseline-title="$t('apiStatistics.performanceBaseline')"
      :all-normal-msg="$t('apiStatistics.allApiPerformanceNormal')"
      :critical-label="$t('apiStatistics.baselineCritical')"
      :degraded-label="$t('apiStatistics.baselineDegraded')"
      :normal-label="$t('apiStatistics.baselineNormal')"
      :baseline-label="$t('apiStatistics.baselineLabel')"
      :current-label="$t('apiStatistics.currentLabel')"
      :deviation-label="$t('apiStatistics.deviationLabel')"
      :endpoint-title="$t('apiStatistics.endpointDetails')"
    />
  </div>
</template>

<script setup lang="ts">
import { useApiStatistics } from '@/composables/useApiStatistics';
import StatsCards from '@/components/ApiStatistics/StatsCards.vue';
import StatsCharts from '@/components/ApiStatistics/StatsCharts.vue';

const {
  timeRange, totalStats, endpointStats, categoryStats, trendData, responseDistribution,
  timeRangeLabel, max_response_time, baselineAlerts, endpointColumns,
  getMethodColor, getDistributionColor, getBaselineStatusColor, formatTime,
  handleRefresh,
} = useApiStatistics();
</script>

<style scoped>
.api-statistics {
  padding: 16px;
}
</style>
