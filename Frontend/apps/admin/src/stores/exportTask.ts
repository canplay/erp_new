/**
 * @file exportTask.ts
 * @description 导出任务状态管理
 * @date 2026-04-04
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { httpClient } from '@/utils/alova';
import { logger } from '@/utils/logger';
import type {
  ExportTask,
  ExportTaskParams,
  ExportTaskStatus,
} from '@/types/importExport';

/**
 * @brief 导出任务状态管理 Store
 */
export const useExportTaskStore = defineStore('exportTask', () => {
  // ============ 状态 ============

  /** 导出任务列表 */
  const tasks = ref<ExportTask[]>([]);

  /** 加载状态 */
  const loading = ref(false);

  /** 当前正在执行的任务 */
  const currentTaskId = ref<string | null>(null);

  // ============ 计算属性 ============

  /** 待处理任务 */
  const pendingTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'pending')
  );

  /** 正在处理的任务 */
  const processingTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'processing')
  );

  /** 已完成的任务 */
  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'completed')
  );

  /** 失败的任务 */
  const failedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'failed')
  );

  /** 当前任务 */
  const currentTask = computed(() =>
    tasks.value.find((t) => t.id === currentTaskId.value)
  );

  // ============ Actions ============

  /**
   * @brief 创建导出任务
   * @param params 导出参数
   * @returns 创建的任务
   */
  async function createTask(params: ExportTaskParams): Promise<ExportTask> {
    try {
      const response = await httpClient.post('/export/tasks', params);
      const task = (response as { data?: ExportTask }).data!;
      tasks.value.unshift(task);
      return task;
    } catch (error) {
      logger.error('【创建导出任务失败】', error);
      throw error;
    }
  }

  /**
   * @brief 获取任务列表
   */
  async function fetchTasks(params?: { status?: ExportTaskStatus; limit?: number }) {
    loading.value = true;
    try {
      const response = await httpClient.get('/export/tasks', { params });
      const respData = response as { list?: ExportTask[]; data?: ExportTask[] | { list?: ExportTask[] } };
      const dataObj = respData.data as { list?: ExportTask[] } | undefined;
      tasks.value = respData.list || dataObj?.list || (Array.isArray(respData.data) ? (respData.data) : []) || [];
    } catch (error) {
      logger.error('【获取导出任务列表失败】', error);
      tasks.value = [];
    } finally {
      loading.value = false;
    }
  }

  /**
   * @brief 获取任务详情
   */
  async function fetchTask(taskId: string): Promise<ExportTask | null> {
    try {
      const response = await httpClient.get(`/export/tasks/${taskId}`);
      const task = (response as { data?: ExportTask }).data!;

      // 更新本地列表
      const index = tasks.value.findIndex((t) => t.id === taskId);
      if (index > -1) {
        tasks.value[index] = task;
      } else {
        tasks.value.unshift(task);
      }

      return task;
    } catch (error) {
      logger.error('【获取导出任务详情失败】', error);
      return null;
    }
  }

  /**
   * @brief 更新任务状态（轮询或 WebSocket 更新时调用）
   */
  function updateTask(taskId: string, updates: Partial<ExportTask>) {
    const index = tasks.value.findIndex((t) => t.id === taskId);
    if (index > -1) {
      const filteredUpdates = Object.fromEntries(
        Object.entries(updates).filter(([, v]) => v !== undefined)
      ) as Partial<ExportTask>;
      tasks.value[index] = { ...tasks.value[index], ...filteredUpdates } as ExportTask;
    }
  }

  /**
   * @brief 轮询任务状态
   */
  function startPolling(taskId: string, interval = 2000) {
    const pollTimer = setInterval(() => {
      void (async () => {
        const task = await fetchTask(taskId);
        if (task) {
          if (task.status === 'completed' || task.status === 'failed' || task.status === 'cancelled') {
            clearInterval(pollTimer);
          }
        } else {
          clearInterval(pollTimer);
        }
      })();
    }, interval);

    return pollTimer;
  }

  /**
   * @brief 取消任务
   */
  async function cancelTask(taskId: string): Promise<boolean> {
    try {
      await httpClient.post(`/export/tasks/${taskId}/cancel`);
      updateTask(taskId, { status: 'cancelled' });
      return true;
    } catch (error) {
      logger.error('【取消导出任务失败】', error);
      return false;
    }
  }

  /**
   * @brief 删除任务
   */
  async function deleteTask(taskId: string): Promise<boolean> {
    try {
      await httpClient.delete(`/export/tasks/${taskId}`);
      tasks.value = tasks.value.filter((t) => t.id !== taskId);
      return true;
    } catch (error) {
      logger.error('【删除导出任务失败】', error);
      return false;
    }
  }

  /**
   * @brief 下载导出文件
   */
  async function downloadFile(taskId: string): Promise<string | null> {
    try {
      const task = tasks.value.find((t) => t.id === taskId);
      if (!task?.downloadUrl) {
        // 如果没有下载链接，尝试获取
        const freshTask = await fetchTask(taskId);
        if (!freshTask?.downloadUrl) {
          logger.error('【导出任务】没有可用的下载链接');
          return null;
        }
        return freshTask.downloadUrl;
      }
      return task.downloadUrl;
    } catch (error) {
      logger.error('【获取导出文件下载地址失败】', error);
      return null;
    }
  }

  /**
   * @brief 清理过期任务
   */
  async function cleanupExpiredTasks(): Promise<number> {
    try {
      const response = await httpClient.post('/export/tasks/cleanup');
      const deletedCount = (response as { data?: { deletedCount: number } }).data?.deletedCount || 0;

      // 移除过期的本地任务
      const now = new Date();
      tasks.value = tasks.value.filter((t) => {
        if (t.expires_at) {
          return new Date(t.expires_at) > now;
        }
        return true;
      });

      return deletedCount;
    } catch (error) {
      logger.error('【清理过期任务失败】', error);
      return 0;
    }
  }

  /**
   * @brief 清除所有任务
   */
  function clearAllTasks() {
    tasks.value = [];
    currentTaskId.value = null;
  }

  /**
   * @brief 设置当前任务
   */
  function setCurrentTask(taskId: string | null) {
    currentTaskId.value = taskId;
  }

  return {
    // 状态
    tasks,
    loading,
    currentTaskId,

    // 计算属性
    pendingTasks,
    processingTasks,
    completedTasks,
    failedTasks,
    currentTask,

    // Actions
    createTask,
    fetchTasks,
    fetchTask,
    updateTask,
    startPolling,
    cancelTask,
    deleteTask,
    downloadFile,
    cleanupExpiredTasks,
    clearAllTasks,
    setCurrentTask,
  };
});

