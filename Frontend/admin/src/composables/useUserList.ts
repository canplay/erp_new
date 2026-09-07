/**
 * @file useUserList.ts
 * @description UserListPage main composable - coordinates sub-composables
 * @date 2026-04-04
 */

import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useUserColumns } from './useUserColumns';
import { useUserForm } from './useUserForm';
import { useUserActions } from './useUserActions';
import { useUserListCore } from './useUserListCore';
import { useUserListPagination } from './useUserListPagination';
import type { User } from '@/api/user';

export function useUserList() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  const { loading, users, selectedUsers } = useUserListCore();
  const { filters, pagination, hasFilters, resetFilters, buildQueryParams } = useUserListPagination();

  const {
    columnDefinitions,
    displayedColumns,
    defaultVisibleColumns,
    visibleColumns,
    statusOptionsForSearch,
    roleOptionsForSearch,
    statusOptionsForDialog,
    roleOptionsForDialog,
    importPreviewColumns,
  } = useUserColumns();

  const {
    showUserDialog,
    isEdit,
    currentUser,
    userForm,
    showImportDialog,
    importStep,
    importFile,
    importPreview,
    selectedImportRows,
    importing,
    importResult,
    showDetailDialog,
    detailUser,
    validCount,
    errorCount,
    openUserDialog,
    viewUser,
    saveUser,
    handleUploadFile,
    handleStartImport,
    handleImportComplete,
  } = useUserForm();

  const {
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
  } = useUserActions(loadUsers);

  async function loadUsers() {
    loading.value = true;
    try {
      const { listUsers } = await import('@/api/user');
      const params = buildQueryParams();
      const response = await listUsers(params);
      const result = (response as { data?: { list?: User[]; users?: User[]; total?: number } }).data;
      users.value = result?.list || result?.users || [];
      pagination.value.rowsNumber = result?.total || 0;
    } catch (error) {
      console.error('【加载用户失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally {
      loading.value = false;
    }
  }

  function onTableRequest(props: { pagination: { page: number; rowsPerPage: number; sortBy?: string; descending?: boolean; rowsNumber?: number } }) {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    pagination.value.sortBy = props.pagination.sortBy || 'id';
    pagination.value.descending = props.pagination.descending ?? true;
    void loadUsers();
  }

  function handleSearch() {
    pagination.value.page = 1;
    void loadUsers();
  }

  function handleReset() {
    resetFilters();
    void loadUsers();
  }

  // Override form callbacks to use loadUsers
  const wrappedSaveUser = async () => {
    await saveUser(loadUsers);
  };

  const wrappedHandleUploadFile = async () => {
    await handleUploadFile(loadUsers);
  };

  const wrappedHandleStartImport = async () => {
    await handleStartImport(loadUsers);
  };

  const wrappedHandleImportComplete = async () => {
    await handleImportComplete(loadUsers);
  };

  const wrappedDoDeleteUser = async () => {
    await doDeleteUser(loadUsers);
  };

  // Batch operations need to clear selection and reload
  const wrappedBatchEnable = async (items: unknown[]) => {
    await handleBatchEnable(items);
    selectedUsers.value = [];
    await loadUsers();
  };

  const wrappedBatchDisable = async (items: unknown[]) => {
    await handleBatchDisable(items);
    selectedUsers.value = [];
    await loadUsers();
  };

  const wrappedBatchDelete = async (items: unknown[]) => {
    await handleBatchDelete(items);
    selectedUsers.value = [];
    await loadUsers();
  };

  // Status/role helper functions
  function getStatusLabel(status: number): string {
    return status === 1 ? $t('user.active') || '启用' : status === 0 ? $t('user.disabled') || '禁用' : String(status);
  }

  function getStatusColor(status: number): string {
    return status === 1 ? 'positive' : 'grey';
  }

  function getStatusIcon(status: number): string {
    return status === 1 ? 'check_circle' : 'cancel';
  }

  function getRoleLabel(role: string): string {
    return role || $t('user.normal') || '普通用户';
  }

  function getRoleChipColor(role: string): string {
    if (role === 'admin') return 'negative';
    if (role === 'editor') return 'primary';
    return 'grey';
  }

  return {
    // State
    loading,
    users,
    selectedUsers,
    filters,
    pagination,
    hasFilters,
    // Columns
    columnDefinitions,
    displayedColumns,
    defaultVisibleColumns,
    visibleColumns,
    statusOptionsForSearch,
    roleOptionsForSearch,
    statusOptionsForDialog,
    roleOptionsForDialog,
    importPreviewColumns,
    validCount,
    errorCount,
    // Status/role helpers
    getStatusLabel,
    getStatusColor,
    getStatusIcon,
    getRoleLabel,
    getRoleChipColor,
    // Dialog state
    showUserDialog,
    isEdit,
    currentUser,
    userForm,
    showImportDialog,
    importStep,
    importFile,
    importPreview,
    selectedImportRows,
    importing,
    importResult,
    showDeleteConfirmDialog,
    pendingDeleteUser,
    showDetailDialog,
    detailUser,
    // List operations
    loadUsers,
    onTableRequest,
    handleSearch,
    handleReset,
    // User CRUD
    openUserDialog,
    viewUser,
    saveUser: wrappedSaveUser,
    handleDelete,
    doDeleteUser: wrappedDoDeleteUser,
    // Import
    downloadTemplate,
    onFileRejected,
    handleUploadFile: wrappedHandleUploadFile,
    handleStartImport: wrappedHandleStartImport,
    handleImportComplete: wrappedHandleImportComplete,
    // Batch operations
    handleBatchEnable: wrappedBatchEnable,
    handleBatchDisable: wrappedBatchDisable,
    handleBatchDelete: wrappedBatchDelete,
    handleBatchExport,
  };
}
