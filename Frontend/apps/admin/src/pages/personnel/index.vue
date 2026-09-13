<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.s35994a') }}</div>

    <!-- KPI 卡片 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-3" v-for="k in kpiItems" :key="k.label">
        <StatCard :label="k.label" :value="k.value" :icon="k.icon" :color="k.color" />
      </div>
    </div>

    <!-- 图表区（抽取为 PersonnelCertVerify） -->
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.sddc4d5') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="certOption" :option="certOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s561642')"
              :hint="i18nT('raw.s323be8')"
            />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s8eaa1b') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="statusOption" :option="statusOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.scf2f25')"
              :hint="i18nT('raw.s25e84f')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- Tab 区 -->
    <q-tabs
      v-model="tab"
      dense
      class="text-grey q-mb-md"
      active-color="primary"
      indicator-color="primary"
      align="left"
      no-caps
    >
      <q-tab name="list" icon="people" :label="i18nT('raw.s72048a')" />
      <q-tab name="certificate" icon="badge" :label="i18nT('raw.s8ae3ae')" />
    </q-tabs>

    <q-tab-panels v-model="tab" animated>
      <!-- 人员列表 -->
      <q-tab-panel name="list">
        <q-card flat bordered class="erp-card">
          <q-card-section>
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
                  @click="exportSelected(selected as RoleMatrixPerson[], clear)"
                />
              </template>
            </PageTable>
          </q-card-section>
        </q-card>
      </q-tab-panel>

      <!-- 持证核验 -->
      <q-tab-panel name="certificate">
        <PersonnelCertVerify :cert-data="certData" :status-data="statusData" />
      </q-tab-panel>
    </q-tab-panels>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { personnelApi } from '@erp-new-frontend-monorepo/api';
import { PageTable, EChartBase, EmptyState, StatCard } from '@erp-new-frontend-monorepo/components';
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { useTableState } from '@erp-new-frontend-monorepo/composables';
import PersonnelCertVerify from './PersonnelCertVerify.vue';

const $q = useQuasar();
const tab = ref('list');

interface RoleMatrixPerson {
  id?: string;
  userName?: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  roles?: string[];
  certificateNo?: string;
  certificateExpiry?: string;
  tenantId?: string;
}

// ── 数据 ──
const { rows, total, loading, load } = useTableState<RoleMatrixPerson>();

const kpiItems = computed(() => [
  { label: i18nT('raw.saa6bc8'), value: total.value || 0, icon: 'people', color: 'blue' as const },
  {
    label: i18nT('raw.s69c72c'),
    value: rows.value.filter((r) => r.certificateNo).length,
    icon: 'verified',
    color: 'green' as const,
  },
  {
    label: i18nT('raw.s4f40aa'),
    value: rows.value.filter(
      (r) => r.certificateNo && parseDaysLeft(r.certificateExpiry ?? '') <= 30,
    ).length,
    icon: 'schedule',
    color: 'orange' as const,
  },
  {
    label: i18nT('raw.s21ec4d'),
    value: rows.value.filter((r) => !r.certificateNo).length,
    icon: 'warning',
    color: 'red' as const,
  },
]);

// ── 图表数据 ──
interface PieDatum {
  name: string;
  value: number;
}
interface BarDatum {
  name: string;
  value: number;
}

const certData = ref<PieDatum[]>([]);
const statusData = ref<BarDatum[]>([]);

function aggregateCert(items: RoleMatrixPerson[]): PieDatum[] {
  const counter = new Map<string, number>();
  for (const r of items) {
    if (!r.certificateNo) {
      counter.set(i18nT('raw.s4329b1'), (counter.get(i18nT('raw.s4329b1')) || 0) + 1);
      continue;
    }
    const days = parseDaysLeft(r.certificateExpiry ?? '');
    if (days < 0) counter.set(i18nT('raw.s543e44'), (counter.get(i18nT('raw.s543e44')) || 0) + 1);
    else if (days <= 30)
      counter.set(i18nT('raw.s28c05f'), (counter.get(i18nT('raw.s28c05f')) || 0) + 1);
    else counter.set(i18nT('raw.s42b3c3'), (counter.get(i18nT('raw.s42b3c3')) || 0) + 1);
  }
  return [...counter.entries()].map(([k, v]) => ({ name: k, value: v }));
}

function aggregateStatus(items: RoleMatrixPerson[]): BarDatum[] {
  const valid = items.filter(
    (r) => r.certificateNo && parseDaysLeft(r.certificateExpiry ?? '') > 30,
  ).length;
  const expiring = items.filter((r) => {
    if (!r.certificateNo) return false;
    const d = parseDaysLeft(r.certificateExpiry ?? '');
    return d >= 0 && d <= 30;
  }).length;
  const expired = items.filter(
    (r) => r.certificateNo && parseDaysLeft(r.certificateExpiry ?? '') < 0,
  ).length;
  const noCert = items.filter((r) => !r.certificateNo).length;
  return [
    { name: i18nT('raw.s42b3c3'), value: valid },
    { name: i18nT('raw.s9b85fe'), value: expiring },
    { name: i18nT('raw.s543e44'), value: expired },
    { name: i18nT('raw.s4329b1'), value: noCert },
  ];
}

const certOption = computed<Record<string, unknown> | null>(() =>
  certData.value.length
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
            data: certData.value,
          },
        ],
      }
    : null,
);

const statusOption = computed<Record<string, unknown> | null>(() =>
  statusData.value.length
    ? {
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
        xAxis: { type: 'category', data: statusData.value.map((d) => d.name) },
        yAxis: { type: 'value' },
        series: [
          {
            name: i18nT('raw.sc332d3'),
            type: 'bar',
            data: statusData.value.map((d) => d.value),
            itemStyle: { color: '#1565c0', borderRadius: [4, 4, 0, 0] },
            barWidth: '50%',
          },
        ],
      }
    : null,
);

// ── 持证核验 ──
type CertStatus = 'valid' | 'expiring' | 'expired';

// eslint-disable-next-line @typescript-eslint/no-unused-vars
interface CertItem {
  id: string;
  name: string;
  certificateNo: string;
  expiry: string;
  daysLeft: number;
  status: CertStatus;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const EXPIRING_DAYS = 30;

function parseDaysLeft(expiry: string): number {
  const d = new Date(expiry);
  if (Number.isNaN(d.getTime())) return Number.MAX_SAFE_INTEGER;
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const target = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  return Math.round((target.getTime() - today.getTime()) / 86400000);
}

// ── 列表 ──
const columns = [
  { name: 'userName', label: i18nT('raw.s9fcdad'), field: 'userName', align: 'left' as const },
  {
    name: 'name',
    label: i18nT('raw.sd84192'),
    field: 'name',
    align: 'left' as const,
    format: (_v: unknown, row: RoleMatrixPerson) =>
      `${row.firstName || ''} ${row.lastName || ''}`.trim() || row.userName || '—',
  },
  { name: 'email', label: i18nT('raw.s54b275'), field: 'email', align: 'left' as const },
  {
    name: 'certificateNo',
    label: i18nT('raw.s7ca440'),
    field: 'certificateNo',
    align: 'left' as const,
  },
  {
    name: 'certificateExpiry',
    label: i18nT('raw.s50bcde'),
    field: 'certificateExpiry',
    align: 'left' as const,
    format: (v: unknown) => (v ? formatDate(v as string) : '—'),
  },
];

function onRequest() {
  // 前端分页（数据已全部加载）
}

const exportColumns = [
  { label: i18nT('raw.s9fcdad'), field: 'userName' },
  {
    label: i18nT('raw.sd84192'),
    field: 'name',
    format: (row: RoleMatrixPerson) =>
      `${row.firstName || ''} ${row.lastName || ''}`.trim() || row.userName || '—',
  },
  { label: i18nT('raw.s54b275'), field: 'email' },
  { label: i18nT('raw.s7ca440'), field: 'certificateNo' },
  {
    label: i18nT('raw.s50bcde'),
    field: 'certificateExpiry',
    format: (row: RoleMatrixPerson) =>
      row.certificateExpiry ? formatDate(row.certificateExpiry) : '—',
  },
];

async function exportSelected(list: RoleMatrixPerson[], clear: () => void) {
  const { exportToXlsx } = await import('@erp-new-frontend-monorepo/utils');
  await exportToXlsx(
    list,
    exportColumns,
    `${i18nT('raw.spen_export_filename', { count: list.length })}`,
  );
  clear();
}

onMounted(async () => {
  try {
    await load(async () => {
      const res = await personnelApi.list();
      rows.value = res as RoleMatrixPerson[];
      certData.value = aggregateCert(rows.value);
      statusData.value = aggregateStatus(rows.value);
    });
  } catch {
    $q.notify({ type: 'negative', message: i18nT('raw.sde3ea5'), timeout: 4000 });
  }
});
</script>
