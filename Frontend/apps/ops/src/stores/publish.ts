import { defineStore } from 'pinia';
import { ref } from 'vue';
import { publishApi } from '@/api/publish';

export const usePublishStore = defineStore('publish', () => {
  const tasks = ref<Array<Record<string, unknown>>>([]);
  const schedules = ref<Array<Record<string, unknown>>>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchTasks() {
    loading.value = true;
    error.value = null;
    try {
      tasks.value = await publishApi.listTasks();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载发布任务失败';
    } finally {
      loading.value = false;
    }
  }

  async function fetchSchedules() {
    loading.value = true;
    error.value = null;
    try {
      schedules.value = await publishApi.listSchedules();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载定时计划失败';
    } finally {
      loading.value = false;
    }
  }

  return { tasks, schedules, loading, error, fetchTasks, fetchSchedules };
});
