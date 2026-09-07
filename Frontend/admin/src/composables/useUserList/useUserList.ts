/**
 * @file useUserList.ts
 * @description UserListPage main composable - coordinates sub-composables
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useUserColumns } from '../useUserColumns';
import { useUserForm } from '../useUserForm';
import { useUserActions } from '../useUserActions';
import type { User } from '@/api/user';

export function useUserList() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  const loading = ref(false);
  const users = ref<User[]>([]);
  const selectedUsers = ref<User[]>([]);

  const filters = ref({
    keyword: '',
    start_date: '',
    end_date: '',
    status: null as number | null,
    role: null as string | null,
  });

  const pagination = ref({
    page: 1,
    rowsPerPage: 10,
    rowsNumber: 0,
    sortBy: 'id',
    descending: true,
  });

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
      const params: Parameters<typeof listUsers>[0] = {};
      params.page = pagination.value.page;
      params.page_size = pagination.value.rowsPerPage;

      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.status !== null && filters.value.status !== undefined) {
        params.status = filters.value.status;
      }
      if (filters.value.role) params.role = filters.value.role;

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
    filters.value = {
      keyword: '',
      start_date: '',
      end_date: '',
      status: null,
      role: null,
    };
    pagination.value.page = 1;
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

  return {
    // State
    loading,
    users,
    selectedUsers,
    filters,
    pagination,
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
