import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { BarChart, LineChart, PieChart } from 'echarts/charts';
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
  ToolboxComponent,
} from 'echarts/components';
import * as echarts from 'echarts/core';
import type { EChartsOption, EChartsType } from 'echarts/core';

let registered = false;
export function setupECharts(): void {
  if (registered) return;
  use([
    CanvasRenderer,
    BarChart,
    LineChart,
    PieChart,
    TitleComponent,
    TooltipComponent,
    GridComponent,
    LegendComponent,
    ToolboxComponent,
  ]);
  registered = true;
}

export { echarts, EChartsOption, type EChartsType as ECharts };
