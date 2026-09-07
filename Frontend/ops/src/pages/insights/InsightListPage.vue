<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">AI 分析报告</div>

    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section class="text-center">
            <q-icon name="insights" size="48px" color="primary" />
            <div class="text-h6 q-mt-sm">AI 发展建议</div>
            <div class="text-grey q-mt-sm">基于统计数据分析账号运营情况，给出针对性的发展建议</div>
            <q-btn color="primary" icon="auto_awesome" label="生成报告" @click="generate" :loading="genLoading" class="q-mt-md" />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section class="text-center">
            <q-icon name="calendar_month" size="48px" color="orange" />
            <div class="text-h6 q-mt-sm">定期报告</div>
            <div class="text-grey q-mt-sm">配置每周/每月自动生成分析报告，追踪账号成长轨迹</div>
            <q-btn color="orange" icon="schedule" label="了解详情" disable class="q-mt-md" />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <q-card bordered>
      <q-card-section><div class="text-h6">历史报告</div></q-card-section>
      <q-table :rows="reports" :columns="columns" row-key="id" flat bordered :loading="loading">
        <template v-slot:body-cell-insight_type="{ row }">
          <td><q-chip dense :color="row.insight_type === 'weekly' ? 'primary' : 'orange'" text-color="white">{{ row.insight_type }}</q-chip></td>
        </template>
        <template v-slot:body-cell-content="{ row }">
          <td class="text-grey">{{ (row.summary || row.analysis_body || '').substring(0, 100) }}</td>
        </template>
        <template v-slot:no-data><div class="text-grey text-center q-py-md">暂无报告，点击上方按钮生成一份</div></template>
      </q-table>
    </q-card>

    <q-dialog v-model="showResult">
      <q-card style="min-width:400px">
        <q-card-section class="bg-primary text-white"><div class="text-h6">AI 分析报告</div></q-card-section>
        <q-card-section style="white-space:pre-wrap">{{ reportContent }}</q-card-section>
        <q-card-actions align="right"><q-btn flat label="关闭" v-close-popup /></q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { insightApi } from '@/api/insights';

const columns = [
  { name: 'insight_type', label: '类型', field: 'insight_type', align: 'left' as const },
  { name: 'title', label: '标题', field: 'title', align: 'left' as const },
  { name: 'content', label: '摘要', field: 'content', align: 'left' as const },
  { name: 'created_at', label: '生成时间', field: 'created_at', align: 'left' as const },
];
const reports = ref<Array<Record<string, unknown>>>([]);
const loading = ref(false);
const genLoading = ref(false);
const showResult = ref(false);
const reportContent = ref('');

async function generate() {
  genLoading.value = true;
  try {
    const result = await insightApi.generate({ insight_type: 'weekly', period_start: '', period_end: '' }) as unknown as Record<string, unknown>;
    reportContent.value = (result?.analysis_body as string) || '报告生成完成，请查看历史报告列表。';
    showResult.value = true;
    void load();
  } catch { reportContent.value = '生成失败，请确保已接入社交账号并完成数据采集。'; showResult.value = true; }
  genLoading.value = false;
}

async function load() {
  loading.value = true;
  try { const data = await insightApi.list('') as unknown as Array<Record<string, unknown>>; reports.value = Array.isArray(data) ? data : []; } catch { void 0; }
  loading.value = false;
}
onMounted(() => void load());
</script>
