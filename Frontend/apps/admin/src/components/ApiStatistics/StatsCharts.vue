/**
 * @file StatsCharts.vue
 * @description API统计 - 趋势图表、分布、分类、基线告警、端点表格
 * @date 2026-08-22
 * @refactored 2026-09-12 - Split into composables
 */

<template>
  <div class="stats-charts">
    <div class="row q-col-gutter-md">
      <!-- 左侧：趋势图表和响应时间分布 -->
      <div class="col-12 col-lg-8">
        <!-- 响应时间趋势 -->
        <q-card flat bordered class="q-mb-md">
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">{{ trendTitle }}</div>
            <div class="chart-container">
              <div class="mock-chart">
                <div class="chart-bars">
                  <div v-for="(point, i) in trendData" :key="i" class="bar-wrapper">
                    <div
                      class="bar bar--avg"
                      :style="{ height: `${(point.avg_response_time / (max_response_time || 1)) * 100}%` }"
                    ></div>
                    <div
                      class="bar bar--p95"
                      :style="{ height: `${(point.p95_response_time / (max_response_time || 1)) * 100}%` }"
                    ></div>
                  </div>
                </div>
                <div class="chart-legend">
                  <span class="legend-item"><span class="legend-color legend-color--avg"></span> {{ legendAvg }}</span>
                  <span class="legend-item"><span class="legend-color legend-color--p95"></span> {{ legendP95 }}</span>
                </div>
              </div>
            </div>
          </q-card-section>
        </q-card>

        <!-- 响应时间分布 -->
        <q-card flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">{{ distTitle }}</div>
            <div class="distribution-bars">
              <div
                v-for="bucket in responseDistribution"
                :key="bucket.bucket"
                class="distribution-item"
              >
                <div class="distribution-label">{{ bucket.bucket }}</div>
                <div class="distribution-bar-wrapper">
                  <div
                    class="distribution-bar"
                    :style="{ width: `${bucket.percentage}%`, background: getDistributionColor(bucket.max) }"
                  ></div>
                </div>
                <div class="distribution-count">{{ bucket.count }}</div>
                <div class="distribution-percent">{{ bucket.percentage.toFixed(1) }}%</div>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 右侧：分类统计和性能基线告警 -->
      <div class="col-12 col-lg-4">
        <!-- 分类统计 -->
        <q-card flat bordered class="q-mb-md">
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">{{ categoryTitle }}</div>
            <div class="category-list">
              <div
                v-for="cat in categoryStats"
                :key="cat.category"
                class="category-item"
              >
                <div class="category-info">
                  <div class="category-name">{{ cat.category }}</div>
                  <div class="category-count text-caption text-grey">
                    {{ cat.callCount }} {{ callCountSuffix }}
                  </div>
                </div>
                <div class="category-metrics">
                  <div class="text-caption">{{ cat.avg_response_time }}ms</div>
                  <q-linear-progress
                    :value="cat.errorRate / 100"
                    color="negative"
                    class="q-mt-xs"
                    style="width: 60px;"
                  />
                  <div class="text-caption text-negative">{{ cat.errorRate.toFixed(1) }}%</div>
                </div>
              </div>
            </div>
          </q-card-section>
        </q-card>

        <!-- 性能基线告警 -->
        <q-card flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">
              {{ baselineTitle }}
              <q-badge v-if="baselineAlerts.length > 0" color="negative" :label="baselineAlerts.length" class="q-ml-sm" />
            </div>

            <div v-if="baselineAlerts.length === 0" class="text-center q-pa-md">
              <q-icon name="check_circle" size="32px" color="positive" />
              <div class="text-caption q-mt-sm">{{ allNormalMsg }}</div>
            </div>

            <div v-else class="baseline-alerts">
              <div
                v-for="alert in baselineAlerts"
                :key="alert.endpoint"
                class="baseline-alert"
                :class="`baseline-alert--${alert.status}`"
              >
                <div class="baseline-alert__header">
                  <q-badge
                    :color="getBaselineStatusColor(alert.status)"
                    :label="alert.status === 'critical' ? criticalLabel : alert.status === 'degraded' ? degradedLabel : normalLabel"
                  />
                  <span class="text-caption q-ml-sm">{{ alert.endpoint }}</span>
                </div>
                <div class="baseline-alert__metrics">
                  <span class="text-caption">
                    {{ baselineLabel }}: {{ alert.baselineAvg }}ms →
                    {{ currentLabel }}: <span :class="`text-${alert.status === 'normal' ? 'positive' : 'negative'}`">{{ alert.currentAvg }}ms</span>
                  </span>
                  <span class="text-caption q-ml-md">
                    {{ deviationLabel }}: <span :class="`text-${alert.status === 'normal' ? 'positive' : 'negative'}`">{{ alert.deviation > 0 ? '+' : '' }}{{ alert.deviation.toFixed(1) }}%</span>
                  </span>
                </div>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 端点详情表格 -->
    <q-card flat bordered class="q-mt-md">
      <q-card-section>
        <div class="text-subtitle1 q-mb-md">{{ endpointTitle }}</div>

        <q-table
          :rows="endpointStats"
          :columns="endpointColumns"
          row-key="path"
          flat
          :pagination="{ rowsPerPage: 10 }"
        >
          <template v-slot:body-cell-endpoint="props">
            <q-td :props="props">
              <div class="row items-center">
                <q-badge
                  :color="getMethodColor(props.row.method)"
                  text-color="white"
                  :label="props.row.method"
                  class="q-mr-sm"
                />
                <span class="text-body2">{{ props.row.path }}</span>
              </div>
            </q-td>
          </template>

          <template v-slot:body-cell-callCount="props">
            <q-td :props="props">
              <div class="text-h6">{{ props.row.callCount }}</div>
            </q-td>
          </template>

          <template v-slot:body-cell-errorRate="props">
            <q-td :props="props">
              <q-badge
                :color="props.row.errorRate > 5 ? 'negative' : props.row.errorRate > 1 ? 'warning' : 'positive'"
                text-color="white"
                :label="`${props.row.errorRate.toFixed(2)}%`"
              />
            </q-td>
          </template>

          <template v-slot:body-cell-response_time="props">
            <q-td :props="props">
              <div class="row items-center">
                <span class="q-mr-sm">{{ props.row.avg_response_time }}ms</span>
                <q-linear-progress
                  :value="props.row.avg_response_time / 1000"
                  color="primary"
                  style="width: 60px;"
                />
              </div>
            </q-td>
          </template>

          <template v-slot:body-cell-p95_response_time="props">
            <q-td :props="props">
              <span class="text-info">{{ props.row.p95_response_time }}ms</span>
            </q-td>
          </template>

          <template v-slot:body-cell-lastCalledAt="props">
            <q-td :props="props">
              <span class="text-caption">{{ formatTime(props.row.lastCalledAt) }}</span>
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>
  </div>
</template>

<script setup lang="ts">
import { useStatsCharts } from './composables/useStatsCharts';

// ============ Types ============

interface TrendPoint {
  avg_response_time: number;
  p95_response_time: number;
}

interface ResponseBucket {
  bucket: string;
  count: number;
  percentage: number;
  max: number;
}

interface CategoryStat {
  category: string;
  callCount: number;
  avg_response_time: number;
  errorRate: number;
}

interface BaselineAlert {
  endpoint: string;
  status: 'critical' | 'degraded' | 'normal';
  baselineAvg: number;
  currentAvg: number;
  deviation: number;
}

interface EndpointStat {
  path: string;
  method: string;
  callCount: number;
  errorRate: number;
  avg_response_time: number;
  p95_response_time: number;
  lastCalledAt: string;
}

// ============ Props ============

interface Props {
  trendData: TrendPoint[];
  responseDistribution: ResponseBucket[];
  categoryStats: CategoryStat[];
  baselineAlerts: BaselineAlert[];
  endpointStats: EndpointStat[];
  endpointColumns: { name: string; label: string; field?: string; align?: string }[];
  max_response_time: number;
  timeRangeLabel: string;
  trendTitle?: string;
  legendAvg?: string;
  legendP95?: string;
  distTitle?: string;
  categoryTitle?: string;
  callCountSuffix?: string;
  baselineTitle?: string;
  allNormalMsg?: string;
  criticalLabel?: string;
  degradedLabel?: string;
  normalLabel?: string;
  baselineLabel?: string;
  currentLabel?: string;
  deviationLabel?: string;
  endpointTitle?: string;
}

withDefaults(defineProps<Props>(), {
  max_response_time: 1000,
  trendTitle: '响应时间趋势',
  legendAvg: '平均响应时间',
  legendP95: 'P95响应时间',
  distTitle: '响应时间分布',
  categoryTitle: 'API分类统计',
  callCountSuffix: '次调用',
  baselineTitle: '性能基线告警',
  allNormalMsg: '所有API性能正常',
  criticalLabel: '严重',
  degradedLabel: '降级',
  normalLabel: '正常',
  baselineLabel: '基线',
  currentLabel: '当前',
  deviationLabel: '偏差',
  endpointTitle: '端点详情',
});

// ============ Composable ============

const { getMethodColor, getDistributionColor, getBaselineStatusColor, formatTime } = useStatsCharts();
</script>

<style scoped>
/* 模拟图表 */
.chart-container {
  height: 200px;
}

.mock-chart {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.chart-bars {
  flex: 1;
  display: flex;
  align-items: flex-end;
  gap: 4px;
  padding: 0 8px;
}

.bar-wrapper {
  flex: 1;
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 100%;
}

.bar {
  width: 50%;
  background: #1976d2;
  border-radius: 2px 2px 0 0;
  transition: height 0.3s;
}

.bar--avg {
  background: #1976d2;
}

.bar--p95 {
  background: #90caf9;
  opacity: 0.7;
}

.chart-legend {
  display: flex;
  justify-content: center;
  gap: 16px;
  padding: 8px 0;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
}

.legend-color--avg {
  background: #1976d2;
}

.legend-color--p95 {
  background: #90caf9;
}

/* 分布图表 */
.distribution-bars {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.distribution-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.distribution-label {
  width: 80px;
  font-size: 12px;
  color: #666;
}

.distribution-bar-wrapper {
  flex: 1;
  height: 20px;
  background: #f5f5f5;
  border-radius: 4px;
  overflow: hidden;
}

.distribution-bar {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s;
}

.distribution-count {
  width: 60px;
  text-align: right;
  font-size: 12px;
}

.distribution-percent {
  width: 50px;
  text-align: right;
  font-size: 12px;
  color: #666;
}

/* 分类列表 */
.category-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.category-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  border-radius: 4px;
  background: #f5f5f5;
}

.category-info {
  flex: 1;
}

.category-name {
  font-weight: 500;
}

.category-metrics {
  text-align: right;
}

/* 性能基线告警 */
.baseline-alerts {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.baseline-alert {
  padding: 8px;
  border-radius: 4px;
  border-left: 3px solid;
}

.baseline-alert--degraded {
  background: rgba(255, 152, 0, 0.1);
  border-color: #ff9800;
}

.baseline-alert--critical {
  background: rgba(244, 67, 54, 0.1);
  border-color: #f44336;
}

.baseline-alert__header {
  display: flex;
  align-items: center;
  margin-bottom: 4px;
}

.baseline-alert__metrics {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12px;
}
</style>
