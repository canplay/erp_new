<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.s273219') }}</div>

    <!-- 合规评估卡 -->
    <q-card flat bordered erp-card class="q-mb-md">
      <q-card-section>
        <div class="row items-center">
          <div class="text-subtitle1">{{ i18nT('raw.s92544a') }}</div>
          <q-space />
          <RiskGauge :score="complianceScore" :level="complianceLevel" />
        </div>
      </q-card-section>
    </q-card>

    <div class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-card flat bordered erp-card>
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s17bb74') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase
              v-if="typeOption"
              :option="typeOption"
              height="300px"
              @click="onTypeClick"
            />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s2f12a6')"
              :hint="i18nT('raw.sd5bd70')"
            />
          </q-card-section>
        </q-card>
      </div>

      <div class="col-12 col-md-6">
        <q-card flat bordered erp-card>
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s677305') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase
              v-if="dangerOption"
              :option="dangerOption"
              height="300px"
              @click="onDangerClick"
            />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.scb1faf')"
              :hint="i18nT('raw.se1f06c')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <div class="row q-col-gutter-md q-mt-md">
      <div class="col-12 col-md-6">
        <q-card flat bordered erp-card>
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.sc25746') }}</div>
          </q-card-section>
          <q-card-section>
            <IssueWordCloud :issues="wordCloudIssues" />
          </q-card-section>
        </q-card>
      </div>

      <div class="col-12 col-md-6">
        <q-card flat bordered erp-card>
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s471784') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="healthOption" :option="healthOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s38fc64')"
              :hint="i18nT('raw.s1eee30')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <AnalyticsDialogs ref="dialogsRef" />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { deviceApi, dangerApi, weeklyApi, analyticsApi } from '@erp-new-frontend-monorepo/api';
import type {
  Device,
  HiddenDanger,
  IssueAnalysis,
  PagedResult,
  ComplianceFactor,
  EChartEventParams,
} from '@erp-new-frontend-monorepo/types';
import {
  EChartBase,
  CHART_PALETTE,
  EmptyState,
  IssueWordCloud,
  RiskGauge,
} from '@erp-new-frontend-monorepo/components';
import { deviceTypeText, dangerStatusText } from '@erp-new-frontend-monorepo/utils';
import AnalyticsDialogs from './AnalyticsDialogs.vue';

const $q = useQuasar();

// ── 合规评估 ──
const complianceScore = ref(0);
const complianceLevel = ref(i18nT('raw.sc1dfbc'));
const complianceFactors = ref<ComplianceFactor[]>([]);

function levelOfScore(score: number): string {
  if (score >= 80) return i18nT('raw.s188468');
  if (score >= 60) return i18nT('raw.s554c69');
  if (score >= 30) return i18nT('raw.sd15a8b');
  return i18nT('raw.sc1dfbc');
}

// ── 图表数据 ──
interface PieDatum {
  name: string;
  value: number;
  raw: string;
}
interface BarDatum {
  name: string;
  value: number;
}

const typeData = ref<PieDatum[]>([]);
const healthData = ref<BarDatum[]>([]);
const dangerData = ref<PieDatum[]>([]);

function aggregate<T extends object>(
  items: T[],
  field: keyof T & string,
  labeler: (k: string) => string,
): PieDatum[] {
  const counter = new Map<string, number>();
  for (const it of items) {
    const key = String(it?.[field] ?? 'other');
    counter.set(key, (counter.get(key) || 0) + 1);
  }
  return [...counter.entries()].map(([k, v]) => ({ name: labeler(k) || k, value: v, raw: k }));
}

function healthLevel(score: number): string {
  if (score >= 85) return 'A';
  if (score >= 70) return 'B';
  if (score >= 55) return 'C';
  return 'D';
}

function aggregateHealth(items: Device[]): BarDatum[] {
  const levels = [
    { name: i18nT('raw.s3c8ffa'), value: 0 },
    { name: i18nT('raw.s6204f2'), value: 0 },
    { name: i18nT('raw.s38fc64'), value: 0 },
    { name: i18nT('raw.s1eee30'), value: 0 },
  ];
  for (const d of items) {
    const level = healthLevel(d.healthScore ?? 0);
    if (level === 'A') levels[0]!.value++;
    else if (level === 'B') levels[1]!.value++;
    else if (level === 'C') levels[2]!.value++;
    else levels[3]!.value++;
  }
  return levels;
}

const typeOption = computed<Record<string, unknown> | null>(() =>
  typeData.value.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: CHART_PALETTE.blues,
        series: [
          {
            name: i18nT('raw.s7b4c61'),
            type: 'pie',
            radius: ['38%', '66%'],
            center: ['50%', '44%'],
            itemStyle: { borderRadius: 4, borderColor: '#fff', borderWidth: 2 },
            label: { formatter: '{b}\n{c}', fontSize: 12, color: '#444' },
            data: typeData.value,
          },
        ],
      }
    : null,
);

const healthOption = computed<Record<string, unknown> | null>(() =>
  healthData.value.length
    ? {
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
        xAxis: { type: 'category', data: healthData.value.map((d) => d.name) },
        yAxis: { type: 'value' },
        series: [
          {
            name: i18nT('raw.sf2aa3a'),
            type: 'bar',
            data: healthData.value.map((d) => d.value),
            itemStyle: { color: '#1565c0', borderRadius: [4, 4, 0, 0] },
            barWidth: '50%',
          },
        ],
      }
    : null,
);

const dangerOption = computed<Record<string, unknown> | null>(() =>
  dangerData.value.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: [...CHART_PALETTE.blues.slice(0, 4), '#0d47a1'],
        series: [
          {
            name: i18nT('raw.sd678ba'),
            type: 'pie',
            radius: ['38%', '66%'],
            center: ['50%', '44%'],
            itemStyle: { borderRadius: 4, borderColor: '#fff', borderWidth: 2 },
            label: { formatter: '{b}\n{c}', fontSize: 12, color: '#444' },
            data: dangerData.value,
          },
        ],
      }
    : null,
);

// ── 问题溯源 ──
const wordCloudIssues = ref<{ word: string; count: number }[]>([]);

// ── 下钻弹窗 ──
const dialogsRef = ref<InstanceType<typeof AnalyticsDialogs> | null>(null);

function onTypeClick(params: EChartEventParams) {
  const raw = params?.data?.raw as string | undefined;
  if (!raw) return;
  $q.notify({
    type: 'info',
    message: `${i18nT('raw.sanl_dev_drill', { name: deviceTypeText(raw) })}`,
    timeout: 2000,
  });
}

function onDangerClick(params: EChartEventParams) {
  const raw = params?.data?.raw as string | undefined;
  if (!raw) return;
  $q.notify({
    type: 'info',
    message: `${i18nT('raw.sanl_dan_drill', { name: dangerStatusText(raw) })}`,
    timeout: 2000,
  });
}

// ── 初始化 ──
const devices = ref<Device[]>([]);
const dangers = ref<HiddenDanger[]>([]);

onMounted(async () => {
  try {
    const res = (await deviceApi.list(1, 100)) as PagedResult<Device>;
    devices.value = res.items;
    typeData.value = aggregate(res.items, 'type', deviceTypeText);
    healthData.value = aggregateHealth(res.items);
  } catch {
    devices.value = [];
    $q.notify({ type: 'negative', message: i18nT('raw.sde3ea5'), timeout: 4000 });
  }

  try {
    const res = (await dangerApi.list(1, 50)) as PagedResult<HiddenDanger>;
    dangers.value = res.items;
    dangerData.value = aggregate(res.items, 'status', dangerStatusText);
  } catch {
    dangers.value = [];
  }

  try {
    const res = await analyticsApi.complianceAssessment();
    complianceScore.value = res?.score ?? 0;
    complianceLevel.value = levelOfScore(complianceScore.value);
    complianceFactors.value = res?.factors ?? [];
  } catch {
    complianceScore.value = 0;
    complianceFactors.value = [];
  }

  try {
    const analysis: IssueAnalysis | null = await weeklyApi.analysis();
    if (analysis?.wordCloud?.length) {
      wordCloudIssues.value = analysis.wordCloud.map((w) => ({ word: w.text, count: w.weight }));
    }
  } catch {
    wordCloudIssues.value = [];
  }
});
</script>
