/**
 * @file usePermissionTree.ts
 * @description Composable for permission tree logic
 * @date 2026-09-12
 */

import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  PERMISSIONS,
  getAllModules,
  getSensitivePermissions,
  PermissionModule,
  type PermissionDefinition,
} from '@/types/permission';

export interface PermissionGroup {
  module: PermissionModule;
  label: string;
  icon?: string;
  expanded: boolean;
  permissions: PermissionDefinition[];
}

export interface PermissionTreeOptions {
  modelValue?: string[];
  disabled?: boolean;
  defaultExpanded?: boolean;
  excludeModules?: PermissionModule[];
  includeModules?: PermissionModule[];
}

export function usePermissionTree(options: PermissionTreeOptions = {}) {
  const { t: $t } = useI18n();

  const selectedPermissions = ref<string[]>(options.modelValue || []);
  const permissionGroups = ref<PermissionGroup[]>([]);
  const searchQuery = ref('');

  // Props with defaults
  const disabled = options.disabled ?? false;
  const defaultExpanded = options.defaultExpanded ?? false;
  const excludeModules = options.excludeModules ?? [];
  const includeModules = options.includeModules;

  watch(() => options.modelValue, (newVal) => {
    selectedPermissions.value = newVal || [];
  }, { immediate: true });

  const filteredPermissionGroups = computed(() => {
    if (!searchQuery.value.trim()) {
      return permissionGroups.value;
    }

    const query = searchQuery.value.toLowerCase().trim();

    return permissionGroups.value.map((group) => {
      const filteredPermissions = group.permissions.filter((p) => {
        return (
          p.name.toLowerCase().includes(query) ||
          p.key.toLowerCase().includes(query) ||
          (p.description && p.description.toLowerCase().includes(query))
        );
      });

      return {
        ...group,
        permissions: filteredPermissions,
        expanded: filteredPermissions.length > 0,
      };
    }).filter((group) => group.permissions.length > 0);
  });

  const selectAll = computed({
    get: () => {
      if (totalCount.value === 0) return false;
      return selectedPermissions.value.length === totalCount.value;
    },
    set: () => {},
  });

  const indeterminate = computed(() => {
    const count = selectedPermissions.value.length;
    return count > 0 && count < totalCount.value;
  });

  const selectedCount = computed(() => selectedPermissions.value.length);

  const totalCount = computed(() => {
    return permissionGroups.value.reduce((sum, group) => sum + group.permissions.length, 0);
  });

  const sensitiveSelectedCount = computed(() => {
    const sensitiveKeys = getSensitivePermissions().map((p) => p.key);
    return selectedPermissions.value.filter((key) => sensitiveKeys.includes(key)).length;
  });

  function getModuleIcon(module: PermissionModule): string {
    const icons: Record<PermissionModule, string> = {
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
    return icons[module] || 'security';
  }

  function initPermissionGroups() {
    const modules = getAllModules();
    const excludeSet = new Set(excludeModules);
    const includeSet = includeModules ? new Set(includeModules) : null;

    permissionGroups.value = modules
      .filter((m) => {
        if (excludeSet.has(m.value)) return false;
        if (includeSet && !includeSet.has(m.value)) return false;
        const moduleValue = String(m.value);
        const modulePerms = PERMISSIONS.filter((p) => String(p.module) === moduleValue);
        return modulePerms.length > 0;
      })
      .map((m) => {
        const moduleValue = String(m.value);
        const modulePerms = PERMISSIONS.filter((p) => String(p.module) === moduleValue);
        return {
          module: m.value,
          label: m.label,
          icon: getModuleIcon(m.value),
          expanded: defaultExpanded,
          permissions: modulePerms,
        };
      });
  }

  function getGroupSelectedCount(group: PermissionGroup): number {
    return group.permissions.filter((p) => selectedPermissions.value.includes(p.key)).length;
  }

  function getGroupMatchedCount(group: PermissionGroup): number {
    if (!searchQuery.value.trim()) return group.permissions.length;
    return group.permissions.length;
  }

  function isGroupAllSelected(group: PermissionGroup): boolean {
    return group.permissions.every((p) => selectedPermissions.value.includes(p.key));
  }

  function isGroupIndeterminate(group: PermissionGroup): boolean {
    const selected = getGroupSelectedCount(group);
    return selected > 0 && selected < group.permissions.length;
  }

  function toggleGroup(group: PermissionGroup, selected: boolean) {
    if (selected) {
      group.permissions.forEach((p) => {
        if (!selectedPermissions.value.includes(p.key)) {
          selectedPermissions.value.push(p.key);
        }
      });
    } else {
      selectedPermissions.value = selectedPermissions.value.filter(
        (key) => !group.permissions.some((p) => p.key === key)
      );
    }
  }

  function toggleSelectAll(selected: boolean) {
    if (selected) {
      selectedPermissions.value = permissionGroups.value.flatMap((g) =>
        g.permissions.map((p) => p.key)
      );
    } else {
      selectedPermissions.value = [];
    }
  }

  function emitUpdate(callback?: () => void) {
    callback?.();
  }

  // Exposed methods
  function selectAllFn() {
    selectedPermissions.value = permissionGroups.value.flatMap((g) =>
      g.permissions.map((p) => p.key)
    );
  }

  function clearAll() {
    selectedPermissions.value = [];
  }

  function getSelected() {
    return [...selectedPermissions.value];
  }

  function expandAll() {
    permissionGroups.value.forEach((g) => (g.expanded = true));
  }

  function collapseAll() {
    permissionGroups.value.forEach((g) => (g.expanded = false));
  }

  function search(query: string) {
    searchQuery.value = query;
  }

  function clearSearch() {
    searchQuery.value = '';
  }

  initPermissionGroups();

  return {
    selectedPermissions,
    permissionGroups,
    searchQuery,
    filteredPermissionGroups,
    selectAll,
    indeterminate,
    selectedCount,
    totalCount,
    sensitiveSelectedCount,
    disabled,
    initPermissionGroups,
    getGroupSelectedCount,
    getGroupMatchedCount,
    isGroupAllSelected,
    isGroupIndeterminate,
    toggleGroup,
    toggleSelectAll,
    emitUpdate,
    selectAllFn,
    clearAll,
    getSelected,
    expandAll,
    collapseAll,
    search,
    clearSearch,
  };
}
