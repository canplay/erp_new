/**
 * @file permission.ts
 * @description 权限状态管理 - 简化版
 * @date 2026-08-15
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { httpClient } from '@/utils/alova';
import { logger } from "@/utils/logger";
import { useAuthStore } from "./auth";
import { PermissionScope, type PermissionChangeLog, type PermissionOption, getSensitivePermissions } from "@/types/permission";
import { getRolePermissionConfig, getRoleDataPermissions, updateRoleDataPermissions, getRoleFieldPermissions, updateRoleFieldPermissions, getRoleInheritChain, setRoleInherit, removeRoleInherit } from "@/api/permission";

interface DataPermissionConfig { permission: string; scope?: PermissionScope; custom_scope?: number[]; }
interface FieldPermissionConfig { field: string; entity_type: string; canView: boolean; canEdit: boolean; }

export interface MenuItem { id: number; name: string; path: string; icon?: string; children?: MenuItem[]; permissions?: string[]; }
export interface InheritPermissionInfo { role: string; inherit_from: string[]; effective_permissions: string[]; }
export interface PermissionInfo { user_id: number; role: string; permissions: string[]; menus: MenuItem[]; }

export const usePermissionStore = defineStore("permission", () => {
  const menus = ref<MenuItem[]>([]);
  const permissions = ref<string[]>([]);
  const role = ref<string>("");
  const isLoaded = ref(false);
  const dataPermissions = ref<DataPermissionConfig[]>([]);
  const fieldPermissions = ref<FieldPermissionConfig[]>([]);
  const inheritChain = ref<InheritPermissionInfo | null>(null);
  
  // 简化：直接使用 permissions.value，移除不必要的 effectivePermissions 计算
  const effectivePermissions = computed(() => permissions.value);
  
  const cacheExpiry = ref<number>(0);
  const CACHE_DURATION = 5 * 60 * 1000;
  const permissionChangeLogs = ref<PermissionChangeLog[]>([]);
  const permissionLogsTotal = ref(0);

  function hasPermission(permission: string, userRole?: string): boolean {
    const currentRole = userRole || role.value;
    if (currentRole === "admin") return true;
    return permissions.value.includes(permission);
  }

  function hasRole(targetRole: string): boolean { return role.value === targetRole; }

  function hasDataPermission(permission: string, resourceOwner?: number, resourceDepartment?: number): boolean {
    if (role.value === "admin") return true;
    const dataPerm = dataPermissions.value.find((dp) => dp.permission === permission);
    if (!dataPerm) return false; // 简化：未配置时默认拒绝
    
    const scope = dataPerm.scope;
    switch (scope) {
      case PermissionScope.ALL: return true;
      case PermissionScope.SELF: if (resourceOwner) return resourceOwner === getCurrentUserId(); return false;
      case PermissionScope.DEPARTMENT:
      case PermissionScope.DEPARTMENT_AND_CHILDREN: {
        const currentDepartmentId = (useAuthStore().userInfo as unknown as { department_id?: number })?.department_id;
        if (!currentDepartmentId || !resourceDepartment) return false;
        return resourceDepartment === currentDepartmentId;
      }
      case PermissionScope.CUSTOM:
        if (dataPerm.custom_scope && dataPerm.custom_scope.length > 0) {
          if (resourceDepartment) return dataPerm.custom_scope.includes(resourceDepartment);
          return false;
        }
        return false;
      default: return true;
    }
  }

  function hasFieldPermission(entity_type: string, field: string, action: "view" | "edit"): boolean {
    if (role.value === "admin") return true;
    const fieldPerm = fieldPermissions.value.find((fp) => fp.entity_type === entity_type && fp.field === field);
    if (!fieldPerm) return false; // 简化：未配置时默认拒绝
    return action === "view" ? fieldPerm.canView : fieldPerm.canEdit;
  }

  const getMenus = computed(() => menus.value);
  const isCacheValid = computed(() => isLoaded.value && Date.now() < cacheExpiry.value);

  function getCurrentUserId(): number {
    const authStore = useAuthStore();
    return authStore.userInfo?.id || 0;
  }

  async function fetchPermissions() {
    if (isCacheValid.value) return;
    try {
      const authStore = useAuthStore();
      if (!authStore.userInfo?.role) { isLoaded.value = true; cacheExpiry.value = Date.now() + CACHE_DURATION; return; }
      const response = await httpClient.get(`/admin/roles/${encodeURIComponent(authStore.userInfo.role)}/permissions`);
      const data = (response as { data?: string[] }).data;
      if (data) {
        role.value = authStore.userInfo?.role || "";
        permissions.value = data;
        menus.value = [];
        isLoaded.value = true;
        cacheExpiry.value = Date.now() + CACHE_DURATION;
      }
    } catch (error) {
      logger.error("【获取权限信息失败】", error);
      throw error;
    }
  }

  async function fetchRoleMenus(role_name: string) {
    try {
      const response = await httpClient.get("/admin/roles/" + role_name + "/menus");
      return (response as { data?: MenuItem[] }).data;
    } catch (error) {
      logger.error("【获取角色菜单失败】", error);
      throw error;
    }
  }

  async function fetchRolePermissionConfig(role_name: string) {
    try {
      const response = await getRolePermissionConfig(role_name);
      return (response as { data?: unknown }).data;
    } catch (error) {
      logger.error("【获取角色权限配置失败】", error);
      throw error;
    }
  }

  async function updateUserRole(user_id: number, userRole: string) {
    try {
      await httpClient.put("/admin/users/" + user_id + "/role", { role: userRole });
      return true;
    } catch (error) {
      logger.error("【更新用户角色失败】", error);
      throw error;
    }
  }

  function clearPermissions() {
    menus.value = []; permissions.value = []; role.value = ""; isLoaded.value = false;
    dataPermissions.value = []; fieldPermissions.value = []; inheritChain.value = null;
    permissionChangeLogs.value = []; cacheExpiry.value = 0;
  }

  function filterAccessibleMenus(allMenus: MenuItem[]): MenuItem[] {
    if (role.value === "admin") return allMenus;
    return allMenus.filter((menu) => {
      if (menu.permissions && menu.permissions.length > 0) {
        return menu.permissions.some((p) => permissions.value.includes(p));
      }
      return true;
    });
  }

  async function loadDataPermissions(role_name: string) {
    try { const response = await getRoleDataPermissions(role_name); dataPermissions.value = (response as { data?: DataPermissionConfig[] }).data || []; }
    catch (error) { logger.error("【加载数据权限失败】", error); dataPermissions.value = []; }
  }

  async function saveDataPermissions(role_name: string) {
    try { await updateRoleDataPermissions(role_name, dataPermissions.value); return true; }
    catch (error) { logger.error("【保存数据权限失败】", error); throw error; }
  }

  function setDataPermissionScope(permission: string, scope: PermissionScope, custom_scope?: number[]) {
    const existing = dataPermissions.value.find((dp) => dp.permission === permission);
    if (existing) {
      existing.scope = scope;
      if (custom_scope !== undefined) existing.custom_scope = custom_scope;
    } else {
      dataPermissions.value.push({ permission, scope, ...(custom_scope !== undefined ? { custom_scope } : {}) });
    }
  }

  function removeDataPermission(permission: string) {
    const index = dataPermissions.value.findIndex((dp) => dp.permission === permission);
    if (index > -1) dataPermissions.value.splice(index, 1);
  }

  async function loadFieldPermissions(role_name: string) {
    try { const response = await getRoleFieldPermissions(role_name); fieldPermissions.value = (response as { data?: FieldPermissionConfig[] }).data || []; }
    catch (error) { logger.error("【加载字段权限失败】", error); fieldPermissions.value = []; }
  }

  async function saveFieldPermissions(role_name: string) {
    try { await updateRoleFieldPermissions(role_name, fieldPermissions.value); return true; }
    catch (error) { logger.error("【保存字段权限失败】", error); throw error; }
  }

  function setFieldPermission(entity_type: string, field: string, canView: boolean, canEdit: boolean) {
    const existing = fieldPermissions.value.find((fp) => fp.entity_type === entity_type && fp.field === field);
    if (existing) { existing.canView = canView; existing.canEdit = canEdit; }
    else { fieldPermissions.value.push({ field, entity_type, canView, canEdit }); }
  }

  async function loadInheritChain(role_name: string) {
    try { const response = await getRoleInheritChain(role_name); inheritChain.value = (response as { data?: InheritPermissionInfo }).data ?? null; }
    catch (error) { logger.error("【加载继承链失败】", error); inheritChain.value = null; }
  }

  async function setInherit(role_name: string, inherit_from: string[]) {
    try { await setRoleInherit(role_name, inherit_from); await loadInheritChain(role_name); return true; }
    catch (error) { logger.error("【设置继承失败】", error); throw error; }
  }

  async function removeInherit(role_name: string) {
    try { await removeRoleInherit(role_name); inheritChain.value = null; return true; }
    catch (error) { logger.error("【移除继承失败】", error); throw error; }
  }

  async function loadPermissionChangeLogs(params?: { page?: number; page_size?: number; role_name?: string; operator?: string; change_type?: "add" | "remove" | "update"; start_date?: string; end_date?: string; }) {
    try { const response = await httpClient.get("/permission-change-logs", { params }); const permData = (response as { data?: { list: PermissionChangeLog[]; total: number } }).data; permissionChangeLogs.value = permData?.list || []; permissionLogsTotal.value = permData?.total || 0; }
    catch (error) { logger.error("【加载权限变更日志失败】", error); permissionChangeLogs.value = []; permissionLogsTotal.value = 0; }
  }

  const permissionHistory = ref<Array<{ role: string; before: string[]; after: string[]; timestamp: number }>>([]);

  function recordPermissionChange(role_name: string, before: string[], after: string[]) {
    permissionHistory.value.push({ role: role_name, before, after, timestamp: Date.now() });
    if (permissionHistory.value.length > 50) permissionHistory.value.shift();
  }

  function getPermissionHistory(role_name?: string) {
    if (role_name) return permissionHistory.value.filter((h) => h.role === role_name);
    return permissionHistory.value;
  }

  function getAllSensitivePermissions(): PermissionOption[] { return getSensitivePermissions(); }
  function isSensitivePermission(permission: string): boolean { return getSensitivePermissions().some(p => p.key === permission); }

  return {
    menus, permissions, role, isLoaded, dataPermissions, fieldPermissions, inheritChain, effectivePermissions, permissionChangeLogs, permissionLogsTotal,
    getMenus, isCacheValid, hasPermission, hasRole, hasDataPermission, hasFieldPermission,
    fetchPermissions, fetchRoleMenus, fetchRolePermissionConfig, updateUserRole, clearPermissions, filterAccessibleMenus,
    loadDataPermissions, saveDataPermissions, setDataPermissionScope, removeDataPermission,
    loadFieldPermissions, saveFieldPermissions, setFieldPermission,
    loadInheritChain, setInherit, removeInherit,
    loadPermissionChangeLogs, recordPermissionChange, getPermissionHistory,
    getAllSensitivePermissions, isSensitivePermission,
  };
});
