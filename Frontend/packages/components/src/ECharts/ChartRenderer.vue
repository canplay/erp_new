/**
 * @file ChartRenderer.vue
 * @description 图表渲染引擎 - 基于 ECharts 的图表渲染
 * @date 2026-04-04
 */

<template>
  <div ref="chartRef" class="echarts-chart" :style="{ width, height }" />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
import { echarts } from '@/utils/echarts';
import type { EChartsOption } from '@/utils/echarts';
import type { ECharts } from '@/utils/echarts';

interface Props {
  option: EChartsOption;
  width?: string;
  height?: string;
  theme?: string;
  lazy?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  width: '100%',
  height: '300px',
  theme: 'default',
  lazy: false,
});

const emit = defineEmits<{
  rendered: [chart: ECharts];
  click: [params: unknown];
}>();

const chartRef = ref<HTMLElement | null>(null);
let chart: ECharts | null = null;

function initChart() {
  if (!chartRef.value) return;
  if (chart) {
    chart.dispose();
  }
  chart = echarts.init(chartRef.value, props.theme !== 'default' ? props.theme : undefined);
  updateChart();
}

function updateChart() {
  if (!chart) return;
  chart.setOption(props.option, true);
}

function handleChartClick(params: unknown) {
  emit('click', params);
}

function resize() {
  chart?.resize();
}

watch(() => props.option, () => updateChart(), { deep: true });

onMounted(async () => {
  if (props.lazy) {
    await nextTick();
  }
  initChart();
  window.addEventListener('resize', resize);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', resize);
  chart?.dispose();
  chart = null;
});
</script>

<style scoped>
.echarts-chart { width: v-bind('width'); height: v-bind('height'); }
</style>
