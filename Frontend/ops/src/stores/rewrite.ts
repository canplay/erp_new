import { defineStore } from 'pinia';
import { ref } from 'vue';
import { rewriteApi } from '@/api/rewrite';

export const useRewriteStore = defineStore('rewrite', () => {
  const tasks = ref<Array<Record<string, unknown>>>([]);
  const currentTask = ref<Record<string, unknown> | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchTasks() {
    loading.value = true;
    error.value = null;
    try {
      tasks.value = await rewriteApi.listTasks();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载洗文任务失败';
    } finally {
      loading.value = false;
    }
  }

  async function fetchTask(id: string) {
    loading.value = true;
    error.value = null;
    try {
      currentTask.value = await rewriteApi.getTask(id);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载洗文详情失败';
    } finally {
      loading.value = false;
    }
  }

  return { tasks, currentTask, loading, error, fetchTasks, fetchTask };
});
