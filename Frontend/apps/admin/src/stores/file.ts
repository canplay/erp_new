/**
 * @file file.ts
 * @description 文件状态管理
 * @date 2026-04-04
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
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




