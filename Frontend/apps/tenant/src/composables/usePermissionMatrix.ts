/**
 * @file usePermissionMatrix.ts
 * @description 权限矩阵可视化组件业务逻辑 composable
 */

import { ref, computed, reactive } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import type { QTableProps } from 'quasar';
import type { PermissionMatrixPermission } from '@/types/permissionMatrix';
import { PERMISSION_META, getAllModulesI18n, type PermissionModule } from '@/types/permission';
import { updateRolePermissionConfig } from '@/api/permission';
import { useI18nT } from '@erp-new-frontend-monorepo/composables/src/useI18nT';;

const DEFAULT_ROLES = ['admin', 'user', 'guest'] as const;

export function usePermissionMatrix() {
  const $q = useQuasar();
  const { i18nT } = useI18nT();

  const moduleFilter = ref<PermissionModule | null>(null);
  const categoryFilter = ref<'page' | 'button' | 'field' | 'data' | null>(null);
  const searchQuery = ref('');
  const viewMode = ref<'matrix' | 'list'>('matrix');
  const showDetailDialog = ref(false);
  const selectedPermission = ref<PermissionMatrixPermission | null>(null);

  const matrixData = reactive<Record<string, Record<string, boolean>>>({});
  const roles = ref<string[]>([...DEFAULT_ROLES]);

  const MODULE_LABELS = computed<Record<string, string>>(() => ({
    user: i18nT('common.moduleUser', 'User Management'),
    role: i18nT('common.moduleRole', 'Role Management'),
    permission: i18nT('common.modulePermission', 'Permission Management'),
    login_log: i18nT('common.moduleLoginLog', 'Login Logs'),
    operation_log: i18nT('common.moduleOperationLog', 'Operation Logs'),
    system_config: i18nT('common.moduleSystemConfig', 'System Config'),
    dictionary: i18nT('common.moduleDictionary', 'Dictionary Mgmt'),
    notification: i18nT('common.moduleNotification', 'Notification Mgmt'),
    announcement: i18nT('common.moduleAnnouncement', 'Announcement Mgmt'),
    monitor: i18nT('common.moduleMonitor', 'Monitoring'),
    file: i18nT('common.moduleFile', 'File Management'),
    task: i18nT('common.moduleTask', 'Task Management'),
    import_export: i18nT('common.moduleImportExport', 'Import/Export'),
    audit: i18nT('common.moduleAudit', 'Audit Management'),
  }));

  const moduleOptions = computed(() => {
    const modules = new Set(PERMISSION_META.map((p) => p.module));
    return [
      { label: i18nT('common.all', 'All'), value: null },
      ...Array.from(modules).map((m) => ({ label: MODULE_LABELS.value[m ?? ''] ?? m ?? '', value: m })),
    ];
  });

  const categoryOptions = computed(() => [
    { label: i18nT('common.all', 'All'), value: null },
    { label: i18nT('common.page', 'Page') || 'Page', value: 'page' },
    { label: 'Button', value: 'button' },
    { label: 'Field', value: 'field' },
    { label: 'Data', value: 'data' },
  ]);

  const filteredPermissions = computed(() => {
    return PERMISSION_META.filter((p) => {
      if (moduleFilter.value && p.module !== moduleFilter.value) return false;
      if (categoryFilter.value && p.category !== categoryFilter.value) return false;
      if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        return p.key.toLowerCase().includes(query);
      }
      return true;
    }).map((p) => ({
      key: p.key,
      name: i18nT(`common.perm.${p.key.replace(/:/g, '')}`, p.key),
      type: p.type,
      category: p.category,
      module: p.module,
      description: i18nT(`common.perm.${p.key.replace(/:/g, '')}Desc`, ''),
      sensitive: p.sensitive ?? false,
    }));
  });

  const filteredPermissionGroups = computed(() => {
    const groups: Record<string, PermissionMatrixPermission[]> = {};
    filteredPermissions.value.forEach((p) => {
      const moduleKey = String(p.module ?? '');
      if (!groups[moduleKey]) groups[moduleKey] = [];
      const matrixPerm: PermissionMatrixPermission = {
        key: p.key, name: p.name, module: moduleKey,
        ...(p.category !== undefined ? { category: p.category } : {}),
        ...(p.description !== undefined ? { description: p.description } : {}),
      };
      groups[moduleKey].push(matrixPerm);
    });
    return groups;
  });

  const allPermissionsGranted = computed(() => {
    const result: Record<string, boolean> = {};
    roles.value.forEach((role) => {
      result[role] = filteredPermissions.value.every((p) => getCellValue(role, p.key));
    });
    return result;
  });

  const somePermissionsGranted = computed(() => {
    const result: Record<string, boolean> = {};
    roles.value.forEach((role) => {
      const granted = filteredPermissions.value.filter((p) => getCellValue(role, p.key)).length;
      result[role] = granted > 0 && granted < filteredPermissions.value.length;
    });
    return result;
  });

  const listColumns: QTableProps['columns'] = [
    { name: 'permission', label: 'Permission', field: 'name', align: 'left', sortable: true },
    { name: 'module', label: 'Module', field: 'module', align: 'center' },
    { name: 'category', label: 'Type', field: 'category', align: 'center' },
    { name: 'roles', label: 'Roles', field: 'roles', align: 'center' },
    { name: 'actions', label: 'Actions', field: 'actions', align: 'center' },
  ];

  function getModuleLabel(module: string): string { return MODULE_LABELS.value[module] || module; }
  function getRoleLabel(role: string): string {
    const labels: Record<string, string> = { admin: 'Admin', user: 'User', guest: 'Guest' };
    return labels[role] || role;
  }

  function getCellValue(role: string, permissionKey: string): boolean {
    const roleData = matrixData[role];
    if (!roleData) {
      matrixData[role] = {};
      if (role === 'admin') {
        PERMISSION_META.forEach((p) => { matrixData[role]![p.key] = true; });
      }
      return matrixData[role][permissionKey] ?? false;
    }
    return roleData[permissionKey] ?? false;
  }

  function handleCellChange(role: string, permissionKey: string, value: boolean) {
    if (!matrixData[role]) matrixData[role] = {};
    matrixData[role][permissionKey] = value;
  }

  function handleRoleToggle(role: string, value: boolean | null) {
    if (value === null) return;
    if (!matrixData[role]) matrixData[role] = {};
    const roleData = matrixData[role];
    filteredPermissions.value.forEach((p) => { roleData[p.key] = value; });
  }

  function handleRefresh() {
    Object.keys(matrixData).forEach((role) => { delete matrixData[role]; });
    initDefaultPermissions();
    $q.notify({ type: 'positive', message: 'Permission matrix refreshed' });
  }

  async function handleSave() {
    const rolePermissions: Record<string, string[]> = {};
    roles.value.forEach((role) => {
      const permissions: string[] = [];
      Object.entries(matrixData[role] || {}).forEach(([key, granted]) => {
        if (granted) permissions.push(key);
      });
      rolePermissions[role] = permissions;
    });
    $q.loading.show({ message: 'Saving permission config...' });
    try {
      const adminConfig = { function_permissions: rolePermissions['admin'] || [], data_permissions: [], field_permissions: [] };
      await updateRolePermissionConfig('admin', adminConfig);
      logger.info('【权限矩阵】保存配置成功：', rolePermissions);
      $q.notify({ type: 'positive', message: 'Permission config saved' });
    } catch (error) {
      logger.error('【权限矩阵】保存配置失败：', error);
      $q.notify({ type: 'negative', message: 'Save failed, please retry' });
    } finally {
      $q.loading.hide();
    }
  }

  function handleEditPermission(permission: PermissionMatrixPermission) {
    selectedPermission.value = permission;
    showDetailDialog.value = true;
  }

  function initDefaultPermissions() {
    roles.value.forEach((role) => {
      if (role === 'admin') {
        matrixData[role] = {};
        PERMISSION_META.forEach((p) => { matrixData[role]![p.key] = true; });
      } else if (role === 'user') {
        matrixData[role] = {};
        PERMISSION_META.filter((p) => p.category === 'page').forEach((p) => { matrixData[role]![p.key] = true; });
      } else {
        matrixData[role] = {};
      }
    });
  }

  initDefaultPermissions();

  return {
    moduleFilter, categoryFilter, searchQuery, viewMode,
    showDetailDialog, selectedPermission, matrixData, roles,
    moduleOptions, categoryOptions, filteredPermissions, filteredPermissionGroups,
    allPermissionsGranted, somePermissionsGranted, listColumns,
    getModuleLabel, getRoleLabel, getCellValue, handleCellChange,
    handleRoleToggle, handleRefresh, handleSave, handleEditPermission,
  };
}
