/**
 * @file useFileManager.ts
 * @description FileManagerPage 业务逻辑 composable
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar'
import { logger } from '@/utils/logger';
import { useI18n } from 'vue-i18n';
import { useFileStore } from '@/stores/file';
import type { FileInfo } from '@/api/file';

export function useFileManager() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const fileStore = useFileStore();

  const searchKeyword = ref('');
  const fileInputRef = ref<HTMLInputElement | null>(null);
  const selectedTableRows = ref<FileInfo[]>([]);

  const showNewFolderDialog = ref(false);
  const newFolderName = ref('');

  const showRenameDialog = ref(false);
  const renameValue = ref('');
  const renamingItem = ref<FileInfo | null>(null);

  const showMoveDialog = ref(false);
  const targetFolderId = ref<number | null>(null);
  const movingItem = ref<FileInfo | null>(null);

  const showShareDialog = ref(false);
  const sharePassword = ref('');
  const shareExpiryDays = ref(7);
  const shareUrl = ref('');

  const showUploadProgress = ref(false);
  const uploadProgress = ref(0);

  const columns = computed(() => [
    { name: 'name', label: $t('file.name'), field: 'name', align: 'left' as const },
    { name: 'size', label: $t('file.size'), field: 'size', align: 'left' as const },
    { name: 'created_at', label: $t('file.created_at'), field: 'created_at', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  const tableRows = computed(() => [...fileStore.folders, ...fileStore.files]);

  const folderOptions = computed(() => [
    { label: $t('file.rootFolder'), value: null as number | null },
    ...fileStore.folders.map((f) => ({ label: f.name, value: f.id })),
  ]);

  const expiryOptions = [
    { label: `1 ${$t('file.day')}`, value: 1 },
    { label: `7 ${$t('file.days')}`, value: 7 },
    { label: `30 ${$t('file.days')}`, value: 30 },
    { label: $t('file.never'), value: 0 },
  ];

  function getFileIcon(item: FileInfo): string {
    if (item.isFolder) return 'folder';
    if (item.isImage) return 'image';
    if (item.mimeType?.includes('pdf')) return 'picture_as_pdf';
    if (item.mimeType?.includes('word') || item.mimeType?.includes('document')) return 'description';
    if (item.mimeType?.includes('excel') || item.mimeType?.includes('spreadsheet')) return 'table_chart';
    if (item.mimeType?.includes('video')) return 'video_file';
    if (item.mimeType?.includes('audio')) return 'audio_file';
    if (item.mimeType?.includes('zip') || item.mimeType?.includes('rar') || item.mimeType?.includes('tar')) return 'folder_zip';
    return 'insert_drive_file';
  }

  function getFileIconColor(item: FileInfo): string {
    if (item.isFolder) return 'warning';
    if (item.isImage) return 'positive';
    return 'grey';
  }

  function handleRowClick(_evt: Event, row: FileInfo) {
    if (row.isFolder) {
      void fileStore.enterFolder({
        id: row.id, name: row.name, path: row.folderPath || '',
        fileCount: 0, childCount: 0,
        created_at: row.created_at, updated_at: row.updated_at, created_by: row.created_by,
      });
    }
  }

  function handleCopy(item: FileInfo) {
    void item;
    $q.notify({ type: 'info', message: $t('file.copySuccess') });
  }

  async function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;
    showUploadProgress.value = true;
    uploadProgress.value = 0;
    try {
      for (const file of Array.from(input.files)) {
        await fileStore.upload(file, fileStore.currentFolderId);
      }
      $q.notify({ type: 'positive', message: $t('file.uploadSuccess') });
    } catch {
      $q.notify({ type: 'negative', message: $t('file.uploadFailed') });
    } finally {
      showUploadProgress.value = false;
      input.value = '';
    }
  }

  async function handleCreateFolder() {
    if (!newFolderName.value.trim()) return;
    try {
      await fileStore.createNewFolder(newFolderName.value.trim(), fileStore.currentFolderId);
      newFolderName.value = '';
      showNewFolderDialog.value = false;
      $q.notify({ type: 'positive', message: $t('file.createFolderSuccess') });
    } catch (error) {
      logger.error('【创建文件夹失败】', error);
      $q.notify({ type: 'negative', message: $t('file.createFolderFailed') });
    }
  }

  function handleRename(item: FileInfo) {
    renamingItem.value = item;
    renameValue.value = item.name;
    showRenameDialog.value = true;
  }

  async function handleDoRename() {
    if (!renamingItem.value || !renameValue.value.trim()) return;
    try {
      await fileStore.rename(renamingItem.value.id, renameValue.value.trim());
      showRenameDialog.value = false;
      $q.notify({ type: 'positive', message: $t('file.renameSuccess') });
    } catch (error) {
      logger.error('【重命名失败】', error);
      $q.notify({ type: 'negative', message: $t('file.renameFailed') });
    }
  }

  function handleMove(item: FileInfo) {
    movingItem.value = item;
    targetFolderId.value = null;
    showMoveDialog.value = true;
  }

  async function handleDoMove() {
    if (!movingItem.value) return;
    try {
      await fileStore.move(movingItem.value.id, targetFolderId.value ?? undefined);
      showMoveDialog.value = false;
      $q.notify({ type: 'positive', message: $t('file.moveSuccess') });
    } catch (error) {
      logger.error('【移动文件夹失败】', error);
      $q.notify({ type: 'negative', message: $t('file.moveFailed') });
    }
  }

  function handleGoBack() { void fileStore.goBack(); }
  function handleRefresh() { void fileStore.loadFiles(fileStore.currentFolderId); }
  function handleClearSelection() { void fileStore.clearSelection(); }

  function handleDelete(item: FileInfo) {
    $q.dialog({ title: $t('common.confirm'), message: $t('file.deleteConfirm', { name: item.name }), cancel: true, persistent: true })
      .onOk(() => {
        void (async () => {
          try { await fileStore.remove(item.id); $q.notify({ type: 'positive', message: $t('file.deleteSuccess') }); }
          catch { $q.notify({ type: 'negative', message: $t('file.deleteFailed') }); }
        })();
      });
  }

  function handleBatchDelete() {
    $q.dialog({ title: $t('common.confirm'), message: $t('file.batchDeleteConfirm', { count: fileStore.selectionCount }), cancel: true, persistent: true })
      .onOk(() => {
        void (async () => {
          try { await fileStore.removeMultiple(fileStore.selectedIds); $q.notify({ type: 'positive', message: $t('file.deleteSuccess') }); }
          catch { $q.notify({ type: 'negative', message: $t('file.deleteFailed') }); }
        })();
      });
  }

  function handleView(item: FileInfo) {
    if (item.isImage && item.url) window.open(item.url, '_blank');
    else $q.notify({ type: 'info', message: item.name });
  }

  async function handleDownload(item: FileInfo) {
    try { const url = await fileStore.getDownloadLink(item.id); window.open(url, '_blank'); }
    catch { $q.notify({ type: 'negative', message: $t('file.downloadFailed') }); }
  }

  function handleBatchDownload() { for (const id of fileStore.selectedIds) void handleDownload({ id } as FileInfo); }

  async function handleCreateShare() {
    try {
      const result = await fileStore.share(fileStore.selectedIds, { expiryDays: shareExpiryDays.value, ...(sharePassword.value ? { password: sharePassword.value } : {}) });
      shareUrl.value = (result as { shareUrl?: string })?.shareUrl || '';
    } catch { $q.notify({ type: 'negative', message: $t('file.shareFailed') }); }
  }

  function copyShareUrl() { void navigator.clipboard.writeText(shareUrl.value); $q.notify({ type: 'positive', message: $t('file.urlCopied') }); }

  return {
    fileStore, searchKeyword, fileInputRef, selectedTableRows,
    showNewFolderDialog, newFolderName,
    showRenameDialog, renameValue, renamingItem,
    showMoveDialog, targetFolderId, movingItem,
    showShareDialog, sharePassword, shareExpiryDays, shareUrl,
    showUploadProgress, uploadProgress,
    columns, tableRows, folderOptions, expiryOptions,
    getFileIcon, getFileIconColor,
    handleRowClick, handleCopy, handleFileSelect, handleCreateFolder,
    handleRename, handleDoRename, handleMove, handleDoMove,
    handleGoBack, handleRefresh, handleClearSelection,
    handleDelete, handleBatchDelete, handleView, handleDownload, handleBatchDownload,
    handleCreateShare, copyShareUrl,
  };
}
