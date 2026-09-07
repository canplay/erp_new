/**
 * @file usePermissionMatrix.ts
 * @description 权限矩阵可视化组件业务逻辑 composable
 */

import { ref, computed, reactive } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import type { QTableProps } from 'quasar';
import type { PermissionMatrixPermission } from '@/types/permissionMatrix';
import { PERMISSIONS, type PermissionModule } from '@/types/permission';
import { updateRolePermissionConfig } from '@/api/permission';

const MODULE_LABELS: Record<string, string> = {
  user: '用户管理', role: '角色管理', permission: '权限管理',
  login_log: '登录日志', operation_log: '操作日志', system_config: '系统配置',
  dictionary: '数据字典', notification: '通知管理', announcement: '公告管理',
  monitor: '系统监控', file: '文件管理', task: '任务调度',
  import_export: '导入导出', audit: '审计日志',
};

const DEFAULT_ROLES = ['admin', 'user', 'guest'] as const;

export function usePermissionMatrix() {
  const $q = useQuasar();

  const moduleFilter = ref<PermissionModule | null>(null);
  const categoryFilter = ref<'page' | 'button' | 'field' | 'data' | null>(null);
  const searchQuery = ref('');
  const viewMode = ref<'matrix' | 'list'>('matrix');
  const showDetailDialog = ref(false);
  const selectedPermission = ref<PermissionMatrixPermission | null>(null);

  const matrixData = reactive<Record<string, Record<string, boolean>>>({});
  const roles = ref<string[]>([...DEFAULT_ROLES]);

  const moduleOptions = computed(() => {
    const modules = new Set(PERMISSIONS.map((p) => p.module));
    return [
      { label: '全部模块', value: null },
      ...Array.from(modules).map((m) => ({ label: MODULE_LABELS[m ?? ''] ?? m ?? '', value: m })),
    ];
  });

  const categoryOptions = computed(() => [
    { label: '全部类型', value: null },
    { label: '页面', value: 'page' },
    { label: '按钮', value: 'button' },
    { label: '字段', value: 'field' },
    { label: '数据', value: 'data' },
  ]);

  const filteredPermissions = computed(() => {
    return PERMISSIONS.filter((p) => {
      if (moduleFilter.value && p.module !== moduleFilter.value) return false;
      if (categoryFilter.value && p.category !== categoryFilter.value) return false;
      if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        return p.name.toLowerCase().includes(query) || p.key.toLowerCase().includes(query);
      }
      return true;
    });
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
    { name: 'permission', label: '权限', field: 'name', align: 'left', sortable: true },
    { name: 'module', label: '模块', field: 'module', align: 'center' },
    { name: 'category', label: '类型', field: 'category', align: 'center' },
    { name: 'roles', label: '角色授权', field: 'roles', align: 'center' },
    { name: 'actions', label: '操作', field: 'actions', align: 'center' },
  ];

  function getModuleLabel(module: string): string { return MODULE_LABELS[module] || module; }
  function getRoleLabel(role: string): string {
    const labels: Record<string, string> = { admin: '管理员', user: '普通用户', guest: '访客' };
    return labels[role] || role;
  }

  function getCellValue(role: string, permissionKey: string): boolean {
    const roleData = matrixData[role];
    if (!roleData) {
      matrixData[role] = {};
      if (role === 'admin') {
        PERMISSIONS.forEach((p) => { matrixData[role]![p.key] = true; });
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
    $q.notify({ type: 'positive', message: '权限矩阵已刷新' });
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
    $q.loading.show({ message: '正在保存权限配置...' });
    try {
      const adminConfig = { function_permissions: rolePermissions['admin'] || [], data_permissions: [], field_permissions: [] };
      await updateRolePermissionConfig('admin', adminConfig);
      logger.info('【权限矩阵】保存配置成功：', rolePermissions);
      $q.notify({ type: 'positive', message: '权限配置已保存' });
    } catch (error) {
      logger.error('【权限矩阵】保存配置失败：', error);
      $q.notify({ type: 'negative', message: '保存失败，请重试' });
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
        PERMISSIONS.forEach((p) => { matrixData[role]![p.key] = true; });
      } else if (role === 'user') {
        matrixData[role] = {};
        PERMISSIONS.filter((p) => p.category === 'page').forEach((p) => { matrixData[role]![p.key] = true; });
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
