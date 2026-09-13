/**
 * @file StatsCharts/ChartFilters.vue
 * @description 图表筛选器子组件
 */

<template>
  <div class="chart-filters q-mb-md">
    <div class="row items-center justify-between">
      <div class="text-subtitle1">{{ title }}</div>
      <div class="row items-center gap-sm">
        <slot name="filters"></slot>
        <q-badge v-if="timeRangeLabel" color="grey-3" text-color="dark" :label="timeRangeLabel" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string;
  timeRangeLabel?: string;
}

withDefaults(defineProps<Props>(), {
  title: '',
  timeRangeLabel: '',
});
</script>

<style scoped>
.chart-filters {
  padding: 8px 0;
}
</style>
