import { defineStore } from 'pinia';
import { ref } from 'vue';
import { crawlApi, type CrawlSource } from '@/api/crawl';

export const useCrawlStore = defineStore('crawl', () => {
  const sources = ref<CrawlSource[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchSources() {
    loading.value = true;
    error.value = null;
    try {
      sources.value = await crawlApi.listSources();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载抓取源失败';
    } finally {
      loading.value = false;
    }
  }

  async function createSource(data: Record<string, unknown>) {
    error.value = null;
    try {
      const result = await crawlApi.createSource(data);
      sources.value.push(result);
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建抓取源失败';
      throw e;
    }
  }

  async function deleteSource(id: string) {
    error.value = null;
    try {
      await crawlApi.deleteSource(id);
      sources.value = sources.value.filter((s) => s.id !== id);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除抓取源失败';
      throw e;
    }
  }

  return { sources, loading, error, fetchSources, createSource, deleteSource };
});
