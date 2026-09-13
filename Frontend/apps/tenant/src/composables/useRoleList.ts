/**
 * @file useRoleList.ts
 * @description 角色管理页面业务逻辑 composable
 */

import { ref, reactive, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import {
  listAdminRoles as listRoles,
  updateAdminRolePermissions as updateRolePermissions,
  deleteAdminRole as deleteRoleApi,
} from '@/api';
import type { Role } from '@/types/user';
import { useDictionaryStore } from '@/stores/system';

export function useRoleList() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const dictionaryStore = useDictionaryStore();

  const loading = ref(false);
  const roles = ref<Role[]>([]);

  const advancedFilters = ref<{
    keyword: string;
    start_date: string;
    end_date: string;
    status: string | number | null;
  }>({ keyword: '', start_date: '', end_date: '', status: null });

  const columns = computed(() => [
    { name: 'name', label: $t('role.role_name'), field: 'name', align: 'left' as const, sortable: true },
    { name: 'description', label: $t('role.description'), field: 'description', align: 'left' as const },
    { name: 'type', label: $t('role.roleType'), field: 'type', align: 'center' as const, sortable: true },
    { name: 'user_count', label: $t('role.user_count'), field: 'user_count', align: 'center' as const, sortable: true },
    { name: 'actions', label: $t('role.actions'), field: 'actions', align: 'center' as const },
  ]);

  const pagination = reactive({ page: 1, rowsPerPage: 10, rowsNumber: 0 });

  const showDelete = ref(false);
  const deleteTarget = ref<Role | null>(null);

  const showPermissionDialog = ref(false);
  const currentRole = ref<Role | null>(null);
  const selectedPermissions = ref<string[]>([]);
  const savingPermissions = ref(false);

  const showCreateDialog = ref(false);
  const creating = ref(false);
  const createForm = reactive({ name: '', description: '', type: 'user' });

  // Role type options — static data, not reactive
  let roleTypeOptions: { label: string; value: string }[] = [
    { label: $t('user.normalUser'), value: 'user' },
    { label: $t('user.vip'), value: 'vip' },
    { label: $t('user.admin'), value: 'admin' },
  ] as const;

  async function loadRoleTypeOptions() {
    try {
      const types = await dictionaryStore.fetchRoleTypes();
      if (types && types.length > 0) roleTypeOptions = types;
    } catch (error) {
      console.error('【加载角色类型选项失败】', error);
    }
  }

  interface PermissionGroup {
    key: string;
    label: string;
    icon: string;
    permissions: { action: string; value: string }[];
  }

  function getPermissionLabel(action: string, groupLabel: string): string {
    const actionKey = `role.permissionActions.${action}` as const;
    return `${groupLabel} - ${$t(actionKey)}`;
  }

  const permissionGroups = ref<PermissionGroup[]>([
    { key: 'user', label: $t('role.permissionGroups.user'), icon: 'people', permissions: [
      { action: 'list', value: 'user:list' }, { action: 'view', value: 'user:view' },
      { action: 'create', value: 'user:create' }, { action: 'edit', value: 'user:edit' },
      { action: 'delete', value: 'user:delete' }, { action: 'resetPassword', value: 'user:reset-password' },
      { action: 'changeRole', value: 'user:change-role' },
    ]},
    { key: 'role', label: $t('role.permissionGroups.role'), icon: 'admin_panel_settings', permissions: [
      { action: 'list', value: 'role:list' }, { action: 'create', value: 'role:create' },
      { action: 'edit', value: 'role:edit' }, { action: 'delete', value: 'role:delete' },
      { action: 'assignPermissions', value: 'role:assign-permissions' },
    ]},
    { key: 'log', label: $t('role.permissionGroups.log'), icon: 'history', permissions: [
      { action: 'loginList', value: 'log:login:list' }, { action: 'operationList', value: 'log:operation:list' },
      { action: 'export', value: 'log:export' },
    ]},
    { key: 'system', label: $t('role.permissionGroups.system'), icon: 'settings', permissions: [
      { action: 'settings', value: 'system:settings' }, { action: 'info', value: 'system:info' },
      { action: 'backup', value: 'system:backup' },
    ]},
    { key: 'profile', label: $t('role.permissionGroups.profile'), icon: 'person', permissions: [
      { action: 'view', value: 'profile:view' }, { action: 'edit', value: 'profile:edit' },
      { action: 'resetPassword', value: 'profile:change-password' },
    ]},
  ]);

  function getRoleTypeLabel(type: string): string {
    const labels: Record<string, string> = { admin: $t('user.admin'), vip: $t('user.vip'), user: $t('user.normalUser') };
    return labels[type] || type;
  }

  const selectAll = computed({
    get: () => permissionGroups.value.flatMap((g) => g.permissions.map((p) => p.value)).every((p) => selectedPermissions.value.includes(p)),
    set: () => {},
  });

  function toggleSelectAll(value: boolean) {
    if (value) {
      selectedPermissions.value = permissionGroups.value.flatMap((g) => g.permissions.map((p) => p.value));
    } else {
      selectedPermissions.value = [];
    }
  }

  function openPermissionDialog(role: Role) {
    currentRole.value = role;
    selectedPermissions.value = role.permissions ? [...role.permissions] : [];
    showPermissionDialog.value = true;
  }

  async function savePermissions() {
    if (!currentRole.value) return;
    savingPermissions.value = true;
    try {
      await updateRolePermissions(currentRole.value.name, selectedPermissions.value);
      $q.notify({ type: 'positive', message: $t('common.success') });
      showPermissionDialog.value = false;
      void loadRoles();
    } catch (error) {
      console.error('【保存权限失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally {
      savingPermissions.value = false;
    }
  }

  async function loadRoles() {
    loading.value = true;
    try {
      const response = await listRoles();
      const respData = response as { data?: { list?: Role[]; total?: number }; list?: Role[]; total?: number };
      // 后端契约: { code, data: { list: Role[], total, page, page_size } }
      // alova transformResponse 已把 { data: { list, total } } 拍平到顶层 list/total
      const dataList: Role[] = respData.data?.list || respData.list || [];
      const total = respData.data?.total ?? respData.total ?? 0;
      let allRoles: Role[] = dataList.map((r) => {
        const apiType = (r as { type?: string }).type;
        const role: Role = {
          name: r.name,
          type: apiType || 'user',
          is_predefined: apiType === 'system',
          user_count: (r as { user_count?: number }).user_count ?? 0,
          permissions: r.permissions || [],
        };
        if (r.description) role.description = r.description;
        const status = r.status;
        if (status !== undefined) role.status = status;
        const created = r.created_at;
        if (created !== undefined) role.created_at = created;
        return role;
      });
      if (advancedFilters.value.keyword) {
        const kw = advancedFilters.value.keyword.toLowerCase();
        allRoles = allRoles.filter((r) => r.name.toLowerCase().includes(kw) || (r.description && r.description.toLowerCase().includes(kw)));
      }
      const start = (pagination.page - 1) * pagination.rowsPerPage;
      roles.value = allRoles.slice(start, start + pagination.rowsPerPage);
      pagination.rowsNumber = total || allRoles.length;
    } catch (error) {
      console.error('【加载角色失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally {
      loading.value = false;
    }
  }

  function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
    pagination.page = props.pagination.page;
    pagination.rowsPerPage = props.pagination.rowsPerPage;
    void loadRoles();
  }

  function handleAdvancedSearch() { pagination.page = 1; void loadRoles(); }
  function handleSearchReset() { pagination.page = 1; void loadRoles(); }

  function confirmDelete(role: Role) { deleteTarget.value = role; showDelete.value = true; }

  async function doDelete() {
    if (!deleteTarget.value) return;
    try {
      await deleteRoleApi(deleteTarget.value.name);
      $q.notify({ type: 'positive', message: $t('common.success') });
      showDelete.value = false;
      void loadRoles();
    } catch (error) {
      console.error('【删除角色失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function openCreateDialog() {
    createForm.name = '';
    createForm.description = '';
    createForm.type = 'user';
    showCreateDialog.value = true;
  }

  async function handleCreate() {
    if (!createForm.name) return;
    creating.value = true;
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      $q.notify({ type: 'positive', message: $t('common.success') });
      showCreateDialog.value = false;
      void loadRoles();
    } catch (error) {
      console.error('【创建角色失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally {
      creating.value = false;
    }
  }

  return {
    loading, roles, advancedFilters, columns, pagination, selectAll,
    showDelete, deleteTarget,
    showPermissionDialog, currentRole, selectedPermissions, savingPermissions,
    permissionGroups, showCreateDialog, creating, createForm, roleTypeOptions,
    loadRoleTypeOptions, getPermissionLabel, getRoleTypeLabel,
    toggleSelectAll, openPermissionDialog, savePermissions,
    loadRoles, onTableRequest, handleAdvancedSearch, handleSearchReset,
    confirmDelete, doDelete, openCreateDialog, handleCreate,
  };
}
