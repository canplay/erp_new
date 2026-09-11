import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  listScheduledTasks,
  type ScheduledTask,
} from '@/api/scheduled-task';

export const useScheduledTaskStore = defineStore('scheduledTask', () => {
  const tasks = ref<ScheduledTask[]>([]);
  const total = ref(0);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchTasks(params?: Record<string, unknown>) {
    loading.value = true;
    error.value = null;
    try {
      const response = await listScheduledTasks(params ?? {});
      const data = response.data as { list?: ScheduledTask[]; total?: number };
      if (data?.list) {
        tasks.value = data.list;
        total.value = data.total ?? data.list.length;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载任务失败';
    } finally {
      loading.value = false;
    }
  }

  return { tasks, total, loading, error, fetchTasks };
});
