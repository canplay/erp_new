/**
 * @file useUserActions.ts
 * @description User actions: delete, reset password, batch operations, import template
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import type { User } from '@/api/user';
import { useExport } from '@erp-new-frontend-monorepo/composables/src/useExport';;

export function useUserActions(onLoadUsers: () => Promise<void>) {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const { exportToCSV } = useExport();
const exportData = exportToCSV;

  const showDeleteConfirmDialog = ref(false);
  const pendingDeleteUser = ref<User | null>(null);

  async function handleDelete(user: User) {
    pendingDeleteUser.value = user;
    showDeleteConfirmDialog.value = true;
  }

  async function doDeleteUser(onSuccess: () => void) {
    if (!pendingDeleteUser.value) return;
    try {
      const { deleteUser } = await import('@/api/user');
      await deleteUser(pendingDeleteUser.value.id);
      $q.notify({ type: 'positive', message: $t('common.success') });
      pendingDeleteUser.value = null;
      onSuccess();
    } catch (error) {
      console.error('【删除失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function handleBatchEnable(items: unknown[]): Promise<void> {
    const user_ids = (items as User[]).map((u) => u.id);
    try {
      const { batchUpdateUserStatus } = await import('@/api/user');
      await batchUpdateUserStatus({ user_ids, status: 1 });
      $q.notify({ type: 'positive', message: $t('batchActions.enableSuccess', { count: user_ids.length }) });
    } catch (error) {
      console.error('【批量启用失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  async function handleBatchDisable(items: unknown[]): Promise<void> {
    const user_ids = (items as User[]).map((u) => u.id);
    try {
      const { batchUpdateUserStatus } = await import('@/api/user');
      await batchUpdateUserStatus({ user_ids, status: 0 });
      $q.notify({ type: 'positive', message: $t('batchActions.disableSuccess', { count: user_ids.length }) });
    } catch (error) {
      console.error('【批量禁用失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  async function handleBatchDelete(items: unknown[]): Promise<void> {
    const user_ids = (items as User[]).map((u) => u.id);
    try {
      const { batchDeleteUsers } = await import('@/api/user');
      await batchDeleteUsers(user_ids);
      $q.notify({ type: 'positive', message: $t('batchActions.deleteSuccess', { count: user_ids.length }) });
    } catch (error) {
      console.error('【批量删除失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  function handleBatchExport(items: unknown[]) {
    const usersToExport = items as User[];
    try {
      void exportData(usersToExport, [], 'users.csv');
      $q.notify({ type: 'positive', message: $t('user.exportSuccess') });
    } catch (error) {
      console.error('【导出失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function downloadTemplate() {
    try {
      const { getUserImportTemplate } = await import('@/api/user');
      const response = await getUserImportTemplate();
      const blob = new Blob([response as unknown as BlobPart], { type: 'text/csv' });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = 'user_import_template.csv';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);
      $q.notify({ type: 'positive', message: $t('user.templateDownloaded') });
    } catch (error) {
      console.error('【下载模板失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function onFileRejected(rejectedEntries: { failedPropValidation: string }[]) {
    rejectedEntries.forEach((entry) => {
      let message = $t('user.fileRejected');
      if (entry.failedPropValidation === 'max-file-size') {
        message = $t('user.fileTooLarge');
      } else if (entry.failedPropValidation === 'accept') {
        message = $t('user.fileTypeError');
      }
      $q.notify({ type: 'negative', message });
    });
  }

  return {
    showDeleteConfirmDialog,
    pendingDeleteUser,
    handleDelete,
    doDeleteUser,
    handleBatchEnable,
    handleBatchDisable,
    handleBatchDelete,
    handleBatchExport,
    downloadTemplate,
    onFileRejected,
  };
}
