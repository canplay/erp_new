/**
 * @file useDataPermissionPanel.ts
 * @description Composable for data permission panel logic
 * @date 2026-09-12
 */

import { ref, computed, watch, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import { listDepartments } from '@/api/department';
import {
  PermissionScope,
  PermissionModule,
  type DataPermission,
  type PermissionDefinition,
  PERMISSIONS,
  getAllModules,
} from '@/types/permission';
import { updateRoleDataPermissions } from '@/api/permission';

export interface PermissionDataScope extends PermissionDefinition {
  scope: PermissionScope;
  custom_scope?: number[];
}

export interface DepartmentNode {
  id: number;
  name: string;
  children?: DepartmentNode[];
}

export interface DataPermissionPanelOptions {
  role_name: string;
  initialDataPermissions?: DataPermission[];
  disabled?: boolean;
}

export function useDataPermissionPanel(options: DataPermissionPanelOptions) {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  // ============ State ============

  const permissionDataScopes = ref<PermissionDataScope[]>([]);
  const isolationDimension = ref<'department' | 'creator' | 'both'>('department');
  const saving = ref(false);
  const showDepartmentDialog = ref(false);
  const selectedDepartmentId = ref<number | null>(null);
  const tickedDepartments = ref<number[]>([]);
  const currentEditingItem = ref<PermissionDataScope | null>(null);
  const departmentTree = ref<DepartmentNode[]>([]);
  const selectedDepartments = ref<Array<{ id: number; name: string }>>([]);

  // ============ Constants ============

  const scopeOptions = [
    { label: '全部数据', value: PermissionScope.ALL },
    { label: '本部门', value: PermissionScope.DEPARTMENT },
    { label: '本部门及下级', value: PermissionScope.DEPARTMENT_AND_CHILDREN },
    { label: '仅本人', value: PermissionScope.SELF },
    { label: '自定义', value: PermissionScope.CUSTOM },
  ];

  const isolationOptions = [
    { label: '按部门隔离', value: 'department' },
    { label: '按创建者隔离', value: 'creator' },
    { label: '两者结合', value: 'both' },
  ];

  // ============ Computed ============

  const hasCustomScope = computed(() => {
    return permissionDataScopes.value.some((item) => item.scope === PermissionScope.CUSTOM);
  });

  // ============ Methods ============

  function getModuleIcon(module: string | PermissionModule | undefined): string {
    const icons: Record<string, string> = {
      [PermissionModule.USER]: 'people',
      [PermissionModule.ROLE]: 'admin_panel_settings',
      [PermissionModule.PERMISSION]: 'vpn_key',
      [PermissionModule.LOGIN_LOG]: 'login',
      [PermissionModule.OPERATION_LOG]: 'history',
      [PermissionModule.SYSTEM_CONFIG]: 'settings',
      [PermissionModule.DICTIONARY]: 'menu_book',
      [PermissionModule.NOTIFICATION]: 'notifications',
      [PermissionModule.ANNOUNCEMENT]: 'campaign',
      [PermissionModule.MONITOR]: 'monitor_heart',
      [PermissionModule.FILE]: 'folder',
      [PermissionModule.TASK]: 'schedule',
      [PermissionModule.IMPORT_EXPORT]: 'import_export',
      [PermissionModule.ORGANIZATION]: 'corporate_fare',
      [PermissionModule.AUDIT]: 'fact_check',
    };
    const moduleStr = String(module ?? '');
    return icons[moduleStr] || 'security';
  }

  function initPermissionDataScopes() {
    const modules = getAllModules();
    const initialPermMap = new Map(
      options.initialDataPermissions?.map((dp) => [dp.permission, dp]) || []
    );

    permissionDataScopes.value = modules
      .filter((m) => {
        const modulePerms = PERMISSIONS.filter((p) => p.module === m.value);
        return modulePerms.length > 0;
      })
      .flatMap((m) => {
        const modulePerms = PERMISSIONS.filter((p) => p.module === m.value);
        return modulePerms.map((p) => {
          const initial = initialPermMap.get(p.key);
          return {
            ...p,
            scope: initial?.scope || PermissionScope.ALL,
            ...(initial?.custom_scope ? { custom_scope: initial.custom_scope } : {}),
          };
        });
      });
  }

  async function loadDepartmentTree() {
    try {
      const response = await listDepartments({ page: 1, page_size: 100 });
      if (response.data?.data) {
        const departments = response.data.data as Array<{ id: number; name: string; children?: Array<{ id: number; name: string }> }>;
        departmentTree.value = departments.map((dept) => ({
          id: dept.id,
          name: dept.name,
          children: dept.children || [],
        }));
        logger.info('【部门树加载成功】', { count: departments.length });
      }
    } catch (error) {
      logger.warn('【部门 API 未实现，使用默认数据】', error);
      departmentTree.value = [
        {
          id: 1,
          name: '总公司',
          children: [
            { id: 2, name: '技术部' },
            { id: 3, name: '运营部' },
            { id: 4, name: '市场部' },
          ],
        },
      ];
    }
  }

  function openDepartmentSelector(item: PermissionDataScope) {
    currentEditingItem.value = item;
    tickedDepartments.value = item.custom_scope || [];
    showDepartmentDialog.value = true;
  }

  function confirmDepartmentSelection() {
    if (currentEditingItem.value) {
      currentEditingItem.value.custom_scope = [...tickedDepartments.value];
      selectedDepartments.value = tickedDepartments.value.map((id) => ({
        id,
        name: `部门${id}`,
      }));
    }
    showDepartmentDialog.value = false;
  }

  function removeDepartment(deptId: number) {
    tickedDepartments.value = tickedDepartments.value.filter((id) => id !== deptId);
    if (currentEditingItem.value) {
      currentEditingItem.value.custom_scope = [...tickedDepartments.value];
    }
    selectedDepartments.value = selectedDepartments.value.filter((d) => d.id !== deptId);
  }

  function handleScopeChange(item: PermissionDataScope, newScope: PermissionScope) {
    item.scope = newScope;
    if (newScope !== PermissionScope.CUSTOM) {
      delete item.custom_scope;
    }
  }

  function getDataPermissions(): DataPermission[] {
    return permissionDataScopes.value
      .filter((item) => item.scope !== PermissionScope.ALL)
      .map((item) => ({
        permission: item.key,
        scope: item.scope,
        ...(item.custom_scope ? { custom_scope: item.custom_scope } : {}),
      }));
  }

  function resetToDefault() {
    permissionDataScopes.value.forEach((item) => {
      item.scope = PermissionScope.ALL;
      delete item.custom_scope;
    });
    tickedDepartments.value = [];
    selectedDepartments.value = [];
  }

  async function saveDataPermissions(): Promise<DataPermission[] | null> {
    saving.value = true;
    try {
      const dataPermissions = getDataPermissions();
      await updateRoleDataPermissions(options.role_name, dataPermissions);
      $q.notify({
        type: 'positive',
        message: '数据权限保存成功',
      });
      return dataPermissions;
    } catch (error) {
      logger.error('【保存数据权限失败】', error);
      $q.notify({
        type: 'negative',
        message: '保存失败，请重试',
      });
      return null;
    } finally {
      saving.value = false;
    }
  }

  // ============ Lifecycle ============

  onMounted(() => {
    void loadDepartmentTree();
  });

  watch(
    () => options.initialDataPermissions,
    () => {
      initPermissionDataScopes();
    },
    { immediate: true, deep: true }
  );

  void loadDepartmentTree();

  return {
    // State
    permissionDataScopes,
    isolationDimension,
    saving,
    showDepartmentDialog,
    selectedDepartmentId,
    tickedDepartments,
    currentEditingItem,
    departmentTree,
    selectedDepartments,
    // Constants
    scopeOptions,
    isolationOptions,
    // Computed
    hasCustomScope,
    // Methods
    getModuleIcon,
    initPermissionDataScopes,
    loadDepartmentTree,
    openDepartmentSelector,
    confirmDepartmentSelection,
    removeDepartment,
    handleScopeChange,
    getDataPermissions,
    resetToDefault,
    saveDataPermissions,
  };
}
