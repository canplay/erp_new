import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface ExportTask {
  id: string;
  label: string;
  /** pending = 服务端生成中; ready = 可下载; failed = 失败 */
  status: 'pending' | 'ready' | 'failed';
  /** 就绪后的下载地址（blob URL 或服务端文件 URL） */
  url: string | undefined;
  createdAt: number;
}

/**
 * 异步导出/下载任务中心。
 * 企业级导出（报表、监管记录、发票 PDF）通常耗时，前端发起后入队，
 * 完成后在侧边栏"下载中心"提示，避免用户阻塞等待或重复点击。
 * 当前对接后端同步导出端点：请求返回后短暂 pending 即标记 ready。
 */
export const useExportTasksStore = defineStore('exportTasks', () => {
  const tasks = ref<ExportTask[]>([]);

  const pendingCount = computed(() => tasks.value.filter((t) => t.status === 'pending').length);
  const readyTasks = computed(() => tasks.value.filter((t) => t.status === 'ready'));

  function enqueue(label: string): ExportTask {
    const task: ExportTask = {
      id: `exp_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
      label,
      status: 'pending',
      url: undefined,
      createdAt: Date.now(),
    };
    tasks.value.unshift(task);
    return task;
  }

  function complete(id: string, url?: string) {
    const t = tasks.value.find((x) => x.id === id);
    if (t) {
      t.status = url ? 'ready' : 'failed';
      t.url = url;
    }
  }

  function dismiss(id: string) {
    tasks.value = tasks.value.filter((x) => x.id !== id);
  }

  return { tasks, pendingCount, readyTasks, enqueue, complete, dismiss };
});
