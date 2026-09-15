<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('feedback.statistics') }}</div>

    <!-- 统计概览 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="mail_outline" size="40px" color="primary" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('feedback.total') }}</div>
            <div class="text-h6">{{ statistics.total || 0 }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="pending" size="40px" color="warning" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('feedback.pending') }}</div>
            <div class="text-h6">{{ statistics.pending || 0 }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="autorenew" size="40px" color="info" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('feedback.processing') }}</div>
            <div class="text-h6">{{ statistics.processing || 0 }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section class="row items-center">
          <q-icon name="check_circle" size="40px" color="positive" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('feedback.resolved') }}</div>
            <div class="text-h6">{{ statistics.resolved || 0 }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 性能指标 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section>
          <div class="text-caption text-grey q-mb-sm">{{ $t('feedback.resolutionRate') }}</div>
          <div class="text-h4 text-positive">{{ resolutionRate }}%</div>
          <q-linear-progress
            :value="resolutionRate / 100"
            color="positive"
            track-color="grey-3"
            class="q-mt-sm"
          />
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section>
          <div class="text-caption text-grey q-mb-sm">{{ $t('feedback.avg_response_time') }}</div>
          <div class="text-h4 text-info">{{ avg_response_time }}</div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 200px">
        <q-card-section>
          <div class="text-caption text-grey q-mb-sm">{{ $t('feedback.satisfaction') }}</div>
          <div class="text-h4 text-positive">
            <q-icon name="star" color="amber" />
            {{ statistics.satisfactionRate?.toFixed(1) || '0.0' }}
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 类型分布图表 -->
    <div class="row q-gutter-md">
      <q-card flat bordered class="col">
        <q-card-section>
          <div class="text-h6 q-mb-md">{{ $t('feedback.typeDistribution') }}</div>
          <div class="row q-gutter-md">
            <div
              v-for="item in typeStatistics"
              :key="item.type"
              class="col text-center"
            >
              <q-circular-progress
                :value="item.percentage / 100"
                size="80px"
                :color="getTypeColor(item.type)"
                track-color="grey-3"
                show-value
                class="q-mb-sm"
              >
                <span class="text-caption">{{ item.percentage }}%</span>
              </q-circular-progress>
              <div class="text-body2">{{ getTypeLabel(item.type) }}</div>
              <div class="text-caption text-grey">{{ item.count }} {{ $t('feedback.items') }}</div>
            </div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 趋势图表区域 -->
    <q-card flat bordered class="q-mt-md">
      <q-card-section>
        <div class="text-h6 q-mb-md">{{ $t('feedback.trendChart') }}</div>
        <div v-if="!chartLoading" class="chart-placeholder">
          <!-- 实际项目中可集成 ECharts 或其他图表库 -->
          <div class="text-grey text-center q-pa-xl">
            <q-icon name="bar_chart" size="64px" />
            <div class="q-mt-md">{{ $t('feedback.chartPlaceholder') }}</div>
          </div>
        </div>
        <div v-else class="text-center q-pa-xl">
          <q-spinner-dots size="48px" color="primary" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 时间范围选择 -->
    <q-card flat bordered class="q-mt-md">
      <q-card-section class="row q-gutter-md items-center">
        <span class="text-body1">{{ $t('feedback.timeRange') }}:</span>
        <q-btn-toggle
          v-model="timeRange"
          toggle-color="primary"
          :options="[
            { label: $t('feedback.today'), value: 'today' },
            { label: $t('feedback.last7Days'), value: 'week' },
            { label: $t('feedback.last30Days'), value: 'month' },
            { label: $t('feedback.all'), value: 'all' },
          ]"
          @update:model-value="loadStatistics"
        />
        <q-space />
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="loadStatistics" />
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file FeedbackStatisticsPage.vue
 * @description 反馈统计页面
 * @date 2026-05-05
 */

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useFeedbackStore } from '@/stores/ebike';

const { t } = useI18n();
const matRefresh = 'refresh';


const feedbackStore = useFeedbackStore();

const timeRange = ref('week');
const chartLoading = ref(false);

const statistics = computed(() => feedbackStore.statistics || {
  total: 0,
  pending: 0,
  processing: 0,
  resolved: 0,
  rejected: 0,
  avg_response_time: 0,
  satisfactionRate: 0,
});

const typeStatistics = computed(() => feedbackStore.typeStatistics || []);

const resolutionRate = computed(() => {
  const stats = feedbackStore.statistics;
  if (!stats || stats.total === 0) return 0;
  return Math.round((stats.resolved / stats.total) * 100);
});

const avg_response_time = computed(() => {
  const minutes = feedbackStore.statistics?.avg_response_time || 0;
  if (minutes < 60) return `${minutes} ${t('feedback.minutes')}`;
  const hours = Math.round(minutes / 60);
  return `${hours} ${t('feedback.hours')}`;
});

function getTypeColor(type: string) {
  const colors: Record<string, string> = {
    suggestion: 'positive',
    bug: 'negative',
    complaint: 'warning',
    other: 'grey',
  };
  return colors[type] || 'primary';
}

function getTypeLabel(type: string) {
  const labels: Record<string, string> = {
    suggestion: t('feedback.suggestion'),
    bug: t('feedback.bug'),
    complaint: t('feedback.complaint'),
    other: t('feedback.other'),
  };
  return labels[type] || type;
}

async function loadStatistics() {
  chartLoading.value = true;
  // 使用 start_date 代替 range
  const params = timeRange.value !== 'all' ? { start_date: timeRange.value } : {};

  await Promise.all([
    feedbackStore.fetchStatistics(params),
    feedbackStore.fetchTypeStatistics(params),
  ]);

  chartLoading.value = false;
}

onMounted(() => {
  void loadStatistics();
});
</script>

<style scoped>
.chart-placeholder {
  min-height: 300px;
}
</style>
