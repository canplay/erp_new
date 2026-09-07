/**
 * @file ChartOptions.vue
 * @description 图表配置构建器 - 根据类型和参数生成 ECharts 配置
 * @date 2026-08-22
 */

<template>
  <!-- 仅逻辑组件，无模板 -->
</template>

<script setup lang="ts">
/**
 * @file ChartOptions.vue
 * @description 图表配置构建器 - 根据类型和参数生成 ECharts 配置
 * @date 2026-08-22
 */

import { useI18n } from 'vue-i18n';

const { t } = useI18n();

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

// ============ 主题颜色 ============

const darkThemeColors = [
  '#5470c6', '#91cc75', '#fac858', '#ee6666',
  '#73c0de', '#3ba272', '#fc8452', '#9a60b4', '#ea7ccc',
];

const lightThemeColors = [
  '#5470c6', '#73c0de', '#91cc75', '#fac858',
  '#ee6666', '#9a60b4', '#ea7ccc', '#3ba272', '#fc8452',
];

/**
 * @brief 构建图表配置
 */
export function buildChartOptions(
  config: EChartsConfig,
  isDark: boolean
): Record<string, unknown> {
  const {
    type = 'bar',
    title,
    data = [],
    color,
    showGrid = true,
    showLegend = true,
    showTooltip = true,
    showToolbox = false,
    xAxisRotate = 0,
    areaFill = false,
    pieInnerRadius = 0,
    pieRadius = '75%',
    gaugeConfig = {},
  } = config;

  const themeColors = isDark ? darkThemeColors : lightThemeColors;
  const textColor = isDark ? '#b0b0b0' : '#666666';
  const splitLineColor = isDark ? '#333333' : '#e0e0e0';

  // 基础配置
  const baseOptions: Record<string, unknown> = {
    backgroundColor: 'transparent',
    color: (color) || themeColors,
    title: title
      ? {
          text: title,
          left: 'center',
          textStyle: {
            color: textColor,
            fontSize: 16,
            fontWeight: 'bold',
          },
        }
      : undefined,
    tooltip: showTooltip
      ? {
          trigger: type === 'pie' || type === 'gauge' ? 'item' : 'axis',
          axisPointer: { type: 'shadow' },
          backgroundColor: isDark ? '#1e1e1e' : '#ffffff',
          borderColor: isDark ? '#333333' : '#e0e0e0',
          textStyle: {
            color: textColor,
          },
        }
      : undefined,
    legend: showLegend && data.length > 0
      ? {
          bottom: 10,
          textStyle: { color: textColor },
        }
      : undefined,
    toolbox: showToolbox
      ? {
          feature: {
            dataZoom: {
              title: {
                zoom: t('echarts.dataZoom'),
                back: t('echarts.dataZoomBack'),
              },
            },
            magicType: {
              title: {
                line: t('echarts.magicTypeLine'),
                bar: t('echarts.magicTypeBar'),
              },
            },
            restore: { title: t('echarts.restore') },
            saveAsImage: { title: t('echarts.saveAsImage') },
          },
        }
      : undefined,
  };

  // 根据类型构建特定配置
  switch (type) {
    case 'line': {
      const seriesData = data.map((d) => d.value);
      return {
        ...baseOptions,
        grid: showGrid
          ? { left: '3%', right: '4%', bottom: showLegend ? '15%' : '3%', top: title ? '15%' : '3%', containLabel: true }
          : undefined,
        xAxis: {
          type: 'category',
          data: data.map((d) => d.label),
          axisLabel: { color: textColor, rotate: xAxisRotate },
          axisLine: { lineStyle: { color: splitLineColor } },
        },
        yAxis: {
          type: 'value',
          axisLabel: { color: textColor },
          splitLine: { lineStyle: { color: splitLineColor } },
        },
        series: [
          {
            type: 'line',
            data: seriesData,
            smooth: true,
            symbol: 'circle',
            symbolSize: 8,
            lineStyle: { width: 2 },
            areaStyle: areaFill
              ? {
                  color: {
                    type: 'linear',
                    x: 0,
                    y: 0,
                    x2: 0,
                    y2: 1,
                    colorStops: [
                      { offset: 0, color: themeColors[0] + '80' },
                      { offset: 1, color: themeColors[0] + '10' },
                    ],
                  },
                }
              : undefined,
            itemStyle: { borderWidth: 2 },
          },
        ],
      };
    }

    case 'bar': {
      const seriesData = data.map((d) => d.value);
      return {
        ...baseOptions,
        grid: showGrid
          ? { left: '3%', right: '4%', bottom: showLegend ? '15%' : '3%', top: title ? '15%' : '3%', containLabel: true }
          : undefined,
        xAxis: {
          type: 'category',
          data: data.map((d) => d.label),
          axisLabel: { color: textColor, rotate: xAxisRotate },
          axisLine: { lineStyle: { color: splitLineColor } },
        },
        yAxis: {
          type: 'value',
          axisLabel: { color: textColor },
          splitLine: { lineStyle: { color: splitLineColor } },
        },
        series: [
          {
            type: 'bar',
            data: seriesData,
            barWidth: '60%',
            itemStyle: {
              borderRadius: [4, 4, 0, 0],
            },
          },
        ],
      };
    }

    case 'pie': {
      const pieData = data.map((d) => ({ name: d.label, value: d.value }));
      return {
        ...baseOptions,
        series: [
          {
            type: 'pie',
            radius: [String(pieInnerRadius), pieRadius],
            center: ['50%', '50%'],
            data: pieData,
            emphasis: {
              itemStyle: {
                shadowBlur: 10,
                shadowOffsetX: 0,
                shadowColor: 'rgba(0, 0, 0, 0.5)',
              },
            },
            label: {
              show: data.length <= 10,
              formatter: '{b}: {d}%',
              color: textColor,
            },
          },
        ],
      };
    }

    case 'gauge': {
      const gaugeValue = data[0]?.value || 0;
      const { min = 0, max = 100, splitNumber = 5 } = gaugeConfig;
      return {
        ...baseOptions,
        series: [
          {
            type: 'gauge',
            startAngle: 90,
            endAngle: -270,
            radius: '90%',
            center: ['50%', '50%'],
            pointer: { show: false },
            progress: {
              show: true,
              overlap: false,
              roundCap: true,
              clip: false,
              itemStyle: {
                color: themeColors[0],
              },
            },
            axisLine: {
              lineStyle: {
                width: 20,
                color: [[1, isDark ? '#333333' : '#e0e0e0']],
              },
            },
            splitLine: { show: false },
            axisTick: { show: false },
            axisLabel: { show: false },
            data: [{ value: gaugeValue, name: data[0]?.label || '' }],
            title: {
              fontSize: 14,
              color: textColor,
            },
            detail: {
              fontSize: 30,
              fontWeight: 'bold',
              color: themeColors[0],
              formatter: '{value}%',
              valueAnimation: true,
            },
            min,
            max,
            splitNumber,
          },
        ],
      };
    }

    case 'radar': {
      const indicator = data.map((d) => ({
        name: d.label,
        max: Math.max(...data.map((item) => item.value)) * 1.2,
      }));
      const seriesData = [
        {
          value: data.map((d) => d.value),
          name: '数据',
          areaStyle: { color: (themeColors[0] + '40') },
          lineStyle: { color: themeColors[0], width: 2 },
          itemStyle: { borderWidth: 2 },
        },
      ];
      return {
        ...baseOptions,
        legend: undefined,
        radar: {
          indicator,
          axisName: { color: textColor },
          splitLine: { lineStyle: { color: splitLineColor } },
          splitArea: { areaStyle: { color: [isDark ? '#1e1e1e' : '#ffffff'] } },
          center: ['50%', '55%'],
          radius: '65%',
        },
        series: [{ type: 'radar', data: seriesData }],
      };
    }

    case 'scatter': {
      const scatterData = data.map((d) => [d.label, d.value]);
      return {
        ...baseOptions,
        grid: showGrid
          ? { left: '3%', right: '4%', bottom: showLegend ? '15%' : '3%', top: title ? '15%' : '3%', containLabel: true }
          : undefined,
        xAxis: {
          type: 'category',
          data: data.map((d) => d.label),
          axisLabel: { color: textColor, rotate: xAxisRotate },
          axisLine: { lineStyle: { color: splitLineColor } },
          splitLine: { show: false },
        },
        yAxis: {
          type: 'value',
          axisLabel: { color: textColor },
          splitLine: { lineStyle: { color: splitLineColor } },
        },
        series: [
          {
            type: 'scatter',
            data: scatterData,
            symbolSize: 10,
            itemStyle: { borderWidth: 2 },
          },
        ],
      };
    }

    default: {
      return baseOptions;
    }
  }
}
</script>
