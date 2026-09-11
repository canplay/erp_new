/**
 * @file ECharts.vue
 * @description 基于 ECharts 的图表组件 - 支持多种图表类型
 * @date 2026-04-03
 */

<template>
  <div ref="chartRef" class="echarts-container" :style="{ width: chartWidth, height: chartHeight }" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import {
  LineChart,
  BarChart,
  PieChart,
} from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
} from 'echarts/components';
import * as echarts from 'echarts/core';

use([
  CanvasRenderer,
  LineChart,
  BarChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
]);

export interface ChartDataItem {
  label: string;
  value: number;
  [key: string]: unknown;
}

export interface EChartsConfig {
  type?: 'line' | 'bar' | 'pie';
  title?: string;
  data?: ChartDataItem[];
  width?: string | number;
  height?: string | number;
  showGrid?: boolean;
  showLegend?: boolean;
  showTooltip?: boolean;
  areaFill?: boolean;
}

const props = withDefaults(
  defineProps<{ config?: EChartsConfig }>(),
  { config: () => ({ type: 'bar' as const, data: [], width: '100%', height: '300px', showGrid: true, showLegend: true, showTooltip: true, areaFill: false }) }
);

const chartRef = ref<HTMLElement | null>(null);
let chartInstance: echarts.ECharts | null = null;

const chartWidth = computed(() => {
  const w = props.config?.width;
  if (w === undefined || w === null) return '100%';
  return typeof w === 'number' ? `${w}px` : w;
});

const chartHeight = computed(() => {
  const h = props.config?.height;
  if (h === undefined || h === null) return '300px';
  return typeof h === 'number' ? `${h}px` : h;
});

function buildChartOptions() {
  const { type = 'bar', title, data = [], showGrid = true, showLegend = true, showTooltip = true, areaFill = false } = props.config;

  const opts: Record<string, unknown> = {
    backgroundColor: 'transparent',
  };

  if (showTooltip) {
    opts.tooltip = { trigger: type === 'pie' ? 'item' : 'axis' };
  }

  if (title) {
    opts.title = { text: title, left: 'center', textStyle: { color: '#666', fontSize: 16 } };
  }

  if (showLegend && data.length > 0) {
    opts.legend = { bottom: 10, textStyle: { color: '#666' } };
  }

  if (type === 'pie') {
    const pieData = data.map(d => ({ name: d.label, value: d.value }));
    opts.series = [{ type: 'pie', radius: ['0%', '75%'], center: ['50%', '50%'], data: pieData, label: { show: data.length <= 10 } }];
    return opts;
  }

  if (showGrid) {
    opts.grid = { left: '3%', right: '4%', bottom: showLegend ? '15%' : '3%', top: title ? '15%' : '3%', containLabel: true };
  }

  opts.xAxis = { type: 'category', data: data.map(d => d.label), axisLabel: { color: '#666' }, axisLine: { lineStyle: { color: '#e0e0e0' } } };
  opts.yAxis = { type: 'value', axisLabel: { color: '#666' }, splitLine: { lineStyle: { color: '#e0e0e0' } } };
  opts.series = [{ type, data: data.map(d => d.value), smooth: true, areaStyle: areaFill ? { color: { type: 'linear', x: 0, y: 0, x2: 0, y2: 1, colorStops: [{ offset: 0, color: '#5470c680' }, { offset: 1, color: '#5470c610' }] } } : undefined }];

  return opts;
}

function initChart() {
  if (!chartRef.value) return;
  chartInstance = echarts.init(chartRef.value);
  chartInstance.setOption(buildChartOptions());
}

watch(() => props.config, () => { chartInstance?.setOption(buildChartOptions()); }, { deep: true });

onMounted(() => {
  initChart();
  window.addEventListener('resize', () => chartInstance?.resize());
});

onBeforeUnmount(() => {
  chartInstance?.dispose();
  chartInstance = null;
});
</script>

<style scoped>
.echarts-container { min-height: 200px; }
</style>
