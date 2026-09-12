/**
 * @file ECharts.vue
 * @description 基于 ECharts 的图表组件 - 主文件（拆分后 (300行)）
 * @date 2026-04-03
 */

<template>
  <div class="echarts-wrapper">
    <ChartRenderer
      :option="chartOption"
      :width="chartWidth"
      :height="chartHeight"
      @click="handleClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import type { EChartsOption } from '@/utils/echarts';
import ChartRenderer from './ChartRenderer.vue';

// ============ 类型定义 ============

/** 图表数据类型 */
export interface ChartDataItem {
  label: string;
  value: number;
  [key: string]: unknown;
}

/** 图表配置接口 */
export interface EChartsConfig {
  type?: 'line' | 'bar' | 'pie' | 'gauge' | 'radar' | 'scatter';
  title?: string;
  data?: ChartDataItem[];
  width?: string | number;
  height?: string | number;
  color?: string | string[];
  showGrid?: boolean;
  showLegend?: boolean;
  showTooltip?: boolean;
  showToolbox?: boolean;
  xAxisRotate?: number;
  stack?: boolean;
  areaFill?: boolean;
  pieInnerRadius?: number | string;
  pieRadius?: number | string;
  gaugeConfig?: { min?: number; max?: number; splitNumber?: number };
}

const { t } = useI18n();

const props = withDefaults(
  defineProps<{ config?: EChartsConfig }>(),
  {
    config: () => ({
      type: 'bar', data: [], width: '100%', height: '300px',
      showGrid: true, showLegend: true, showTooltip: true,
      showToolbox: false, areaFill: false,
    }),
  },
);

const emit = defineEmits<{
  (e: 'click', params: unknown): void;
  (e: 'ready', instance: unknown): void;
}>();

// ============ 计算属性 ============

const chartWidth = computed(() => {
  const w = props.config?.width;
  if (w === undefined || w === null) return '100%';
  if (typeof w === 'number') return `${w}px`;
  return w;
});

const chartHeight = computed(() => {
  const h = props.config?.height;
  if (h === undefined || h === null) return '300px';
  if (typeof h === 'number') return `${h}px`;
  return h;
});

const darkThemeColors = ['#5470c6','#91cc75','#fac858','#ee6666','#73c0de','#3ba272','#fc8452','#9a60b4','#ea7ccc'];
const lightThemeColors = ['#5470c6','#73c0de','#91cc75','#fac858','#ee6666','#9a60b4','#ea7ccc','#3ba272','#fc8452'];

const chartOption = computed<EChartsOption>(() => {
  const {
    type = 'bar', title, data = [], color, showGrid = true,
    showLegend = true, showTooltip = true, showToolbox = false,
    xAxisRotate = 0, areaFill = false, pieInnerRadius = 0,
    pieRadius = '75%', gaugeConfig = {},
  } = props.config;

  const textColor = '#666666';
  const splitLineColor = '#e0e0e0';
  const themeColors = lightThemeColors;

  const baseOptions = {
    backgroundColor: 'transparent',
    color: (color as string[]) || themeColors,
    title: title ? {
      text: title, left: 'center',
      textStyle: { color: textColor, fontSize: 16, fontWeight: 'bold' },
    } : undefined,
    tooltip: showTooltip ? {
      trigger: type === 'pie' || type === 'gauge' ? 'item' : 'axis',
      axisPointer: { type: 'shadow' },
      backgroundColor: '#ffffff',
      borderColor: '#e0e0e0',
      textStyle: { color: textColor },
    } : undefined,
    legend: showLegend && data.length > 0 ? {
      bottom: 10, textStyle: { color: textColor },
    } : undefined,
    toolbox: showToolbox ? {
      feature: {
        dataZoom: { title: { zoom: t('echarts.dataZoom'), back: t('echarts.dataZoomBack') } },
        magicType: { line: t('echarts.magicTypeLine'), bar: t('echarts.magicTypeBar') },
        restore: { title: t('echarts.restore') },
        saveAsImage: { title: t('echarts.saveAsImage') },
      },
    } : undefined,
  };

  const gridOption = showGrid
    ? { left: '3%', right: '4%', bottom: showLegend ? '15%' : '3%', top: (title ? '15%' : '3%'), containLabel: true }
    : null;

  const axisConfig = {
    xAxis: {
      type: 'category' as const, data: data.map((d) => d.label),
      axisLabel: { color: textColor, rotate: xAxisRotate },
      axisLine: { lineStyle: { color: splitLineColor } },
    },
    yAxis: {
      type: 'value' as const, axisLabel: { color: textColor },
      splitLine: { lineStyle: { color: splitLineColor } },
    },
  };

  const seriesData = data.map((d) => d.value);

  switch (type) {
    case 'line': {
      const opts = { ...baseOptions, ...axisConfig,
        series: [{ type: 'line', data: seriesData, smooth: true, symbol: 'circle',
          areaStyle: areaFill ? { color: { type: 'linear' as const, x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [{ offset: 0, color: themeColors[0] + '80' }, { offset: 1, color: themeColors[0] + '10' }] } } : undefined,
        }] };
      if (gridOption) (opts as any).grid = gridOption;
      return opts as EChartsOption;
    }
    case 'bar': {
      const opts = { ...baseOptions, ...axisConfig,
        series: [{ type: 'bar', data: seriesData, barWidth: '60%', itemStyle: { borderRadius: [4, 4, 0, 0] } }] };
      if (gridOption) (opts as any).grid = gridOption;
      return opts as EChartsOption;
    }
    case 'pie': {
      const pieData = data.map((d) => ({ name: d.label, value: d.value }));
      return { ...baseOptions,
        series: [{ type: 'pie', radius: [String(pieInnerRadius), pieRadius], center: ['50%', '50%'], data: pieData,
          label: { show: data.length <= 10, formatter: '{b}: {d}%', color: textColor } }] } as EChartsOption;
    }
    case 'gauge': {
      const gaugeValue = data[0]?.value || 0;
      const { min = 0, max = 100, splitNumber = 5 } = gaugeConfig;
      return { ...baseOptions,
        series: [{ type: 'gauge', startAngle: 90, endAngle: -270, radius: '90%', center: ['50%', '50%'],
          progress: { show: true, overlap: false, roundCap: true, clip: false, itemStyle: { color: themeColors[0] } },
          axisLine: { lineStyle: { width: 20, color: [[1, '#e0e0e0']] } },
          data: [{ value: gaugeValue, name: data[0]?.label || '' }],
          detail: { fontSize: 30, fontWeight: 'bold', color: themeColors[0], formatter: '{value}%', valueAnimation: true },
          min, max, splitNumber }] } as EChartsOption;
    }
    case 'radar': {
      const indicator = data.map((d) => ({ name: d.label, max: Math.max(...data.map((item) => item.value)) * 1.2 }));
      return { ...baseOptions,
        legend: showLegend ? { bottom: 10, textStyle: { color: textColor } } : undefined,
        radar: { indicator, axisName: { color: textColor }, splitLine: { lineStyle: { color: splitLineColor } },
          center: ['50%', '55%'] as string[], radius: '65%' },
        series: [{ type: 'radar' as const, data: [{ value: data.map((d) => d.value), name: '数据' }] }] } as EChartsOption;
    }
    case 'scatter': {
      const opts = { ...baseOptions,
        xAxis: { ...axisConfig.xAxis, splitLine: { show: false } },
        series: [{ type: 'scatter', data: data.map((d) => [d.label, d.value]), symbolSize: 12, itemStyle: { color: themeColors[0] } }] };
      if (gridOption) (opts as any).grid = gridOption;
      return opts as EChartsOption;
    }
    default:
      return baseOptions as EChartsOption;
  }
});

function handleClick(params: unknown) {
  emit('click', params);
}

// ============ 生命周期 ============

onMounted(() => {
  // ChartRenderer handles init
});

onBeforeUnmount(() => {
  // ChartRenderer handles cleanup
});

onUnmounted(() => {
  // Cleanup
});

defineExpose({});
</script>

<style scoped>
.echarts-wrapper { width: 100%; }
</style>
