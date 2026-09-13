<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.sbd00ff') }}</div>

    <!-- KPI 卡片 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-3" v-for="k in kpiItems" :key="k.label">
        <StatCard :label="k.label" :value="k.value" :icon="k.icon" :color="k.color" />
      </div>
    </div>

    <!-- 图表区 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s2367b4') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="levelOption" :option="levelOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.scb1faf')"
              :hint="i18nT('raw.sfa70c8')"
            />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s677305') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="statusOption" :option="statusOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.scb1faf')"
              :hint="i18nT('raw.sfa70c8')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 重大隐患自动上报统计 -->
    <q-card flat bordered class="erp-card q-mb-lg">
      <q-card-section class="q-pb-none">
        <div class="text-subtitle1">{{ i18nT('raw.sb41c57') }}</div>
      </q-card-section>
      <q-card-section>
        <div class="row q-gutter-md">
          <q-badge color="red" class="text-body2 q-pa-sm"> 重大隐患：{{ majorCount }} 条 </q-badge>
          <q-badge color="orange" class="text-body2 q-pa-sm">
            紧急隐患：{{ emergencyCount }} 条
          </q-badge>
          <q-badge color="green" class="text-body2 q-pa-sm">
            已自动上报：{{ reportedCount }} 条
          </q-badge>
        </div>
      </q-card-section>
    </q-card>

    <!-- 隐患列表 -->
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">{{ i18nT('raw.sebf8e5') }}</div>
        <PageTable
          :rows="rows"
          :columns="columns"
          :loading="loading"
          :total="total"
          :dense="$q.screen.lt.md"
          selection-mode
          @request="onRequest"
        >
          <template #selection-actions="{ selected, clear }">
            <q-btn
              v-if="selected.length > 0"
              color="primary"
              outline
              icon="file_download"
              :label="i18nT('raw.s1b06cd')"
              @click="exportSelected(selected as HiddenDanger[], clear)"
            />
          </template>
          <template #body-cell-level="{ row }">
            <q-td>
              <DangerLevelTag :level="row.level" />
            </q-td>
          </template>
        </PageTable>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { dangerApi } from '@erp-new-frontend-monorepo/api';
import type { HiddenDanger, PagedResult } from '@erp-new-frontend-monorepo/types';
import {
  PageTable,
  DangerLevelTag,
  EChartBase,
  CHART_PALETTE,
  EmptyState,
  StatCard,
} from '@erp-new-frontend-monorepo/components';
import {
  formatDate,
  dangerStatusText,
  dangerLevelText,
  exportToXlsx,
} from '@erp-new-frontend-monorepo/utils';

const $q = useQuasar();

// ── 数据 ──
const rows = ref<HiddenDanger[]>([]);
const total = ref(0);
const loading = ref(false);
const allDangers = ref<HiddenDanger[]>([]);

const lv = (d: HiddenDanger) => ((d.level as string) ?? '').toLowerCase();
const sv = (d: HiddenDanger) => ((d.status as string) ?? '').toLowerCase();

const kpiItems = computed(() => [
  {
    label: i18nT('raw.s3b8d4d'),
    value: total.value || 0,
    icon: 'warning',
    color: 'orange' as const,
  },
  {
    label: i18nT('raw.s885c6f'),
    value: allDangers.value.filter((d) => lv(d) === 'general').length,
    icon: 'info',
    color: 'blue' as const,
  },
  { label: i18nT('raw.sdaad5f'), value: majorCount.value, icon: 'error', color: 'red' as const },
  {
    label: i18nT('raw.s1fb93d'),
    value: allDangers.value.filter((d) => sv(d) === 'closed').length,
    icon: 'schedule',
    color: 'red' as const,
  },
]);

const majorCount = computed(() => allDangers.value.filter((d) => lv(d) === 'major').length);
const emergencyCount = computed(() => allDangers.value.filter((d) => lv(d) === 'emergency').length);
const reportedCount = computed(
  () => allDangers.value.filter((d) => lv(d) === 'major' || lv(d) === 'emergency').length,
);

// ── 图表数据 ──
interface PieDatum {
  name: string;
  value: number;
}

const levelData = ref<PieDatum[]>([]);
const statusData = ref<PieDatum[]>([]);

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
  return [...counter.entries()].map(([k, v]) => ({ name: labeler(k) || k, value: v }));
}

const levelOption = computed<Record<string, unknown> | null>(() =>
  levelData.value.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: CHART_PALETTE.reds,
        series: [
          {
            name: i18nT('raw.s1952ea'),
            type: 'pie',
            radius: ['38%', '66%'],
            center: ['50%', '44%'],
            itemStyle: { borderRadius: 4, borderColor: '#fff', borderWidth: 2 },
            label: { formatter: '{b}\n{c}', fontSize: 12, color: '#444' },
            data: levelData.value,
          },
        ],
      }
    : null,
);

const statusOption = computed<Record<string, unknown> | null>(() =>
  statusData.value.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: ['#c62828', '#ef5350', '#ef9a9a', '#ffcdd2', '#b71c1c'],
        series: [
          {
            name: i18nT('raw.sd678ba'),
            type: 'pie',
            radius: ['38%', '66%'],
            center: ['50%', '44%'],
            itemStyle: { borderRadius: 4, borderColor: '#fff', borderWidth: 2 },
            label: { formatter: '{b}\n{c}', fontSize: 12, color: '#444' },
            data: statusData.value,
          },
        ],
      }
    : null,
);

// ── 列表 ──
const columns = [
  { name: 'code', label: i18nT('raw.s667f13'), field: 'deviceId', align: 'left' as const },
  {
    name: 'description',
    label: i18nT('raw.sd79448'),
    field: 'description',
    align: 'left' as const,
  },
  { name: 'level', label: i18nT('raw.s509e53'), field: 'level', align: 'center' as const },
  {
    name: 'status',
    label: i18nT('raw.s9fb403'),
    field: 'status',
    align: 'center' as const,
    format: (v: string) => dangerStatusText(v),
  },
  {
    name: 'createdAt',
    label: i18nT('raw.s1d36cd'),
    field: 'createTime',
    align: 'left' as const,
    format: (v: unknown) => (v ? formatDate(v as string) : '—'),
  },
];

function onRequest() {
  // 前端分页（数据已全部加载）
}

const exportColumns = [
  { label: i18nT('raw.s667f13'), field: 'deviceId' },
  { label: i18nT('raw.sd79448'), field: 'description' },
  { label: i18nT('raw.s509e53'), field: 'level' },
  { label: i18nT('raw.s9fb403'), field: 'status' },
  {
    label: i18nT('raw.s1d36cd'),
    field: 'createTime',
    format: (row: HiddenDanger) => (row.createTime ? formatDate(row.createTime) : '—'),
  },
];

async function exportSelected(list: HiddenDanger[], clear: () => void) {
  await exportToXlsx(list, exportColumns, `隐患统计_选中${list.length}项`);
  clear();
}

onMounted(async () => {
  try {
    const res = (await dangerApi.list(1, 20)) as PagedResult<HiddenDanger>;
    allDangers.value = res.items;
    total.value = res.total || 0;
    rows.value = res.items;
    levelData.value = aggregate(res.items, 'level', dangerLevelText);
    statusData.value = aggregate(res.items, 'status', dangerStatusText);
  } catch {
    allDangers.value = [];
    $q.notify({ type: 'negative', message: i18nT('raw.s3b41e7'), timeout: 4000 });
  }
});
</script>
