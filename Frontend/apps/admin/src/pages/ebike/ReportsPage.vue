<template>
  <q-page class="q-pa-md">
    <div class="row">
      <q-card class="col" style="background-color: rgba(0, 0, 0, 0.5); backdrop-filter: blur(2px)">
        <q-card-section>
          <div ref="echarts1Ref" style="width: 100%; height: 400px" />
        </q-card-section>
      </q-card>
      <div class="col-auto" style="width: 10px" />
      <q-card class="col" style="background-color: rgba(0, 0, 0, 0.5); backdrop-filter: blur(2px)">
        <q-card-section>
          <div ref="echarts4Ref" style="width: 100%; height: 400px" />
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { date } from 'quasar';
import { useI18n } from 'vue-i18n';
import * as echarts from 'echarts';
import { httpClient } from '@/utils/alova';
import { useEbikeStore as useStore } from '@/stores/ebike';
import { parseCarResponse } from '@/utils/ebike';

const { t: $t } = useI18n();

const store = useStore();

const echarts1Ref = ref<HTMLDivElement | null>(null);
const echarts4Ref = ref<HTMLDivElement | null>(null);
let echarts1Instance: echarts.ECharts | null = null;
let echarts4Instance: echarts.ECharts | null = null;
let chart4Interval: ReturnType<typeof setInterval> | null = null;

async function draw(): Promise<void> {
  await initEchart1();
  initEchart4();
}

async function initEchart1(): Promise<void> {
  if (!echarts1Ref.value) return;

  const s = await httpClient.post(store.backend.private + '/storage', {
    method: 'query',
    code: '',
    provide: '',
    status: -1,
  });

  const storageData = parseCarResponse(s) as Array<Record<string, unknown>>;
  const pieData: Array<{ value: number; name: string }> = [];
  for (const element of storageData) {
    const codeVal = element?.code;
    if (!codeVal || typeof codeVal !== 'string' || codeVal === '') continue;

    const totalCap = Number(element.total_capacity) || 0;
    pieData.push({
      value: totalCap,
      name: codeVal,
    });
  }

  echarts1Instance = echarts.init(echarts1Ref.value);
  echarts1Instance.setOption({
    title: {
      text: $t('ebike.reportTitle'),
      left: 'center',
      textStyle: { color: '#ffffff' },
    },
    tooltip: { trigger: 'item' },
    series: [
      {
        name: '区域',
        type: 'pie',
        radius: '50%',
        data: pieData,
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
      },
    ],
  });
}

function initEchart4(): void {
  if (!echarts4Ref.value) return;

  echarts4Instance = echarts.init(echarts4Ref.value);

  const now = Date.now();
  const xLabels: string[] = [];
  const t = (key: string) => $t(key);
  const seriesData: Record<string, number[]> = {
    [t('ebike.providerXiqi')]: [],
    [t('ebike.providerHeluo')]: [],
    [t('ebike.providerMeituan')]: [],
    [t('ebike.revenue')]: [],
  };
  for (let i = 0; i < 7; i++) {
    const timeStr = date.formatDate(now - (6 - i) * 86400000, 'YYYY-MM-DD H:mm:ss');
    xLabels.push(timeStr);
    const base = Math.random() * 100;
    (seriesData[$t('ebike.providerXiqi')] ??= []).push(Math.floor(base * 0.3));
    (seriesData[$t('ebike.providerHeluo')] ??= []).push(Math.floor(base * 0.4));
    (seriesData[$t('ebike.providerMeituan')] ??= []).push(Math.floor(base * 0.2));
    (seriesData[$t('ebike.revenue')] ??= []).push(
      Math.floor(base * 0.3) + Math.floor(base * 0.4) + Math.floor(base * 0.2),
    );
  }

  const option: echarts.EChartsOption = {
    tooltip: {
      trigger: 'axis',
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: xLabels,
      axisLine: { lineStyle: { color: 'white' } },
    },
    yAxis: {
      type: 'value',
      name: '元',
      boundaryGap: [0, '100%'],
      axisLine: { lineStyle: { color: 'white' } },
    },
    legend: {
      data: [$t('ebike.providerXiqi'), $t('ebike.providerHeluo'), $t('ebike.providerMeituan'), $t('ebike.revenue')],
      textStyle: { color: 'white' },
    },
    series: [
      { name: $t('ebike.providerXiqi'), type: 'line', data: seriesData[$t('ebike.providerXiqi')] ?? [] },
      { name: $t('ebike.providerHeluo'), type: 'line', data: seriesData[$t('ebike.providerHeluo')] ?? [] },
      { name: $t('ebike.providerMeituan'), type: 'line', data: seriesData[$t('ebike.providerMeituan')] ?? [] },
      { name: $t('ebike.revenue'), type: 'line', data: seriesData[$t('ebike.revenue')] ?? [] },
    ],
  };

  echarts4Instance.setOption(option);

  if (chart4Interval !== null) {
    clearInterval(chart4Interval);
  }
  chart4Interval = setInterval(() => {
    void (async () => {
      if (!echarts4Instance) return;

      const resp = await httpClient.post(store.backend.private + '/order', {
        method: 'query',
        code: '',
        provide: '',
        status: -1,
        time: { start: '', end: '' },
      });
      const orders = parseCarResponse(resp) as Array<Record<string, unknown>>;

      const revenueByProvider: Record<string, number> = {
        [$t('ebike.providerXiqi')]: 0,
        [$t('ebike.providerHeluo')]: 0,
        [$t('ebike.providerMeituan')]: 0,
      };
      for (const order of orders) {
        const provider = order?.provide as string | undefined;
        const speed = Number(order?.speed) || 0;
        if (provider === $t('ebike.providerXiqi')) {
          const v = revenueByProvider[$t('ebike.providerXiqi')] || 0;
          revenueByProvider[$t('ebike.providerXiqi')] = v + speed;
        } else if (provider === $t('ebike.providerHeluo')) {
          const v = revenueByProvider[$t('ebike.providerHeluo')] || 0;
          revenueByProvider[$t('ebike.providerHeluo')] = v + speed;
        } else if (provider === $t('ebike.providerMeituan')) {
          const v = revenueByProvider[$t('ebike.providerMeituan')] || 0;
          revenueByProvider[$t('ebike.providerMeituan')] = v + speed;
        }
      }

      const currentTime = date.formatDate(Date.now(), 'YYYY-MM-DD H:mm:ss');

      const xqVal = revenueByProvider[$t('ebike.providerXiqi')] || 0;
      const hlVal = revenueByProvider[$t('ebike.providerHeluo')] || 0;
      const mtVal = revenueByProvider[$t('ebike.providerMeituan')] || 0;

      const newSeriesData: Record<string, number[]> = {
        [$t('ebike.providerXiqi')]: (seriesData[$t('ebike.providerXiqi')] ?? []).slice(1).concat([xqVal]),
        [$t('ebike.providerHeluo')]: (seriesData[$t('ebike.providerHeluo')] ?? []).slice(1).concat([hlVal]),
        [$t('ebike.providerMeituan')]: (seriesData[$t('ebike.providerMeituan')] ?? []).slice(1).concat([mtVal]),
        [$t('ebike.revenue')]: (seriesData[$t('ebike.revenue')] ?? []).slice(1).concat([xqVal + hlVal + mtVal]),
      };

      const newXLabels: string[] = xLabels.slice(1).concat([currentTime]);

      echarts4Instance.setOption({
        xAxis: { data: newXLabels },
        series: [
          { data: newSeriesData[$t('ebike.providerXiqi')] ?? [] },
          { data: newSeriesData[$t('ebike.providerHeluo')] ?? [] },
          { data: newSeriesData[$t('ebike.providerMeituan')] ?? [] },
          { data: newSeriesData[$t('ebike.revenue')] ?? [] },
        ],
      });
    });
  }, 5000);
}

function disposeCharts(): void {
  if (chart4Interval !== null) {
    clearInterval(chart4Interval);
    chart4Interval = null;
  }
  echarts1Instance?.dispose();
  echarts1Instance = null;
  echarts4Instance?.dispose();
  echarts4Instance = null;
  console.debug('[ReportsPage] eCharts instances disposed');
}

onMounted(() => {
  void draw();
});

onBeforeUnmount(() => {
  disposeCharts();
});
</script>