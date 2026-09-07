import { defineStore } from 'pinia';
import { ref } from 'vue';
import { contentApi, type ContentItem } from '@/api/contents';

export const useContentStore = defineStore('content', () => {
  const contents = ref<ContentItem[]>([]);
  const current = ref<ContentItem | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchList(params?: Record<string, unknown>) {
    loading.value = true;
    error.value = null;
    try {
      contents.value = await contentApi.list(params);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载内容列表失败';
    } finally {
      loading.value = false;
    }
  }

  async function fetchDetail(id: string) {
    loading.value = true;
    error.value = null;
    try {
      current.value = await contentApi.get(id);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载内容详情失败';
    } finally {
      loading.value = false;
    }
  }

  return { contents, current, loading, error, fetchList, fetchDetail };
});
