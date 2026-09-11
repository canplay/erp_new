/**
 * @file ECharts.vue
 * @description 基于 ECharts 的图表组件 - 支持多种图表类型
 * @date 2026-04-03
 */

<template>
  <div ref="chartRef" class="echarts-container" :style="{ width: chartWidth, height: chartHeight }" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import {
  LineChart, BarChart, PieChart,
  GaugeChart, RadarChart, ScatterChart,
} from 'echarts/charts';
import {
  GridComponent, TooltipComponent, LegendComponent,
  TitleComponent, ToolboxComponent, DataZoomComponent,
  VisualMapComponent,
} from 'echarts/components';
import * as echarts from 'echarts/core';
import type { EChartsOption } from 'echarts';

// 注册 ECharts 组件（按需引入以减小打包体积）
use([
  CanvasRenderer, LineChart, BarChart, PieChart,
  GaugeChart, RadarChart, ScatterChart,
  GridComponent, TooltipComponent, LegendComponent,
  TitleComponent, ToolboxComponent, DataZoomComponent,
  VisualMapComponent,
]);

// ============ 类型定义 ============

export interface ChartDataItem {
  label: string;
  value: number;
  [key: string]: unknown;
}

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
  gaugeConfig?: {
    min?: number;
    max?: number;
    splitNumber?: number;
  };
}

// ============ Props ============

const { t } = useI18n();
const props = withDefaults(
  defineProps<{
    config?: EChartsConfig;
  }>(),
  {
    config: () => ({
      type: 'bar',
      data: [],
      width: '100%',
      height: '300px',
      showGrid: true,
      showLegend: true,
      showTooltip: true,
      showToolbox: false,
      areaFill: false,
    }),
  }
);

const emit = defineEmits<{
  (e: 'click', params: unknown): void;
  (e: 'ready', instance: unknown): void;
}>();

// ============ 响应式状态 ============

const chartRef = ref<HTMLElement | null>(null);
const isDark = ref(false);
let chartInstance: echarts.ECharts | null = null;

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

// ============ 监听主题变化 ============

function checkDarkMode() {
  isDark.value = document.body.classList.contains('body--dark');
  updateChartTheme();
}

function updateChartTheme() {
  if (chartInstance) {
    chartInstance.dispose();
    initChart();
  }
}

// ============ 图表初始化 ============

function initChart() {
  if (!chartRef.value) return;

  chartInstance = echarts.init(chartRef.value, isDark.value ? 'dark' : undefined);
  const options = buildChartOptions();
  chartInstance.setOption(options);

  chartInstance.on('click', (params: unknown) => {
    emit('click', params);
  });

  emit('ready', chartInstance);
}

function buildChartOptions(): EChartsOption {
  const {
    type = 'bar', title, data = [], color,
    showGrid = true, showLegend = true, showTooltip = true,
    showToolbox = false, xAxisRotate = 0, areaFill = false,
    pieInnerRadius = 0, pieRadius = '75%', gaugeConfig = {},
  } = props.config;

  const themeColors = isDark.value
    ? ['#5470c6','#91cc75','#fac858','#ee6666','#73c0de','#3ba272','#fc8452','#9a60b4','#ea7ccc']
    : ['#5470c6','#73c0de','#91cc75','#fac858','#ee6666','#9a60b4','#ea7ccc','#3ba272','#fc8452'];
  const textColor = isDark.value ? '#b0b0b0' : '#666666';
  const splitLineColor = isDark.value ? '#333333' : '#e0e0e0';

  const baseOptions: EChartsOption = {
    backgroundColor: 'transparent',
    color: (color as string[]) || themeColors,
  };

  if (title) {
    baseOptions.title = {
      text: title,
      left: 'center' as const,
      textStyle: { color: textColor, fontSize: 16, fontWeight: 'bold' } as const,
    };
  }
  if (showTooltip) {
    const tooltipOpt: EChartsOption['tooltip'] = {
      trigger: type === 'pie' || type === 'gauge' ? 'item' : 'axis',
      axisPointer: { type: 'shadow' } as const,
      backgroundColor: isDark.value ? '#1e1e1e' : '#ffffff',
      borderColor: isDark.value ? '#333333' : '#e0e0e0',
      textStyle: { color: textColor } as const,
    };
    baseOptions.tooltip = tooltipOpt;
  }
  if (showLegend && data.length > 0) {
    const legendOpt: EChartsOption['legend'] = { bottom: 10, textStyle: { color: textColor } } as const;
    baseOptions.legend = legendOpt;
  }
  if (showToolbox) {
    baseOptions.toolbox = {
      feature: {
        dataZoom: { title: { zoom: t('echarts.dataZoom'), back: t('echarts.dataZoomBack') } },
        magicType: { title: { line: t('echarts.magicTypeLine'), bar: t('echarts.magicTypeBar') } },
        restore: { title: t('echarts.restore') },
        saveAsImage: { title: t('echarts.saveAsImage') },
      },
    };
  }

  let grid: EChartsOption['grid'] = undefined;
  if (showGrid) {
    grid = { left: '3%', right: '4%', bottom: showLegend ? '15%' : '3%', top: title ? '15%' : '3%', containLabel: true };
  }

  const axisBase = {
    axisLabel: { color: textColor, rotate: xAxisRotate } as Record<string, unknown>,
    axisLine: { lineStyle: { color: splitLineColor } },
  };
  const yAxisBase = {
    type: 'value' as const,
    axisLabel: { color: textColor },
    splitLine: { lineStyle: { color: splitLineColor } },
  };

  switch (type) {
    case 'line': {
      const seriesData = data.map((d) => d.value);
      const seriesItem = {
        type: 'line' as const, data: seriesData, smooth: true, symbol: 'circle' as const,
        symbolSize: 8, lineStyle: { width: 2 } as const,
        ...(areaFill ? { areaStyle: { color: { type: 'linear' as const, x: 0, y: 0, x2: 0, y2: 1, colorStops: [
            { offset: 0, color: themeColors[0] + '80' },
            { offset: 1, color: themeColors[0] + '10' },
          ]} } } : {}),
        itemStyle: { borderWidth: 2 } as const,
      };
      const opts: EChartsOption = {
        ...baseOptions,
        ...(grid !== undefined && { grid }),
        xAxis: { type: 'category' as const, data: data.map((d) => d.label), ...axisBase },
        yAxis: yAxisBase,
        series: [seriesItem],
      };
      return opts;
    }
    case 'bar': {
      const seriesData = data.map((d) => d.value);
      const opts: EChartsOption = {
        ...baseOptions,
        ...(grid !== undefined && { grid }),
        xAxis: { type: 'category' as const, data: data.map((d) => d.label), ...axisBase },
        yAxis: yAxisBase,
        series: [{
          type: 'bar' as const, data: seriesData, barWidth: '60%',
          itemStyle: { borderRadius: [4, 4, 0, 0] as [number, number, number, number] },
        }],
      };
      return opts;
    }
    case 'pie': {
      const pieData = data.map((d) => ({ name: d.label, value: d.value }));
      const opts: EChartsOption = {
        ...baseOptions,
        series: [{
          type: 'pie' as const,
          radius: [String(pieInnerRadius), pieRadius] as [string, string],
          center: ['50%', '50%'] as [string, string],
          data: pieData,
          emphasis: { itemStyle: { shadowBlur: 10, shadowOffsetX: 0, shadowColor: 'rgba(0, 0, 0, 0.5)' } } as const,
          label: { show: data.length <= 10, formatter: '{b}: {d}%', color: textColor } as const,
        }],
      };
      return opts;
    }
    case 'gauge': {
      const gaugeValue = data[0]?.value || 0;
      const { min = 0, max = 100, splitNumber = 5 } = gaugeConfig;
      const opts: EChartsOption = {
        ...baseOptions,
        series: [{
          type: 'gauge' as const, startAngle: 90, endAngle: -270, radius: '90%', center: ['50%', '50%'] as [string, string],
          pointer: { show: false } as const,
          progress: { show: true, overlap: false, roundCap: true, clip: false, itemStyle: { color: themeColors[0] as string } } as const,
          axisLine: { lineStyle: { width: 20, color: [[1, isDark.value ? '#333333' : '#e0e0e0'] as [number, string]] } } as const,
          splitLine: { show: false } as const, axisTick: { show: false } as const, axisLabel: { show: false } as const,
          data: [{ value: gaugeValue, name: data[0]?.label || '' }],
          title: { fontSize: 14, color: textColor } as const,
          detail: { fontSize: 30, fontWeight: 'bold' as const, color: themeColors[0] as string, formatter: '{value}%', valueAnimation: true } as const,
          min, max, splitNumber,
        }],
      };
      return opts;
    }
    case 'radar': {
      const indicator = data.map((d) => ({ name: d.label, max: Math.max(...data.map((item) => item.value)) * 1.2 }));
      const seriesData = [{
        value: data.map((d) => d.value), name: '数据',
        areaStyle: { color: themeColors[0] + '40' } as const,
        lineStyle: { color: themeColors[0] as string, width: 2 } as const,
        itemStyle: { borderWidth: 2 } as const,
      }];
      const opts: EChartsOption = {
        ...baseOptions,
        ...(grid !== undefined && { grid }),
        legend: showLegend && data.length > 0
          ? ({ bottom: 10, textStyle: { color: textColor } } as const)
          : (undefined as unknown as NonNullable<EChartsOption['legend']>),
        radar: {
          indicator,
          axisName: { color: textColor } as const,
          splitLine: { lineStyle: { color: splitLineColor } } as const,
          splitArea: { areaStyle: { color: [isDark.value ? '#1e1e1e' : '#ffffff'] } } as const,
          center: ['50%', '55%'] as [string, string], radius: '65%',
        } as const,
        series: [{ type: 'radar' as const, data: seriesData }],
      };
      return opts;
    }
    case 'scatter': {
      const scatterData = data.map((d) => [d.label, d.value]);
      const opts: EChartsOption = {
        ...baseOptions,
        ...(grid !== undefined && { grid }),
        xAxis: { type: 'category' as const, data: data.map((d) => d.label), ...axisBase, splitLine: { show: false } as const },
        yAxis: yAxisBase,
        series: [{ type: 'scatter' as const, data: scatterData, symbolSize: 10, itemStyle: { borderWidth: 2 } as const }],
      };
      return opts;
    }
    default: return baseOptions;
  }
}

// ============ 生命周期 ============

onMounted(() => {
  checkDarkMode();
  initChart();

  const observer = new MutationObserver((mutations) => {
    for (const m of mutations) {
      if (m.attributeName === 'class') {
        checkDarkMode();
        return;
      }
    }
  });
  observer.observe(document.body, { attributes: true, attributeFilter: ['class'] });

  const resizeObserver = new ResizeObserver(() => {
    if (chartInstance) {
      chartInstance.resize();
    }
  });
  if (chartRef.value) {
    resizeObserver.observe(chartRef.value);
  }
});

onUnmounted(() => {
  chartInstance?.dispose();
  chartInstance = null;
});

onBeforeUnmount(() => {
  chartInstance?.dispose();
  chartInstance = null;
});
</script>

<style scoped>
.echarts-container {
  width: 100%;
  min-height: 300px;
  border-radius: 8px;
  background: transparent;
}
</style>
