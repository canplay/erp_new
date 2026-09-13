<template>
  <div class="q-mb-md">
    <div class="text-subtitle1 q-mb-sm">{{ i18nT('raw.s_cert_verify') }}</div>
    <div class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-card flat bordered>
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s_cert_status_dist') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="certOption" :option="certOption" height="240px" />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card flat bordered>
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s_cert_status_bar') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="statusOption" :option="statusOption" height="240px" />
          </q-card-section>
        </q-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { computed } from 'vue';
import { EChartBase } from '@erp-new-frontend-monorepo/components';

interface PieDatum {
  name: string;
  value: number;
}
interface BarDatum {
  name: string;
  value: number;
}

const props = defineProps<{
  certData: PieDatum[];
  statusData: BarDatum[];
}>();

const certOption = computed<Record<string, unknown> | null>(() =>
  props.certData.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: ['#1565c0', '#42a5f5', '#90caf9', '#bbdefb'],
        series: [
          {
            name: i18nT('raw.sddc4d5'),
            type: 'pie',
            radius: ['38%', '66%'],
            center: ['50%', '44%'],
            itemStyle: { borderRadius: 4, borderColor: '#fff', borderWidth: 2 },
            label: { formatter: '{b}\n{c}', fontSize: 12, color: '#444' },
            data: props.certData,
          },
        ],
      }
    : null,
);

const statusOption = computed<Record<string, unknown> | null>(() =>
  props.statusData.length
    ? {
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
        xAxis: { type: 'category', data: props.statusData.map((d) => d.name) },
        yAxis: { type: 'value' },
        series: [
          {
            name: i18nT('raw.sc332d3'),
            type: 'bar',
            data: props.statusData.map((d) => d.value),
            itemStyle: { color: '#1565c0', borderRadius: [4, 4, 0, 0] },
            barWidth: '50%',
          },
        ],
      }
    : null,
);
</script>
