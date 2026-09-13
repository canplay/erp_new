<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.sb3a0b6') }}</div>

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
            <div class="text-subtitle2">{{ i18nT('raw.s55d493') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="statusOption" :option="statusOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s525cf2')"
              :hint="i18nT('raw.s1c57c2')"
            />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.s11ccf9') }}</div>
          </q-card-section>
          <q-card-section>
            <EChartBase v-if="tenantOption" :option="tenantOption" height="300px" />
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s525cf2')"
              :hint="i18nT('raw.s95d9ae')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 任务列表 -->
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">{{ i18nT('raw.s1c6762') }}</div>
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
              color="negative"
              outline
              icon="delete"
              :label="i18nT('raw.s1b10b2')"
              :loading="batchDeleting"
              @click="confirmBatchDelete(selected as InspectionTask[], clear)"
            />
            <q-btn
              v-if="selected.length > 0"
              class="q-ml-sm"
              color="primary"
              outline
              icon="file_download"
              :label="i18nT('raw.s1b06cd')"
              @click="exportSelected(selected as InspectionTask[], clear)"
            />
          </template>
          <template #body-cell-tenantId="{ row }">
            <q-td
              ><code>{{ row.tenantId }}</code></q-td
            >
          </template>
          <template #body-cell-status="{ row }">
            <q-td>
              <q-badge :color="statusColor(row.status)" :label="statusLabel(row.status)" />
            </q-td>
          </template>
        </PageTable>
      </q-card-section>
    </q-card>

    <ConfirmDialog
      v-model="batchDeleteDialog"
      :title="i18nT('raw.sbd7449')"
      :message="`确认删除 ${pendingDelete?.list.length || 0} 条巡检任务？该操作不可恢复。`"
      :loading="batchDeleting"
      @confirm="doBatchDelete"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { dailyApi } from '@erp-new-frontend-monorepo/api';
import type { InspectionTask, PagedResult } from '@erp-new-frontend-monorepo/types';
import {
  PageTable,
  EChartBase,
  EmptyState,
  StatCard,
  ConfirmDialog,
} from '@erp-new-frontend-monorepo/components';
import { exportToXlsx, deviceTypeText } from '@erp-new-frontend-monorepo/utils';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

const $q = useQuasar();

const BLUE_PALETTE = ['#1565c0', '#42a5f5', '#90caf9', '#1976d2', '#64b5f6', '#bbdefb', '#0d47a1'];

// ── 数据 ──
const { rows, total, loading, load } = useTableState<InspectionTask>();

const kpiItems = computed(() => [
  { label: i18nT('raw.scc12ca'), value: total.value || 0, icon: 'task', color: 'blue' as const },
  {
    label: i18nT('raw.s5ade6d'),
    value: rows.value.filter((r) => r.status === 'done').length,
    icon: 'check_circle',
    color: 'green' as const,
  },
  {
    label: i18nT('raw.sb17443'),
    value: rows.value.filter((r) => r.status === 'doing').length,
    icon: 'pending',
    color: 'orange' as const,
  },
  {
    label: i18nT('raw.s4e19b1'),
    value: rows.value.filter((r) => r.status === 'pending').length,
    icon: 'schedule',
    color: 'red' as const,
  },
]);

// ── 图表数据 ──
interface PieDatum {
  name: string;
  value: number;
}

const statusData = ref<PieDatum[]>([]);
const tenantData = ref<PieDatum[]>([]);

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

const statusLabel = (s: string) =>
  ({ pending: i18nT('raw.s4e19b1'), doing: i18nT('raw.sb17443'), done: i18nT('raw.s5ade6d') })[
    s?.toLowerCase()
  ] || s;
const statusColor = (s: string) =>
  ({ pending: 'orange', doing: 'blue', done: 'green' })[s?.toLowerCase()] || 'grey';

const statusOption = computed<Record<string, unknown> | null>(() =>
  statusData.value.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: BLUE_PALETTE,
        series: [
          {
            name: i18nT('raw.s21e73d'),
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

const tenantOption = computed<Record<string, unknown> | null>(() =>
  tenantData.value.length
    ? {
        tooltip: { trigger: 'item', formatter: '{b}: {c}（{d}%）' },
        legend: { bottom: 0, type: 'scroll', textStyle: { color: '#666' } },
        color: BLUE_PALETTE,
        series: [
          {
            name: i18nT('raw.s3b1f79'),
            type: 'pie',
            radius: ['38%', '66%'],
            center: ['50%', '44%'],
            itemStyle: { borderRadius: 4, borderColor: '#fff', borderWidth: 2 },
            label: { formatter: '{b}\n{c}', fontSize: 12, color: '#444' },
            data: tenantData.value,
          },
        ],
      }
    : null,
);

// ── 列表 ──
const columns = [
  { name: 'deviceName', label: i18nT('raw.s59fc53'), field: 'deviceName', align: 'left' as const },
  {
    name: 'deviceType',
    label: i18nT('raw.s7b4c61'),
    field: 'deviceType',
    align: 'left' as const,
    format: (v: string) => deviceTypeText(v),
  },
  { name: 'inspector', label: i18nT('raw.sf11850'), field: 'inspector', align: 'left' as const },
  { name: 'tenantId', label: i18nT('raw.s8b7cb0'), field: 'tenantId', align: 'left' as const },
  { name: 'planDate', label: i18nT('raw.s50737f'), field: 'planDate', align: 'left' as const },
  { name: 'status', label: i18nT('raw.s9fb403'), field: 'status', align: 'center' as const },
];

function onRequest() {
  // 前端分页（数据已全部加载）
}

const exportColumns = [
  { label: i18nT('raw.s59fc53'), field: 'deviceName' },
  {
    label: i18nT('raw.s7b4c61'),
    field: 'deviceType',
    format: (row: InspectionTask) => deviceTypeText(row.deviceType),
  },
  { label: i18nT('raw.sf11850'), field: 'inspector' },
  { label: i18nT('raw.s8b7cb0'), field: 'tenantId' },
  { label: i18nT('raw.s50737f'), field: 'planDate' },
  {
    label: i18nT('raw.s9fb403'),
    field: 'status',
    format: (row: InspectionTask) => statusLabel(row.status),
  },
];

const batchDeleting = ref(false);
const pendingDelete = ref<{ list: InspectionTask[]; clear: () => void } | null>(null);
const batchDeleteDialog = ref(false);

function confirmBatchDelete(list: InspectionTask[], clear: () => void) {
  if (!list.length) return;
  pendingDelete.value = { list, clear };
  batchDeleteDialog.value = true;
}

async function batchDeleteSelected(list: InspectionTask[], clear: () => void) {
  batchDeleting.value = true;
  try {
    await Promise.all(list.map((t) => dailyApi.delete(t.id)));
    $q.notify({ type: 'positive', message: `已删除 ${list.length} 条任务`, timeout: 3000 });
    clear();
    await loadTasks();
  } catch {
    $q.notify({ type: 'negative', message: i18nT('raw.sfa416d'), timeout: 4000 });
  } finally {
    batchDeleting.value = false;
  }
}

async function doBatchDelete() {
  if (!pendingDelete.value) return;
  await batchDeleteSelected(pendingDelete.value.list, pendingDelete.value.clear);
  batchDeleteDialog.value = false;
}

async function loadTasks() {
  await load(() =>
    dailyApi.tasks().then((res) => {
      const data = (res as unknown as PagedResult<InspectionTask>) || { items: [], total: 0 };
      rows.value = data.items || [];
      total.value = data.total || 0;
      statusData.value = aggregate(rows.value, 'status', statusLabel);
      tenantData.value = aggregate(rows.value, 'tenantId', (k) => k || i18nT('raw.s376ddb'));
    }),
  );
}

async function exportSelected(list: InspectionTask[], clear: () => void) {
  await exportToXlsx(list, exportColumns, `日管控任务_选中${list.length}项`);
  clear();
}

onMounted(() => {
  void loadTasks();
});
</script>
