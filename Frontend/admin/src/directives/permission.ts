/**
 * @file permission.ts
 * @description 权限指令 - v-permission (增强版)
 * @date 2026-04-04
 * @features
 * - 支持 any/all 模式
 * - 支持敏感权限二次确认
 * - 支持权限有效期检查
 */

import type { Directive, DirectiveBinding } from 'vue';
import { usePermissionStore } from '@/stores/permission';
import { getSensitivePermissions } from '@/types/permission';
import { checkSensitivePermission } from '@/api/permission';
import { useQuasar } from 'quasar';

/**
 * @brief 权限指令修饰符
 */
interface PermissionModifiers {
  any?: boolean;
  all?: boolean;
  role?: boolean;
  confirm?: boolean; // 敏感权限确认
}

/**
 * @brief 权限绑定值类型
 */
interface PermissionBindingValue {
  permission?: string | string[];
  mode?: 'any' | 'all';
  confirm?: boolean; // 是否需要确认
}

/**
 * @brief 已确认的敏感权限缓存（会话级别）
 */
const confirmedPermissions = new Set<string>();

/**
 * @brief 获取绑定的权限值
 */
function getBindingValue(binding: DirectiveBinding): { permissions: string[]; mode: 'any' | 'all'; needConfirm: boolean } {
  const value = binding.value;
  const modifiers = binding.modifiers as PermissionModifiers;

  // 确定模式
  let mode: 'any' | 'all' = 'any';
  if (modifiers.all) {
    mode = 'all';
  } else if (modifiers.any) {
    mode = 'any';
  } else if (value && typeof value === 'object' && 'mode' in value) {
    mode = value.mode || 'any';
  }

  // 解析权限值
  let permissions: string[] = [];
  let needConfirm = modifiers.confirm || false;

  if (typeof value === 'string') {
    permissions = [value];
  } else if (Array.isArray(value)) {
    permissions = value;
  } else if (value && typeof value === 'object') {
    const obj = value as PermissionBindingValue;
    if (obj.permission) {
      permissions = Array.isArray(obj.permission) ? obj.permission : [obj.permission];
    }
    if (obj.mode) {
      mode = obj.mode;
    }
    if (obj.confirm) {
      needConfirm = true;
    }
  }

  return { permissions, mode, needConfirm };
}

/**
 * @brief 检查是否为敏感权限
 */
function isSensitivePermission(permission: string): boolean {
  const sensitiveKeys = getSensitivePermissions().map((p) => p.key);
  return sensitiveKeys.includes(permission);
}

/**
 * @brief 检查是否已确认过该权限
 */
function isConfirmed(permission: string): boolean {
  return confirmedPermissions.has(permission);
}

/**
 * @brief 标记权限为已确认
 */
function markConfirmed(permission: string, duration: number = 5 * 60 * 1000) {
  confirmedPermissions.add(permission);
  // 指定时间后清除确认状态
  setTimeout(() => {
    void confirmedPermissions.delete(permission);
  }, duration);
}

/**
 * @brief 保存原始 display 样式的元素
 */
const elementDisplayMap = new WeakMap<HTMLElement, string>();

/**
 * @brief 隐藏元素
 */
function hideElement(el: HTMLElement) {
  if (!elementDisplayMap.has(el)) {
    elementDisplayMap.set(el, el.style.display);
  }
  el.style.display = 'none';
}

/**
 * @brief 恢复元素
 */
function restoreElement(el: HTMLElement) {
  const originalDisplay = elementDisplayMap.get(el);
  if (originalDisplay !== undefined) {
    el.style.display = originalDisplay;
    elementDisplayMap.delete(el);
  }
}

/**
 * @brief 执行敏感权限确认
 */
async function confirmSensitivePermission(permission: string): Promise<boolean> {
  // 如果已经确认过，直接返回 true
  if (isConfirmed(permission)) {
    return true;
  }

  const $q = useQuasar();

  return new Promise((resolve) => {
    $q.dialog({
      title: '敏感权限确认',
      message: `您正在执行敏感操作："${permission}"。此操作需要额外确认。是否继续？`,
      prompt: {
        model: '',
        type: 'text',
        placeholder: '请输入 "确认" 以继续',
        isValid: (val: string) => val === '确认',
      },
      cancel: {
        flat: true,
        color: 'grey',
        label: '取消',
      },
      ok: {
        color: 'negative',
        label: '确认执行',
      },
      persistent: true,
    }).onOk(() => {
      markConfirmed(permission);
      resolve(true);
    }).onCancel(() => {
      resolve(false);
    }).onDismiss(() => {
      resolve(false);
    });
  });
}

/**
 * @brief 权限检查核心逻辑
 */
async function checkPermission(
  el: HTMLElement,
  binding: DirectiveBinding,
  permissionStore: ReturnType<typeof usePermissionStore>
): Promise<boolean> {
  const { permissions, mode, needConfirm } = getBindingValue(binding);

  if (permissions.length === 0) {
    restoreElement(el);
    return true;
  }

  // 检查权限
  let hasPermission: boolean;
  if (mode === 'all') {
    hasPermission = permissions.every((p) => permissionStore.hasPermission(p));
  } else {
    hasPermission = permissions.some((p) => permissionStore.hasPermission(p));
  }

  if (!hasPermission) {
    hideElement(el);
    return false;
  }

  // 如果没有配置需要确认，恢复元素
  if (!needConfirm) {
    restoreElement(el);
    return true;
  }

  // 检查是否包含敏感权限且未确认
  const sensitiveUnconfirmed = permissions.filter(
    (p) => isSensitivePermission(p) && !isConfirmed(p)
  );

  if (sensitiveUnconfirmed.length === 0) {
    restoreElement(el);
    return true;
  }

  // 需要确认
  const confirmed = await confirmSensitivePermission(sensitiveUnconfirmed[0]!);
  if (confirmed) {
    restoreElement(el);
    return true;
  }
  hideElement(el);
  return false;
}

/**
 * @brief 权限指令
 */
const Permission: Directive = {
  mounted(el: HTMLElement, binding: DirectiveBinding) {
    const permissionStore = usePermissionStore();
    void checkPermission(el, binding, permissionStore);
  },

  updated(el: HTMLElement, binding: DirectiveBinding) {
    const permissionStore = usePermissionStore();
    void checkPermission(el, binding, permissionStore);
  },
};

/**
 * @brief 清除所有权限确认状态
 */
function clearPermissionConfirmations() {
  confirmedPermissions.clear();
}

/**
 * @brief 权限指令组合式函数版本
 */
export function usePermission() {
  const permissionStore = usePermissionStore();

  /**
   * @brief 检查是否有权限
   */
  function check(permission: string | string[], mode: 'any' | 'all' = 'any'): boolean {
    const permissions = Array.isArray(permission) ? permission : [permission];
    if (mode === 'all') {
      return permissions.every((p) => permissionStore.hasPermission(p));
    }
    return permissions.some((p) => permissionStore.hasPermission(p));
  }

  /**
   * @brief 检查并确认敏感权限
   */
  async function checkWithConfirmation(permission: string): Promise<boolean> {
    if (!permissionStore.hasPermission(permission)) {
      return false;
    }

    if (isSensitivePermission(permission) && !isConfirmed(permission)) {
      const confirmed = await confirmSensitivePermission(permission);
      return confirmed;
    }

    return true;
  }

  /**
   * @brief 获取用户有效权限
   */
  function getEffectivePermissions(): string[] {
    return permissionStore.effectivePermissions;
  }

  /**
   * @brief 检查数据权限
   */
  function checkDataPermission(
    permission: string,
    resourceOwner?: number,
    resourceDepartment?: number
  ): boolean {
    return permissionStore.hasDataPermission(permission, resourceOwner, resourceDepartment);
  }

  /**
   * @brief 检查字段权限
   */
  function checkFieldPermission(entity_type: string, field: string, action: 'view' | 'edit'): boolean {
    return permissionStore.hasFieldPermission(entity_type, field, action);
  }

  return {
    check,
    checkWithConfirmation,
    getEffectivePermissions,
    checkDataPermission,
    checkFieldPermission,
    clearConfirmations: clearPermissionConfirmations,
  };
}

export { checkSensitivePermission, isSensitivePermission, isConfirmed, markConfirmed };
export default Permission;
