<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.s4bc116') }}</div>

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
            <div class="text-subtitle2">{{ i18nT('raw.s20902c') }}</div>
          </q-card-section>
          <q-card-section>
            <IssueWordCloud :issues="wordCloudIssues" />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card flat bordered class="erp-card">
          <q-card-section class="q-pb-none">
            <div class="text-subtitle2">{{ i18nT('raw.se2f80a') }}</div>
          </q-card-section>
          <q-card-section>
            <RiskGauge :score="riskScore" :level="riskLevel" />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 跨租户周排查计划列表 -->
    <q-card flat bordered class="erp-card">
      <q-card-section class="q-pb-none">
        <div class="text-subtitle1 q-mb-sm">{{ i18nT('raw.s7e3071') }}（跨租户）</div>
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
import { weeklyApi } from '@erp-new-frontend-monorepo/api';
import type { WeeklyPlanListItem } from '@erp-new-frontend-monorepo/types';
import {
  IssueWordCloud,
  RiskGauge,
  StatCard,
  PageTable,
} from '@erp-new-frontend-monorepo/components';

const kpiItems = ref([
  { label: i18nT('raw.s7e3071'), value: 0, icon: 'assignment', color: 'blue' as const },
  { label: i18nT('raw.s036b7f'), value: 0, icon: 'speed', color: 'orange' as const },
  { label: i18nT('raw.sb7cfba'), value: '—', icon: 'check_circle', color: 'green' as const },
  { label: i18nT('raw.s1fb93d'), value: 0, icon: 'warning', color: 'red' as const },
]);

const wordCloudIssues = ref<{ word: string; count: number }[]>([]);
const riskScore = ref(0);
const riskLevel = ref(i18nT('raw.sc1dfbc'));

// 跨租户列表
const rows = ref<WeeklyPlanListItem[]>([]);
const loading = ref(false);
const total = ref(0);

const columns = [
  { name: 'deviceName', label: i18nT('raw.s59fc53'), field: 'deviceName', align: 'left' as const },
  { name: 'weekNumber', label: '周次', field: 'weekNumber', align: 'left' as const },
  { name: 'riskIndex', label: i18nT('raw.s6e5e75'), field: 'riskIndex', align: 'center' as const },
  { name: 'focusAreas', label: '重点区域', field: 'focusAreas', align: 'left' as const },
  { name: 'tenantId', label: i18nT('raw.s8b7cb0'), field: 'tenantId', align: 'left' as const },
  {
    name: 'generatedAt',
    label: '生成时间',
    field: 'generatedAt',
    align: 'left' as const,
    format: (v: string) => (v ? new Date(v).toLocaleString('zh-CN') : '—'),
  },
];

function riskColor(score: number) {
  return score >= 70 ? 'red' : score >= 50 ? 'orange' : 'green';
}

function levelOfRisk(score: number): string {
  if (score >= 80) return i18nT('raw.s188468');
  if (score >= 60) return i18nT('raw.s554c69');
  if (score >= 30) return i18nT('raw.sd15a8b');
  return i18nT('raw.sc1dfbc');
}

async function onRequest({ page, rowsPerPage }: { page: number; rowsPerPage: number }) {
  loading.value = true;
  try {
    const res = await weeklyApi.list(page, rowsPerPage);
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
    const analysis = await weeklyApi.analysis();
    if (analysis?.wordCloud?.length) {
      wordCloudIssues.value = analysis.wordCloud.map((w: { text: string; weight: number }) => ({
        word: w.text,
        count: w.weight,
      }));
    }
  } catch {
    /* 保留空态 */
  }

  try {
    const plan = await weeklyApi.plan();
    if (plan?.riskIndex != null) {
      riskScore.value = Math.max(0, Math.min(100, Number(plan.riskIndex)));
      riskLevel.value = levelOfRisk(riskScore.value);
    }
  } catch {
    /* 保留空态 */
  }
});
</script>
