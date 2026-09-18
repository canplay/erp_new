/**
 * @file useRolePermissionPage.ts
 * @description RolePermissionPage 业务逻辑 composable
 */

import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import type { QTableProps } from 'quasar';
import { PERMISSIONS, getAllModules, PermissionModule, type PermissionDefinition } from '@/types/permission';
import { listAdminRoles as listRoles, getAdminRolePermissions as getRolePermissions, updateAdminRolePermissions as updateRolePermissions } from '@/api';
import { listPermissions, createPermission, updatePermission, deletePermission } from '@/api/permission';

export function useRolePermissionPage() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  const roles = ref<{ label: string; value: string }[]>([]);
  const selectedRole = ref<string | null>(null);
  const saving = ref(false);
  const rolePermissions = ref<string[]>([]);

  const permissionGroups = ref<{ module: PermissionModule; label: string; selected: boolean; permissions: { key: string; name: string; description?: string; selected: boolean }[] }[]>([]);

  const permissionDefinitions = ref<PermissionDefinition[]>([]);
  const loadingPermissions = ref(false);
  const permissionPagination = reactive({ page: 1, rowsPerPage: 20, rowsNumber: 0 });

  const showPermissionDialog = ref(false);
  const editingPermission = ref<PermissionDefinition | null>(null);
  const permissionForm = reactive<{ name: string; module: string | PermissionModule; category: 'page' | 'button' | 'field' | 'data'; description: string; sensitive: boolean }>({ name: '', module: PermissionModule.USER, category: 'button', description: '', sensitive: false });
  const savingPermission = ref(false);
  const showDeleteDialog = ref(false);
  const deletingPermission = ref<PermissionDefinition | null>(null);

  const roleOptions = computed(() => roles.value);
  const hasChanges = computed(() => {
    const current = getCurrentPermissions();
    if (current.length !== rolePermissions.value.length) return true;
    return [...current].sort().some((p, i) => p !== [...rolePermissions.value].sort()[i]);
  });
  const is_read_only = computed(() => selectedRole.value === 'admin');

  const permissionColumns: QTableProps['columns'] = [
    { name: 'key', label: '权限标识', field: 'key', align: 'left' as const, sortable: true },
    { name: 'name', label: '权限名称', field: 'name', align: 'left' as const, sortable: true },
    { name: 'module', label: '所属模块', field: 'module', align: 'center' as const },
    { name: 'category', label: '类别', field: 'category', align: 'center' as const },
    { name: 'description', label: '描述', field: 'description', align: 'left' as const },
    { name: 'sensitive', label: '敏感权限', field: 'sensitive', align: 'center' as const },
    { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
  ];

  const moduleOptions = getAllModules();
  const categoryOptions = [{ label: '页面', value: 'page' }, { label: '按钮', value: 'button' }, { label: '字段', value: 'field' }, { label: '数据', value: 'data' }];

  async function loadRoles() {
    try {
      const response = await listRoles();
      const data = (response.data as { roles?: string[]; predefined_roles?: string[] }) || { roles: [], predefined_roles: [] };
      roles.value = (data.roles || []).map((role_name: string) => ({ label: role_name, value: role_name }));
    } catch (error) { if (import.meta.env.DEV) console.error('加载角色列表失败', error); }
  }

  async function loadRolePermissions(role_name: string) {
    if (!role_name) return;
    try {
      const response = await getRolePermissions(role_name);
      rolePermissions.value = (response.data as string[] | { permissions?: string[] }) 
    ? Array.isArray(response.data) ? (response.data as string[]) : ((response.data as { permissions?: string[] }).permissions || [])
    : [];
      initPermissionGroups(rolePermissions.value);
    } catch (error) {
      if (import.meta.env.DEV) console.error('加载角色权限失败', error);
      initPermissionGroups([]);
    }
  }

  function initPermissionGroups(selectedPermissions: string[]) {
    permissionGroups.value = getAllModules().map((module) => ({
      module: module.value,
      label: module.label,
      selected: false,
      permissions: PERMISSIONS.filter((p) => String(p.module) === String(module.value)).map((p) => ({
        key: p.key, name: p.name, ...(p.description ? { description: p.description } : {}),
        selected: selectedPermissions.includes(p.key),
      })),
    }));
  }

  function getCurrentPermissions(): string[] {
    const perms: string[] = [];
    permissionGroups.value.forEach((g) => g.permissions.forEach((p) => { if (p.selected) perms.push(p.key); }));
    return perms;
  }

  function getSelectedCount(group: { permissions: { selected: boolean }[] }): number {
    return group.permissions.filter((p) => p.selected).length;
  }

  function toggleGroup(group: { permissions: { selected: boolean }[] }, selected: boolean) {
    group.permissions.forEach((p) => { p.selected = selected; });
  }

  function resetChanges() { initPermissionGroups(rolePermissions.value); }

  async function savePermissions() {
    if (!selectedRole.value) return;
    saving.value = true;
    try {
      const perms = getCurrentPermissions();
      await updateRolePermissions(selectedRole.value, perms);
      rolePermissions.value = perms;
      initPermissionGroups(perms);
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch {
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally { saving.value = false; }
  }

  async function loadPermissions() {
    loadingPermissions.value = true;
    try {
      const response = await listPermissions() as { data?: { list?: unknown[]; data?: unknown[] }; list?: unknown[] };
      const dataObj = response.data as { list?: unknown[]; data?: unknown[] } | undefined;
      // 空数组也是 truthy，不能直接 `||` 兜底——权限定义是前端配置型数据，
      // API 未实现/返回空时回退到内置 PERMISSIONS 常量
      const fetched = response.list?.length ? response.list : (dataObj?.list?.length ? dataObj.list : (dataObj?.data?.length ? dataObj.data : null));
      permissionDefinitions.value = (fetched || PERMISSIONS) as typeof PERMISSIONS;
      permissionPagination.rowsNumber = permissionDefinitions.value.length;
    } catch {
      permissionDefinitions.value = PERMISSIONS;
      permissionPagination.rowsNumber = PERMISSIONS.length;
    } finally { loadingPermissions.value = false; }
  }

  function onPermissionRequest(props: Parameters<NonNullable<QTableProps['onRequest']>>[0]) {
    permissionPagination.page = props.pagination.page;
    permissionPagination.rowsPerPage = props.pagination.rowsPerPage;
  }

  function getModuleLabel(module: string): string { return moduleOptions.find((m) => String(m.value) === module)?.label || module; }
  function getCategoryColor(category: string): string { const c: Record<string, string> = { page: 'blue', button: 'green', field: 'orange', data: 'purple' }; return c[category] || 'grey'; }
  function getCategoryLabel(category: string): string { return categoryOptions.find((c) => c.value === category)?.label || category; }

  function openPermissionDialog(permission?: PermissionDefinition) {
    if (permission) {
      editingPermission.value = permission;
      permissionForm.name = permission.name;
      permissionForm.module = permission.module ?? PermissionModule.USER;
      permissionForm.category = permission.category ?? 'button';
      permissionForm.description = permission.description || '';
      permissionForm.sensitive = permission.sensitive || false;
    } else {
      editingPermission.value = null;
      permissionForm.name = '';
      permissionForm.module = PermissionModule.USER;
      permissionForm.category = 'button';
      permissionForm.description = '';
      permissionForm.sensitive = false;
    }
    showPermissionDialog.value = true;
  }

  async function handleSavePermission() {
    if (!permissionForm.name) { $q.notify({ type: 'negative', message: '请输入权限名称' }); return; }
    savingPermission.value = true;
    try {
      if (editingPermission.value) {
        await updatePermission(editingPermission.value.key, { name: permissionForm.name, category: permissionForm.category, description: permissionForm.description, sensitive: permissionForm.sensitive });
      } else {
        await createPermission({ name: permissionForm.name, module: permissionForm.module, category: permissionForm.category, description: permissionForm.description, sensitive: permissionForm.sensitive });
      }
      $q.notify({ type: 'positive', message: '保存成功' });
      showPermissionDialog.value = false;
      void loadPermissions();
    } catch {
      $q.notify({ type: 'negative', message: '保存失败' });
    } finally { savingPermission.value = false; }
  }

  function confirmDeletePermission(permission: PermissionDefinition) { deletingPermission.value = permission; showDeleteDialog.value = true; }

  async function doDeletePermission() {
    if (!deletingPermission.value) return;
    try {
      await deletePermission(deletingPermission.value.key);
      $q.notify({ type: 'positive', message: '删除成功' });
      showDeleteDialog.value = false;
      deletingPermission.value = null;
      void loadPermissions();
    } catch { $q.notify({ type: 'negative', message: '删除失败' }); }
  }

  onMounted(async () => { await Promise.all([loadRoles(), loadPermissions()]); initPermissionGroups([]); });

  return {
    roles, selectedRole, saving, rolePermissions, permissionGroups, permissionDefinitions, loadingPermissions,
    permissionPagination, showPermissionDialog, editingPermission, permissionForm, savingPermission,
    showDeleteDialog, deletingPermission, roleOptions, hasChanges, is_read_only, permissionColumns,
    moduleOptions, categoryOptions,
    loadRoles, loadRolePermissions, initPermissionGroups, getCurrentPermissions, getSelectedCount,
    toggleGroup, resetChanges, savePermissions, loadPermissions, onPermissionRequest,
    getModuleLabel, getCategoryColor, getCategoryLabel, openPermissionDialog, handleSavePermission,
    confirmDeletePermission, doDeletePermission,
  };
}
