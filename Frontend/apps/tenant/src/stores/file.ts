/**
 * @file file.ts
 * @description 文件状态管理
 * @date 2026-04-04
 */

import {
  listFiles,
  createFolder,
  renameFile,
  moveFile,
  deleteFile,
  deleteFiles,
  uploadFile,
  getDownloadUrl,
  createShare,
  getStorageStats,
  type FileInfo,
  type FolderInfo,
} from '@/api/file';

export const useFileStore = defineStore('file', () => {
  // ============ 状态定义 ============

  /** 文件列表 */
  const files = ref<FileInfo[]>([]);

  /** 文件夹列表 */
  const folders = ref<FolderInfo[]>([]);

  /** 当前文件夹 ID */
  const currentFolderId = ref<number | undefined>(undefined);

  /** 当前路径面包屑 */
  const breadcrumbs = ref<{ id?: number; name: string }[]>([{ name: '全部文件' }]);

  /** 加载状态 */
  const loading = ref(false);

  /** 上传进度 */
  const uploadProgress = ref<Record<string, number>>({});

  /** 存储统计 */
  const storageStats = ref<{
    used: number;
    total: number;
    fileCount: number;
    folderCount: number;
  } | null>(null);

  /** 选中文件 */
  const selectedFiles = ref<FileInfo[]>([]);

  /** 搜索关键词 */
  const searchKeyword = ref('');

  // ============ 计算属性 ============

  /** 是否有选中文件 */
  const hasSelection = computed(() => selectedFiles.value.length > 0);

  /** 选中的文件数量 */
  const selectionCount = computed(() => selectedFiles.value.length);

  /** 选中的文件 ID 列表 */
  const selectedIds = computed(() => selectedFiles.value.map((f) => f.id));

  /** 存储使用百分比 */
  const storageUsedPercent = computed(() => {
    if (!storageStats.value) return 0;
    return Math.round((storageStats.value.used / storageStats.value.total) * 100);
  });

  /** 格式化存储大小 */
  const formattedStorage = computed(() => {
    if (!storageStats.value) return { used: '0 B', total: '0 B' };
    return {
      used: formatBytes(storageStats.value.used),
      total: formatBytes(storageStats.value.total),
    };
  });

  // ============ 方法 ============

  /**
   * @brief 加载文件列表
   */
  async function loadFiles(folderId?: number) {
    loading.value = true;
    currentFolderId.value = folderId;

    try {
      const response = await listFiles({
        ...(folderId !== undefined ? { folderId } : {}),
        ...(searchKeyword.value ? { keyword: searchKeyword.value } : {}),
      });
      const respData = response as { list?: unknown[]; data?: { list?: unknown[]; files?: FileInfo[]; folders?: FolderInfo[]; type?: string } };
      const dataObj = respData.data as { list?: unknown[]; files?: FileInfo[]; folders?: FolderInfo[]; type?: string } | undefined;
      const items = (respData.list || dataObj?.list || []) as Array<Record<string, unknown>>;

      // API 返回 {list, total, type}，其中 type 区分 file/folder
      const folderItems = items.filter((item) => item.type === 'folder' || item.is_dir);
      const fileItems = items.filter((item) => item.type !== 'folder' && !item.is_dir);
      files.value = (dataObj?.files || fileItems) as FileInfo[];
      folders.value = (dataObj?.folders || folderItems) as FolderInfo[];
    } catch (error) {
      logger.error('【加载文件列表失败】', error);
      files.value = [];
      folders.value = [];
    } finally {
      loading.value = false;
    }
  }

  /**
   * @brief 创建文件夹
   */
  async function createNewFolder(name: string, parent_id?: number) {
    try {
      await createFolder({ name, ...(parent_id !== undefined ? { parent_id } : {}) });
      await loadFiles(parent_id);
      return true;
    } catch (error) {
      logger.error('【创建文件夹失败】', error);
      throw error;
    }
  }

  /**
   * @brief 重命名文件/文件夹
   */
  async function rename(id: number, newName: string) {
    try {
      await renameFile(id, newName);
      await loadFiles(currentFolderId.value);
      return true;
    } catch (error) {
      logger.error('【重命名失败】', error);
      throw error;
    }
  }

  /**
   * @brief 移动文件/文件夹
   */
  async function move(id: number, targetFolderId?: number) {
    try {
      await moveFile(id, targetFolderId);
      await loadFiles(currentFolderId.value);
      return true;
    } catch (error) {
      logger.error('【移动失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除文件/文件夹
   */
  async function remove(id: number) {
    try {
      await deleteFile(id);
      await loadFiles(currentFolderId.value);
      return true;
    } catch (error) {
      logger.error('【删除失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量删除文件
   */
  async function removeMultiple(ids: number[]) {
    try {
      await deleteFiles(ids);
      selectedFiles.value = [];
      await loadFiles(currentFolderId.value);
      return true;
    } catch (error) {
      logger.error('【批量删除失败】', error);
      throw error;
    }
  }

  /**
   * @brief 上传文件
   */
  async function upload(file: File, folderId?: number, onProgress?: (percent: number) => void) {
    const fileId = `${Date.now()}-${file.name}`;
    uploadProgress.value[fileId] = 0;

    try {
      const formData = new FormData();
      formData.append('file', file);
      if (folderId !== undefined) {
        formData.append('folder_id', String(folderId));
      }

      await uploadFile(formData, (percent) => {
        uploadProgress.value[fileId] = percent;
        onProgress?.(percent);
      });

      delete uploadProgress.value[fileId];
      await loadFiles(currentFolderId.value);
      return true;
    } catch (error) {
      delete uploadProgress.value[fileId];
      logger.error('【上传失败】', error);
      throw error;
    }
  }

  /**
   * @brief 获取下载链接
   */
  async function getDownloadLink(id: number): Promise<string> {
    try {
      const response = await getDownloadUrl(id);
      return (response as { data?: { url?: string } }).data?.url || '';
    } catch (error) {
      logger.error('【获取下载链接失败】', error);
      throw error;
    }
  }

  /**
   * @brief 创建分享链接
   */
  async function share(fileIds: number[], options?: { password?: string; expiryDays?: number }) {
    try {
      const response = await createShare({
        fileIds,
        ...(options?.password !== undefined ? { password: options.password } : {}),
        ...(options?.expiryDays !== undefined ? { expiryDays: options.expiryDays } : {}),
      });
      return response.data;
    } catch (error) {
      logger.error('【创建分享失败】', error);
      throw error;
    }
  }

  /**
   * @brief 加载存储统计
   */
  async function loadStorageStats() {
    try {
      const response = await getStorageStats();
      storageStats.value = (response as { data?: { used: number; total: number; fileCount: number; folderCount: number } }).data || null;
    } catch (error) {
      logger.error('【加载存储统计失败】', error);
    }
  }

  /**
   * @brief 进入文件夹
   */
  async function enterFolder(folder: FolderInfo) {
    // 更新面包屑
    const existingIndex = breadcrumbs.value.findIndex((b) => b.id === folder.id);
    if (existingIndex >= 0) {
      breadcrumbs.value = breadcrumbs.value.slice(0, existingIndex + 1);
    } else {
      breadcrumbs.value.push({ id: folder.id, name: folder.name });
    }

    await loadFiles(folder.id);
  }

  /**
   * @brief 返回上一级
   */
  async function goBack() {
    if (breadcrumbs.value.length <= 1) return;

    breadcrumbs.value.pop();
    const parent = breadcrumbs.value[breadcrumbs.value.length - 1];
    await loadFiles(parent!.id);
  }

  /**
   * @brief 返回根目录
   */
  async function goToRoot() {
    breadcrumbs.value = [{ name: '全部文件' }];
    await loadFiles();
  }

  /**
   * @brief 选择文件
   */
  function selectFile(file: FileInfo) {
    selectedFiles.value.push(file);
  }

  /**
   * @brief 取消选择文件
   */
  function deselectFile(file: FileInfo) {
    const index = selectedFiles.value.findIndex((f) => f.id === file.id);
    if (index >= 0) {
      selectedFiles.value.splice(index, 1);
    }
  }

  /**
   * @brief 切换选择
   */
  function toggleSelection(file: FileInfo) {
    const index = selectedFiles.value.findIndex((f) => f.id === file.id);
    if (index >= 0) {
      selectedFiles.value.splice(index, 1);
    } else {
      selectedFiles.value.push(file);
    }
  }

  /**
   * @brief 全选
   */
  function selectAll() {
    selectedFiles.value = [...files.value, ...folders.value] as FileInfo[];
  }

  /**
   * @brief 清除选择
   */
  function clearSelection() {
    selectedFiles.value = [];
  }

  /**
   * @brief 格式化字节大小
   */
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  }

  /**
   * @brief 搜索文件
   */
  async function search(keyword: string) {
    searchKeyword.value = keyword;
    await loadFiles(currentFolderId.value);
  }

  /**
   * @brief 清除搜索
   */
  async function clearSearch() {
    searchKeyword.value = '';
    await loadFiles(currentFolderId.value);
  }

  return {
    // 状态
    files,
    folders,
    currentFolderId,
    breadcrumbs,
    loading,
    uploadProgress,
    storageStats,
    selectedFiles,
    searchKeyword,

    // 计算属性
    hasSelection,
    selectionCount,
    selectedIds,
    storageUsedPercent,
    formattedStorage,

    // 方法
    loadFiles,
    createNewFolder,
    rename,
    move,
    remove,
    removeMultiple,
    upload,
    getDownloadLink,
    share,
    loadStorageStats,
    enterFolder,
    goBack,
    goToRoot,
    selectFile,
    deselectFile,
    toggleSelection,
    selectAll,
    clearSelection,
    formatBytes,
    search,
    clearSearch,
  };
});






/**
 * @file exportTask.ts
 * @description 导出任务状态管理
 * @date 2026-04-04
 */

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

