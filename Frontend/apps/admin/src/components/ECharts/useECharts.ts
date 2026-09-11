import type { Ref} from 'vue';
import { onBeforeUnmount } from 'vue'
import * as echarts from 'echarts'

interface UseEChartsOptions {
  containerRef: Ref<HTMLDivElement | null>
  height: string | number
  onRender: (event: 'render', chart: unknown) => void
}

export function useECharts(options: UseEChartsOptions) {
  let chart: echarts.ECharts | null = null

  function initChart(): void {
    if (!options.containerRef.value) return

    chart = echarts.init(options.containerRef.value)
    options.onRender('render', chart)
  }

  function setOptions(optionsData: Record<string, unknown>): void {
    if (!chart) return
    chart.setOption(optionsData)
  }

  function destroyChart(): void {
    if (chart) {
      chart.dispose()
      chart = null
    }
  }

  function resize(): void {
    chart?.resize()
  }

  onBeforeUnmount(() => {
    destroyChart()
  })

  return {
    initChart,
    setOptions,
    destroyChart,
    resize
  }
}
