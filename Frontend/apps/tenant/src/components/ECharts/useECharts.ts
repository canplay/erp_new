import type { Ref } from 'vue';
import { onBeforeUnmount } from 'vue';
import { echarts, setupECharts } from '@/utils/echarts';
import type { ECharts } from '@/utils/echarts';

interface UseEChartsOptions {
  containerRef: Ref<HTMLDivElement | null>;
  height: string | number;
  onRender: (event: 'render', chart: unknown) => void;
}

export function useECharts(options: UseEChartsOptions) {
  let chart: ECharts | null = null;

  setupECharts();

  function initChart(): void {
    if (!options.containerRef.value) return;

    chart = echarts.init(options.containerRef.value);
    options.onRender('render', chart);
  }

  function setOptions(optionsData: Record<string, unknown>): void {
    if (!chart) return;
    chart.setOption(optionsData);
  }

  function destroyChart(): void {
    if (chart) {
      chart.dispose();
      chart = null;
    }
  }

  function resize(): void {
    chart?.resize();
  }

  onBeforeUnmount(() => {
    destroyChart();
  });

  return {
    initChart,
    setOptions,
    destroyChart,
    resize,
  };
}
