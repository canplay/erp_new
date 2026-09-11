import { defineStore } from 'pinia';
import { ref } from 'vue';
import { statsApi } from '@/api/stats';

export const useStatsStore = defineStore('stats', () => {
  const overview = ref<Record<string, unknown> | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchOverview() {
    loading.value = true;
    error.value = null;
    try {
      overview.value = await statsApi.overview();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载统计总览失败';
    } finally {
      loading.value = false;
    }
  }

  return { overview, loading, error, fetchOverview };
});
