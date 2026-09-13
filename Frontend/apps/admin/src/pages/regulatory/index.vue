<!-- @layout main -->
<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.sba1a13') }}</div>

    <q-banner v-if="error" dense class="bg-negative text-white q-mb-md" icon="error">
      {{ error }}
    </q-banner>

    <!-- ① 导出监管报文 -->
    <q-card flat bordered erp-card>
      <q-card-section>
        <div class="text-subtitle1">{{ i18nT('raw.sf56cf8') }}</div>
        <div class="row items-center q-col-gutter-sm">
          <div class="col-12 col-md-4">
            <q-select
              v-model="scope"
              :options="scopeOpts"
              :label="i18nT('raw.sbf4a05')"
              dense
              outlined
            />
          </div>
          <div class="col-12 col-md-4">
            <q-input v-model="month" :label="i18nT('raw.s427d84')" dense outlined />
          </div>
          <div class="col-12 col-md-4">
            <q-btn
              color="primary"
              :label="i18nT('raw.sc794a2')"
              :loading="exporting"
              @click="doExport"
              class="full-width"
            />
          </div>
        </div>
        <div v-if="result" class="q-mt-sm">
          <RegulatoryExport :result="result" />
        </div>
        <!-- 导出历史 -->
        <div v-if="exportRows.length || exportRecordsLoading" class="q-mt-md">
          <div class="row items-center q-mb-xs">
            <div class="text-subtitle2">导出历史（本租户）</div>
            <q-space />
            <q-btn
              flat
              dense
              icon="refresh"
              :loading="exportRecordsLoading"
              @click="loadExportRecords"
            />
          </div>
          <q-table
            flat
            bordered
            dense
            :rows="exportRows"
            :columns="exportColumns"
            row-key="id"
            hide-bottom
            :loading="exportRecordsLoading"
          >
            <template #body-cell-scope="{ row }">
              <q-td>{{ scopeLabel(row.scope) }}</q-td>
            </template>
            <template #body-cell-fileUrl="{ row }">
              <q-td>
                <q-btn
                  dense
                  flat
                  size="sm"
                  color="primary"
                  icon="download"
                  label="下载"
                  @click="downloadExport(row)"
                />
              </q-td>
            </template>
          </q-table>
        </div>
      </q-card-section>
    </q-card>

    <!-- ② 接收记录 -->
    <q-card flat bordered erp-card class="q-mt-md">
      <q-card-section>
        <div class="row items-center">
          <div class="text-subtitle1">{{ i18nT('raw.s8795e3') }}</div>
          <q-space />
          <q-btn
            flat
            dense
            icon="refresh"
            :title="i18nT('raw.s6d94e0')"
            :loading="receivesLoading"
            @click="loadReceives()"
          />
        </div>
        <q-banner dense class="bg-amber-2 q-mt-sm" icon="info">
          {{ i18nT('raw.s40reg1') }}
        </q-banner>
        <PageTable
          :rows="receiveRows"
          :columns="receiveColumns"
          :loading="receivesLoading"
          :total="receiveTotal"
          @request="onReceiveRequest"
        >
          <template #no-data>
            <EmptyState icon="inbox" :title="i18nT('raw.s687c47')" :hint="i18nT('raw.s97ec3c')" />
          </template>
          <template #body-cell-source="{ row }">
            <q-td>
              {{ row.source }}
              <q-badge color="amber-8" outline class="q-ml-xs">{{ i18nT('raw.sb82f80') }}</q-badge>
            </q-td>
          </template>
        </PageTable>
      </q-card-section>
    </q-card>

    <!-- ③ 反馈提交 -->
    <RegulatoryFeedbackSection />

    <!-- ④ 审计轨迹（时间线）-->
    <q-card flat bordered erp-card class="q-mt-md">
      <q-card-section>
        <div class="row items-center">
          <div class="text-subtitle1">{{ i18nT('raw.s3717dc') }}</div>
          <q-space />
          <q-checkbox
            v-model="regulatoryOnly"
            dense
            :label="i18nT('raw.s34acb8')"
            class="q-mr-sm"
          />
          <q-btn
            flat
            dense
            icon="refresh"
            :title="i18nT('raw.s6d94e0')"
            :loading="auditsLoading"
            @click="loadAudits"
          />
        </div>
        <q-banner v-if="!auditRegulatoryHit" dense class="bg-grey-2 q-mt-sm" icon="info">
          {{ i18nT('raw.s131aud') }}
        </q-banner>
        <AuditTrail :items="auditTrailItems" />
      </q-card-section>
    </q-card>

    <q-banner class="q-mt-md bg-grey-2" dense>
      {{ i18nT('raw.s138all') }}
    </q-banner>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted, watch } from 'vue';
import { useQuasar } from 'quasar';
import { regulatoryApi, auditsApi } from '@/api';
import type { RegulatoryReceiveRecordItem, RegulatoryExportRecordItem } from '@/api';
import type {
  RegulatoryExportRequest,
  RegulatoryExportResponse,
} from '@erp-new-frontend-monorepo/types';
import { formatDateTime } from '@erp-new-frontend-monorepo/utils';
import {
  RegulatoryExport,
  EmptyState,
  PageTable,
  AuditTrail,
} from '@erp-new-frontend-monorepo/components';
import RegulatoryFeedbackSection from './RegulatoryFeedbackSection.vue';

const $q = useQuasar();

// ── ① 导出 ──
const scope = ref<'device' | 'danger' | 'monthly' | 'all'>('all');
const scopeOpts = ['device', 'danger', 'monthly', 'all'];
const month = ref('2026-08');
const exporting = ref(false);
const error = ref('');
const result = ref<RegulatoryExportResponse | null>(null);

async function doExport() {
  exporting.value = true;
  error.value = '';
  try {
    const body: RegulatoryExportRequest = {
      scope: scope.value,
      month: month.value || undefined,
    };
    result.value = await regulatoryApi.export(body);
    $q.notify({ type: 'positive', message: i18nT('raw.sb8df95') });
    await loadExportRecords();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    $q.notify({ type: 'negative', message: `${i18nT('raw.s176exp')}: ${error.value}` });
  } finally {
    exporting.value = false;
  }
}

// ── ①b 导出历史 ──
const exportRows = ref<RegulatoryExportRecordItem[]>([]);
const exportRecordsLoading = ref(false);
const exportColumns: {
  name: string;
  label: string;
  field: string;
  style?: string;
  format?: (v: unknown) => string;
}[] = [
  { name: 'scope', label: '范围', field: 'scope', style: 'width: 100px' },
  { name: 'month', label: '月份', field: 'month', style: 'width: 100px' },
  { name: 'standard', label: '标准', field: 'standard', style: 'width: 160px' },
  { name: 'generatedBy', label: '生成人', field: 'generatedBy', style: 'width: 120px' },
  {
    name: 'generatedAt',
    label: '生成时间',
    field: 'generatedAt',
    style: 'width: 170px',
    format: (v: unknown) => formatDateTime(v as string),
  },
  { name: 'fileUrl', label: '文件', field: 'fileUrl', style: 'width: 80px' },
];

const SCOPE_LABEL: Record<string, string> = {
  device: '设备台账',
  danger: '隐患台账',
  monthly: '月度报表',
  all: '全量',
};
function scopeLabel(s: string) {
  return SCOPE_LABEL[s] ?? s;
}

async function loadExportRecords() {
  exportRecordsLoading.value = true;
  try {
    const res = await regulatoryApi.exportRecords();
    exportRows.value = res.exports ?? [];
  } catch {
    exportRows.value = [];
  } finally {
    exportRecordsLoading.value = false;
  }
}

function downloadExport(row: RegulatoryExportRecordItem) {
  const a = document.createElement('a');
  a.href = row.fileUrl;
  a.download = `regulatory-${row.scope}-${row.month ?? 'all'}.txt`;
  document.body.appendChild(a);
  a.click();
  a.remove();
}

// ── ② 接收记录 ──
const receivesLoading = ref(false);
const receivePage = ref(1);
const receivePageSize = ref(20);
const receiveTotal = ref(0);

const receiveRows = ref<RegulatoryReceiveRecordItem[]>([]);
const receiveColumns = [
  { name: 'referenceNo', label: i18nT('raw.sa86114'), field: 'referenceNo', style: 'width: 140px' },
  { name: 'title', label: i18nT('raw.sa0661a'), field: 'title' },
  { name: 'source', label: i18nT('raw.sb62ace'), field: 'source', style: 'width: 180px' },
  {
    name: 'receivedAt',
    label: i18nT('raw.sc51525'),
    field: 'receivedAt',
    format: (v: unknown) => formatDateTime(v as string),
    style: 'width: 150px',
  },
];

function loadReceives(page = receivePage.value, pageSize = receivePageSize.value) {
  receivesLoading.value = true;
  receivePage.value = page;
  receivePageSize.value = pageSize;
  regulatoryApi
    .receiveRecords(page, pageSize)
    .then((res) => {
      receiveRows.value = res.items;
      receiveTotal.value = res.total;
    })
    .catch(() => {
      receiveRows.value = [];
      receiveTotal.value = 0;
    })
    .finally(() => {
      receivesLoading.value = false;
    });
}

function onReceiveRequest(payload: { page: number; rowsPerPage: number }) {
  loadReceives(payload.page, payload.rowsPerPage);
}

// ── ④ 审计轨迹 ──
const auditsLoading = ref(false);
const regulatoryOnly = ref(true);
const auditRegulatoryHit = ref(true);
const auditRows = ref<AuditRow[]>([]);

interface AuditRow {
  id?: string;
  eventType?: string;
  action?: string;
  entity?: string;
  entityName?: string;
  source?: string;
  module?: string;
  detail?: string;
  occurredAtUtc?: string;
}

const auditTrailItems = computed(() =>
  auditRows.value.map((a) => ({
    action: a.eventType || a.action || '-',
    entity: a.entity || a.entityName || a.source || '-',
    detail: a.detail || a.module || '',
    createTime: a.occurredAtUtc ? formatDateTime(a.occurredAtUtc) : '-',
  })),
);

function isRegulatoryAudit(a: AuditRow): boolean {
  const hay = [a.eventType, a.source, a.action, a.entity, a.entityName, a.module, a.detail]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
  return (
    hay.includes('regulatory') ||
    hay.includes(i18nT('raw.s51de65')) ||
    hay.includes(i18nT('raw.s948f35'))
  );
}

async function loadAudits() {
  auditsLoading.value = true;
  try {
    const res = await auditsApi.list<AuditRow>(1, 20);
    const items = res.items || [];
    const regulatory = items.filter((i) => isRegulatoryAudit(i ?? {}));
    auditRegulatoryHit.value = regulatory.length > 0;
    auditRows.value = regulatoryOnly.value && regulatory.length ? regulatory : items;
  } catch (e) {
    auditRows.value = [];
    auditRegulatoryHit.value = true;
    $q.notify({
      type: 'negative',
      message: `${i18nT('raw.s296aud')}: ${e instanceof Error ? e.message : String(e)}`,
    });
  } finally {
    auditsLoading.value = false;
  }
}

watch(regulatoryOnly, () => {
  void loadAudits();
});

onMounted(() => {
  void loadReceives();
  void loadAudits();
  void loadExportRecords();
});
</script>
