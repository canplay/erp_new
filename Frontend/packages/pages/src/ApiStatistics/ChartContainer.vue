<template>
  <div class="chart-container">
    <q-card class="chart-container__card">
      <q-card-section>
        <div class="text-h6">{{ title }}</div>
      </q-card-section>
      <q-card-section>
        <div ref="chartRef" class="chart-container__chart" />
      </q-card-section>
    </q-card>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart, LineChart, PieChart } from 'echarts/charts'
import { TitleComponent, TooltipComponent, GridComponent, LegendComponent } from 'echarts/components'
import type { ECharts } from 'echarts/core'

use([CanvasRenderer, BarChart, LineChart, PieChart, TitleComponent, TooltipComponent, GridComponent, LegendComponent])

interface Props {
  title: string
  chartData: Record<string, unknown>
}

const props = defineProps<Props>()

const chartRef = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

function initChart(): void {
  if (!chartRef.value) return
  chart = echarts.init(chartRef.value)
  setChartOptions()
}

function setChartOptions(): void {
  if (!chart) return
  chart.setOption(props.chartData)
}

watch(() => props.chartData, () => {
  setChartOptions()
})

onMounted(() => {
  initChart()
})

onBeforeUnmount(() => {
  chart?.dispose()
})
</script>

<style scoped>
.chart-container__card {
  min-height: 300px;
}

.chart-container__chart {
  width: 100%;
  height: 300px;
}
</style>
