<template>
  <div class="api-statistics">
    <div class="text-h5 q-mb-md">{{ $t('api.title') }}</div>

    <div class="row q-col-gutter-sm q-mb-md">
      <StatCard
        :title="$t('common.totalRequests')"
        :value="stats.totalRequests"
        icon="api"
        color="primary"
      />
      <StatCard
        :title="$t('common.successRequests')"
        :value="stats.successRequests"
        icon="check_circle"
        color="positive"
      />
      <div class="col-3">
        <StatCard
          :title="$t('common.failedRequests')"
          :value="stats.failedRequests"
          icon="cancel"
          color="negative"
        />
      </div>
      <div class="col-3">
        <StatCard
          :title="$t('common.avgResponseTime')"
          :value="`${stats.avgResponseTime}ms`"
          icon="schedule"
          color="warning"
        />
      </div>
    </div>

    <div class="row q-col-gutter-sm">
      <div class="col-8">
        <ChartContainer :title="$t('common.requestTrend')" :chart-data="stats.trendData" />
      </div>
      <div class="col-4">
        <ChartContainer :title="$t('common.statusCodeDistribution')" :chart-data="stats.statusData" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import StatCard from './StatCard.vue'
import ChartContainer from './ChartContainer.vue'
import { useApiStatistics } from './useApiStatistics'

const { t: $t } = useI18n()

const { stats } = useApiStatistics()
</script>

<style scoped>
.api-statistics {
  padding: 24px;
}
</style>
