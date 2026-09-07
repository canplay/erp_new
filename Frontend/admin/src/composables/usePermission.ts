/**
 * @file usePermission.ts
 * @description 权限 composable
 * @date 2026-04-03
 */

import { computed } from 'vue';
import { usePermissionStore, type MenuItem } from '@/stores/permission';

/**
 * @brief 权限检查 composable
 */
export function usePermission() {
  const permissionStore = usePermissionStore();

  // ============ 状态 ============

  /** @brief 当前用户的权限列表 */
  const permissions = computed(() => permissionStore.permissions);

  /** @brief 当前用户的角色 */
  const role = computed(() => permissionStore.role);

  /** @brief 当前用户是否为管理员 */
  const isAdmin = computed(() => role.value === 'admin');

  // ============ 权限判断 ============

  /**
   * @brief 检查是否拥有指定权限
   * @param permission 权限标识
   */
  function hasPermission(permission: string): boolean {
    return permissionStore.hasPermission(permission);
  }

  /**
   * @brief 检查是否拥有指定角色
   * @param targetRole 角色标识
   */
  function hasRole(targetRole: string): boolean {
    return permissionStore.hasRole(targetRole);
  }

  /**
   * @brief 检查是否拥有所有指定权限
   * @param perms 权限标识数组
   */
  function hasAllPermissions(perms: string[]): boolean {
    return perms.every((p) => permissionStore.hasPermission(p));
  }

  /**
   * @brief 检查是否拥有任一指定权限
   * @param perms 权限标识数组
   */
  function hasAnyPermission(perms: string[]): boolean {
    return perms.some((p) => permissionStore.hasPermission(p));
  }

  /**
   * @brief 检查是否拥有所有指定角色
   * @param rolesList 角色标识数组
   */
  function hasAllRoles(rolesList: string[]): boolean {
    return rolesList.every((r) => permissionStore.hasRole(r));
  }

  /**
   * @brief 检查是否拥有任一指定角色
   * @param rolesList 角色标识数组
   */
  function hasAnyRole(rolesList: string[]): boolean {
    return rolesList.some((r) => permissionStore.hasRole(r));
  }

  // ============ 菜单过滤 ============

  /**
   * @brief 过滤可访问的菜单
   * @param menus 菜单列表
   */
  function filterAccessibleMenus<T extends MenuItem>(menus: T[]): T[] {
    return permissionStore.filterAccessibleMenus(menus) as T[];
  }

  // ============ 权限判断（指令式）==========

  /**
   * @brief 检查权限（返回 boolean）
   * @param permission 权限标识或权限数组
   * @param mode 模式：any(任一) 或 all(所有)
   */
  function check(permission: string | string[], mode: 'any' | 'all' = 'any'): boolean {
    const perms = Array.isArray(permission) ? permission : [permission];
    if (mode === 'all') {
      return hasAllPermissions(perms);
    }
    return hasAnyPermission(perms);
  }

  return {
    // 状态
    permissions,
    role,
    isAdmin,

    // 权限判断
    hasPermission,
    hasRole,
    hasAllPermissions,
    hasAnyPermission,
    hasAllRoles,
    hasAnyRole,
    check,

    // 菜单过滤
    filterAccessibleMenus,
  };
}
