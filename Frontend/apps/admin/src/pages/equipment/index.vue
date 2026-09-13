<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.s62bebb') }}</div>

    <!-- KPI 卡片 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-3" v-for="k in kpiItems" :key="k.label">
        <StatCard :label="k.label" :value="k.value" :icon="k.icon" :color="k.color" />
      </div>
      <div class="col-12 col-md-3">
        <q-card flat bordered class="erp-card">
          <q-card-section class="row items-center q-gutter-md">
            <q-circular-progress
              show-value
              font-size="16px"
              :value="healthRate"
              size="72px"
              :thickness="0.22"
              :color="healthRate >= 70 ? 'green' : healthRate >= 40 ? 'orange' : 'red'"
              track-color="grey-3"
              >{{ healthRate }}%</q-circular-progress
            >
            <div>
              <div class="text-subtitle2">设备健康率</div>
              <div class="text-caption text-grey-6">{{ i18nT('raw.s8b1a2c') }}</div>
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 图表区 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
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
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.sd78d95') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="healthOption" :option="healthOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s7da25a')"
              :hint="i18nT('raw.s941b25')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 设备列表 -->
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">{{ i18nT('raw.s0ff5ec') }}</div>
        <PageTable
          :rows="filteredRows"
          :columns="columns"
          :loading="loading"
          :total="total"
          :dense="$q.screen.lt.md"
          selection-mode
          @request="onRequest"
        >
          <template #top>
            <div class="row items-center q-gutter-sm">
              <SearchInput v-model="keyword" placeholder="搜索设备名称/编号/类型" />
            </div>
          </template>
          <template #selection-actions="{ selected, clear }">
            <q-btn
              v-if="selected.length > 0"
              color="primary"
              outline
              icon="file_download"
              :label="i18nT('raw.s1b06cd')"
              @click="exportSelected(selected as Device[], clear)"
            />
          </template>
          <template #body-cell-healthScore="{ row }">
            <q-td>
              <HealthScoreBadge :score="row.healthScore" :level="row.healthLevel" />
            </q-td>
          </template>
        </PageTable>
      </q-card-section>
    </q-card>

    <!-- 下钻弹窗 -->
    <q-dialog v-model="detailDialog">
      <q-card style="width: 720px; max-width: 92vw">
        <q-card-section class="row items-center">
          <div class="text-h6">设备明细 · {{ detailTitle }}</div>
          <q-space />
          <q-btn flat round dense icon="close" v-close-popup />
        </q-card-section>
        <q-separator />
        <q-card-section>
          <PageTable
            :rows="detailRows"
            :columns="detailColumns"
            :loading="false"
            :total="detailRows.length"
            :rows-per-page="10"
            flat
            bordered
          >
            <template #body-cell-healthScore="{ row }">
              <q-td>
                <HealthScoreBadge :score="row.healthScore" :level="row.healthLevel" />
              </q-td>
            </template>
          </PageTable>
        </q-card-section>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { deviceApi } from '@erp-new-frontend-monorepo/api';
import type { Device, PagedResult } from '@erp-new-frontend-monorepo/types';
import {
  PageTable,
  SearchInput,
  HealthScoreBadge,
  EChartBase,
  CHART_PALETTE,
  EmptyState,
  StatCard,
} from '@erp-new-frontend-monorepo/components';
import { formatDate, deviceTypeText, exportToXlsx } from '@erp-new-frontend-monorepo/utils';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

const $q = useQuasar();

// ── 数据 ──
const { rows, total, loading, page, size, load } = useTableState<Device>({ size: 20 });
const keyword = ref('');
const filteredRows = computed(() => {
  const k = keyword.value.trim().toLowerCase();
  if (!k) return rows.value;
  return rows.value.filter((d) =>
    [d.name, d.code, d.type].some((f) =>
      String(f ?? '')
        .toLowerCase()
        .includes(k),
    ),
  );
});
const allDevices = ref<Device[]>([]);
const detailDialog = ref(false);
const detailTitle = ref('');
const detailRows = ref<Device[]>([]);

const kpiItems = computed(() => [
  {
    label: i18nT('raw.sdf8044'),
    value: total.value || 0,
    icon: 'inventory_2',
    color: 'blue' as const,
  },
  {
    label: i18nT('raw.s8b1a2c'),
    value: allDevices.value.filter((d) => (d.healthScore ?? 0) >= 70).length,
    icon: 'check_circle',
    color: 'green' as const,
  },
  {
    label: i18nT('raw.sb8918e'),
    value: allDevices.value.filter((d) => (d.healthScore ?? 0) < 55).length,
    icon: 'error',
    color: 'red' as const,
  },
  {
    label: i18nT('raw.sf92b3f'),
    value: allDevices.value.filter((d) => !d.installDate).length,
    icon: 'schedule',
    color: 'orange' as const,
  },
]);

const healthRate = computed(() => {
  const total = allDevices.value.length;
  if (!total) return 0;
  const healthy = allDevices.value.filter((d) => (d.healthScore ?? 0) >= 70).length;
  return Math.round((healthy / total) * 100);
});

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

// ── 下钻 ──
interface PieClickParams {
  data?: { raw?: string; name?: string; value?: number };
}
function onTypeClick(params: PieClickParams) {
  const raw = params?.data?.raw;
  if (!raw) return;
  const list = allDevices.value.filter((d) => d.type === raw);
  if (!list.length) return;
  detailRows.value = list;
  detailTitle.value = `${deviceTypeText(raw)}（${list.length} 台）`;
  detailDialog.value = true;
}

const detailColumns = [
  { name: 'code', label: i18nT('raw.s667f13'), field: 'code', align: 'left' as const },
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name', align: 'left' as const },
  { name: 'tenantId', label: i18nT('raw.s8b7cb0'), field: 'tenantId', align: 'left' as const },
  {
    name: 'healthScore',
    label: i18nT('raw.s471784'),
    field: 'healthScore',
    align: 'center' as const,
  },
];

// ── 列表 ──
const columns = [
  { name: 'code', label: i18nT('raw.scbbea3'), field: 'code', align: 'left' as const },
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name', align: 'left' as const },
  { name: 'type', label: i18nT('raw.sd68dd1'), field: 'type', align: 'left' as const },
  { name: 'tenantId', label: i18nT('raw.s8b7cb0'), field: 'tenantId', align: 'left' as const },
  {
    name: 'healthScore',
    label: i18nT('raw.s471784'),
    field: 'healthScore',
    align: 'center' as const,
  },
];

async function loadDevices() {
  const res = (await deviceApi.list(page.value, size.value)) as PagedResult<Device>;
  rows.value = res.items;
  total.value = res.total || 0;
}

async function onRequest({ page: p, rowsPerPage: s }: { page: number; rowsPerPage: number }) {
  page.value = p;
  size.value = s;
  await load(loadDevices);
}

const exportColumns = [
  { label: i18nT('raw.scbbea3'), field: 'code' },
  { label: i18nT('raw.sf43de3'), field: 'name' },
  { label: i18nT('raw.sd68dd1'), field: 'type' },
  { label: i18nT('raw.s97a093'), field: 'tenantId' },
  { label: i18nT('raw.s471784'), field: 'healthScore' },
  {
    label: i18nT('raw.saf9956'),
    field: 'createdAt',
    format: (row: Device) => formatDate(row.createdAt),
  },
];

async function exportSelected(list: Device[], clear: () => void) {
  await exportToXlsx(list, exportColumns, `设备台账_选中${list.length}项`);
  clear();
}

onMounted(async () => {
  try {
    const res = (await deviceApi.list(1, 20)) as PagedResult<Device>;
    allDevices.value = res.items;
    total.value = res.total || 0;
    rows.value = res.items;
    typeData.value = aggregate(res.items, 'type', deviceTypeText);
    healthData.value = aggregateHealth(res.items);
  } catch {
    allDevices.value = [];
    $q.notify({ type: 'negative', message: i18nT('raw.sde3ea5'), timeout: 4000 });
  }
});
</script>
