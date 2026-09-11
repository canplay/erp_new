import { defineStore } from 'pinia';
import { ref } from 'vue';
import { insightApi, type InsightReport } from '@/api/insights';

export const useInsightStore = defineStore('insights', () => {
  const reports = ref<InsightReport[]>([]);
  const current = ref<InsightReport | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchReports(accountId: string) {
    loading.value = true;
    error.value = null;
    try {
      reports.value = await insightApi.list(accountId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载分析报告失败';
    } finally {
      loading.value = false;
    }
  }

  return { reports, current, loading, error, fetchReports };
});
