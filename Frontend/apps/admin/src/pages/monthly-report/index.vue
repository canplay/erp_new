<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.sed6f86') }}</div>

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
            <div class="text-subtitle2">{{ i18nT('raw.s7fd062') }}</div>
          </q-card-section>
          <q-card-section>
            <RiskGauge :score="complianceScore" :level="complianceLevel" />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.sbde34a') }}</div>
          </q-card-section>
          <q-card-section>
            <div v-if="complianceFactors.length" class="q-gutter-md">
              <div v-for="f in complianceFactors" :key="f.name" class="row items-center">
                <div class="text-subtitle2" style="min-width: 130px">{{ f.name }}</div>
                <q-linear-progress
                  :value="f.rate / 100"
                  color="primary"
                  class="q-mx-sm"
                  style="width: 160px"
                />
                <div class="text-caption text-grey-7">{{ f.rate }}%（权重×{{ f.weight }}）</div>
              </div>
            </div>
            <EmptyState
              v-else
              icon="inbox"
              :title="i18nT('raw.s597b04')"
              :hint="i18nT('raw.sbeda23')"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 跨租户月调度报告列表 -->
    <q-card flat bordered class="erp-card">
      <q-card-section class="q-pb-none">
        <div class="text-subtitle1 q-mb-sm">{{ i18nT('raw.s57330f') }}（跨租户）</div>
      </q-card-section>
      <q-card-section>
        <PageTable
          :rows="rows"
          :columns="columns"
          :loading="loading"
          :total="total"
          @request="onRequest"
        >
          <template #body-cell-tenantId="{ row }">
            <q-td
              ><code>{{ row.tenantId }}</code></q-td
            >
          </template>
          <template #body-cell-riskIndex="{ row }">
            <q-td>
              <q-badge :color="riskColor(row.riskIndex)" :label="row.riskIndex" />
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
import { ref, onMounted } from 'vue';
import { analyticsApi, monthlyApi } from '@erp-new-frontend-monorepo/api';
import type { ComplianceFactor, MonthlyReportListItem } from '@erp-new-frontend-monorepo/types';
import { RiskGauge, EmptyState, StatCard, PageTable } from '@erp-new-frontend-monorepo/components';

const kpiItems = ref([
  { label: i18nT('raw.s57330f'), value: 0, icon: 'description', color: 'blue' as const },
  { label: i18nT('raw.s2968a1'), value: 0, icon: 'verified', color: 'green' as const },
  { label: i18nT('raw.s6bb60d'), value: 0, icon: 'pending', color: 'orange' as const },
  { label: i18nT('raw.se036fa'), value: 0, icon: 'check_circle', color: 'teal' as const },
]);

const complianceScore = ref(0);
const complianceLevel = ref(i18nT('raw.sc1dfbc'));
const complianceFactors = ref<ComplianceFactor[]>([]);

// 跨租户列表
const rows = ref<MonthlyReportListItem[]>([]);
const loading = ref(false);
const total = ref(0);

const columns = [
  { name: 'month', label: '月份', field: 'month', align: 'left' as const },
  { name: 'title', label: '标题', field: 'title', align: 'left' as const },
  { name: 'riskIndex', label: '风险指数', field: 'riskIndex', align: 'center' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
  { name: 'tenantId', label: i18nT('raw.s8b7cb0'), field: 'tenantId', align: 'left' as const },
];

function riskColor(score: number) {
  return score >= 70 ? 'red' : score >= 50 ? 'orange' : 'green';
}

function levelOfScore(score: number): string {
  if (score >= 80) return i18nT('raw.s188468');
  if (score >= 60) return i18nT('raw.s554c69');
  if (score >= 30) return i18nT('raw.sd15a8b');
  return i18nT('raw.sc1dfbc');
}

async function onRequest({ page, rowsPerPage }: { page: number; rowsPerPage: number }) {
  loading.value = true;
  try {
    const res = await monthlyApi.list(page, rowsPerPage);
    rows.value = res.items ?? [];
    total.value = res.total ?? 0;
    kpiItems.value[0]!.value = total.value;
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  void onRequest({ page: 1, rowsPerPage: 10 });

  try {
    const res = await analyticsApi.complianceAssessment();
    complianceScore.value = res?.score ?? 0;
    complianceLevel.value = levelOfScore(complianceScore.value);
    complianceFactors.value = res?.factors ?? [];
  } catch {
    /* 保留空态 */
  }
});
</script>
