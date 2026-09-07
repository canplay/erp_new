/**
 * @file useRowPermission.ts
 * @description 行级权限 Composable
 * @date 2026-05-19
 * @description 2026-05-19 更新：实现实际 API 调用
 */

import { computed, ref } from 'vue';
import { usePermissionStore } from '@/stores/permission';
import { logger } from '@/utils/logger';
import { httpClient } from '@/utils/alova';
import type {
  RowPermission,
  RowPermissionCheckResult,
  FilterGroup,
  FilterRule,
} from '@/types/rowPermission';
import {
  FilterOperator,
  LogicalOperator,
  filterGroupToApiFormat,
} from '@/types/rowPermission';

/**
 * @brief 行级权限 Composable
 */
export function useRowPermission() {
  const permissionStore = usePermissionStore();

  /** 行级权限列表 */
  const rowPermissions = ref<RowPermission[]>([]);

  /** 加载状态 */
  const loading = ref(false);

  // ============ 计算属性 ============

  /** 获取当前用户适用的行级权限 */
  const applicableRowPermissions = computed(() => {
    return rowPermissions.value.filter((rp) => {
      // 检查权限是否匹配
      if (!permissionStore.hasPermission(rp.permission)) {
        return false;
      }
      // 检查是否启用
      if (!rp.enabled) {
        return false;
      }
      return true;
    });
  });

  // ============ 权限检查 ============

  /**
   * @brief 检查用户是否有权访问特定数据行
   * @param entity_type 实体类型
   * @param data 待检查的数据对象
   * @returns 检查结果
   */
  function checkRowPermission(entity_type: string, data: Record<string, unknown>): RowPermissionCheckResult {
    // 管理员拥有所有权限
    if (permissionStore.role === 'admin') {
      return { allowed: true };
    }

    // 获取适用的行级权限
    const applicablePerms = applicableRowPermissions.value.filter(
      (rp) => rp.entity_type === entity_type
    );

    // 如果没有配置行级权限，默认允许
    if (applicablePerms.length === 0) {
      return { allowed: true };
    }

    // 检查是否满足任一行级权限
    for (const rp of applicablePerms) {
      if (evaluateFilterGroup(rp.filterGroup, data)) {
        return {
          allowed: true,
          applied_filter: rp.filterGroup,
        };
      }
    }

    // 所有规则都不满足，拒绝访问
    return {
      allowed: false,
      reason: '您没有权限访问该数据',
    };
  }

  /**
   * @brief 过滤数据集，只返回用户有权访问的数据
   * @param entity_type 实体类型
   * @param dataList 数据列表
   * @returns 过滤后的数据列表
   */
  function filterAccessibleData<T extends Record<string, unknown>>(
    entity_type: string,
    dataList: T[]
  ): T[] {
    return dataList.filter((data) => checkRowPermission(entity_type, data).allowed);
  }

  /**
   * @brief 为数据添加权限过滤条件（用于 API 查询）
   * @param entity_type 实体类型
   * @returns API 格式的过滤条件
   */
  function getApiFilter(entity_type: string): Record<string, unknown> {
    // 管理员不添加过滤条件
    if (permissionStore.role === 'admin') {
      return {};
    }

    // 获取适用的行级权限
    const applicablePerms = applicableRowPermissions.value.filter(
      (rp) => rp.entity_type === entity_type
    );

    if (applicablePerms.length === 0) {
      return {};
    }

    // 合并所有适用的过滤条件（使用 OR）
    const filterGroups = applicablePerms.map((rp) => rp.filterGroup);

    // 已实现多个 filterGroup 的合并逻辑（使用 OR）
    if (filterGroups.length > 0) {
      // 构建复合过滤条件
      return buildMergedFilter(filterGroups);
    }

    return {};
  }

  /**
   * @brief 构建合并后的过滤条件（使用 OR 逻辑）
   * @param filterGroups 过滤组列表
   * @returns API 格式的过滤条件
   */
  function buildMergedFilter(filterGroups: FilterGroup[]): Record<string, unknown> {
    if (filterGroups.length === 1) {
      return filterGroupToApiFormat(filterGroups[0]!);
    }

    // 多个过滤组使用 OR 连接
    const mergedConditions: Record<string, unknown>[] = [];
    for (const group of filterGroups) {
      mergedConditions.push(filterGroupToApiFormat(group));
    }

    return {
      _logic: 'OR',
      conditions: mergedConditions,
    };
  }

  // ============ 过滤条件评估 ============

  /**
   * @brief 评估过滤组是否满足条件
   */
  function evaluateFilterGroup(filterGroup: FilterGroup, data: Record<string, unknown>): boolean {
    if (filterGroup.rules.length === 0) {
      return true;
    }

    const results = filterGroup.rules.map((rule) => evaluateRule(rule, data));

    // 根据逻辑运算符合并结果
    if (filterGroup.logic === LogicalOperator.AND) {
      return results.every((r) => r);
    }
    return results.some((r) => r);
  }

  /**
   * @brief 评估单条规则是否满足条件
   */
  function evaluateRule(rule: FilterRule, data: Record<string, unknown>): boolean {
    const fieldValue = data[rule.field];

    switch (rule.operator) {
      case FilterOperator.EQ:
        return fieldValue === rule.value;

      case FilterOperator.NE:
        return fieldValue !== rule.value;

      case FilterOperator.GT:
      case FilterOperator.GTE:
      case FilterOperator.LT:
      case FilterOperator.LTE:
      case FilterOperator.BETWEEN: {
        const num = Number(fieldValue);
        if (rule.operator === FilterOperator.GT) return num > Number(rule.value);
        if (rule.operator === FilterOperator.GTE) return num >= Number(rule.value);
        if (rule.operator === FilterOperator.LT) return num < Number(rule.value);
        if (rule.operator === FilterOperator.LTE) return num <= Number(rule.value);
        if (rule.operator === FilterOperator.BETWEEN) {
          return num >= Number(rule.value) && num <= Number(rule.value2);
        }
        return true;
      }

      case FilterOperator.IN: {
        const arr = rule.value as unknown[];
        return Array.isArray(arr) && arr.includes(fieldValue);
      }

      case FilterOperator.NOT_IN: {
        const arr = rule.value as unknown[];
        return !Array.isArray(arr) || !arr.includes(fieldValue);
      }

      case FilterOperator.LIKE:
        return typeof fieldValue === 'string' && fieldValue.includes(String(rule.value));

      case FilterOperator.STARTS_WITH:
        return typeof fieldValue === 'string' && fieldValue.startsWith(String(rule.value));

      case FilterOperator.ENDS_WITH:
        return typeof fieldValue === 'string' && fieldValue.endsWith(String(rule.value));

      case FilterOperator.IS_NULL:
        return fieldValue === null || fieldValue === undefined;

      case FilterOperator.IS_NOT_NULL:
        return fieldValue !== null && fieldValue !== undefined;

      default:
        return true;
    }
  }

  // ============ 数据加载 ============

  /**
   * @brief 加载行级权限配置 - 已实现实际 API 调用
   * @param role_name 可选的角色名称，用于加载特定角色的规则
   */
  function loadRowPermissions(role_name?: string): void {
    loading.value = true;

      // 审计修复(C1): 移除硬编码 /api 前缀, 由 alova baseURL('/api') 统一拼接
      httpClient.get(`/admin/roles/${encodeURIComponent(role_name || 'default')}/data-permissions`, { params: role_name ? { role_name } : {} })
      .then((response) => {
        const respData = response as { list?: RowPermission[]; data?: RowPermission[] | { list?: RowPermission[]; data?: RowPermission[] } };
        const dataObj = respData.data as { list?: RowPermission[]; data?: RowPermission[] } | undefined;
        rowPermissions.value = respData.list || (Array.isArray(respData.data) ? (respData.data) : dataObj?.list || dataObj?.data) || [];
        logger.info('【行级权限】加载成功', { count: rowPermissions.value.length });
      })
      .catch((error: unknown) => {
        logger.error('【行级权限】加载失败', error);
        rowPermissions.value = [];
      })
      .finally(() => {
        loading.value = false;
      });
  }

  /**
   * @brief 保存行级权限配置 - 已实现实际 API 调用
   * @param role_name 可选的角色名称
   */
  function saveRowPermissions(permissions: RowPermission[], role_name?: string): Promise<void> {
    // 审计修复(C1): 移除硬编码 /api 前缀, 由 alova baseURL('/api') 统一拼接
    return httpClient.put(`/admin/roles/${encodeURIComponent(role_name || 'default')}/data-permissions`, { permissions })
      .then(() => {
        rowPermissions.value = permissions;
        logger.info('【行级权限】保存成功');
        return Promise.resolve();
      })
      .catch((error: unknown) => {
        logger.error('【行级权限】保存失败', error);
        throw error;
      });
  }

  /**
   * @brief 获取用户自定义的行级权限 - 已实现实际 API 调用
   */
  function loadUserCustomPermissions(user_id: number, entity_type: string): Promise<unknown> {
    return httpClient.get('/permissions/row/custom', {
      params: { user_id, entity_type },
    })
      .then((response: { data?: unknown }) => {
        logger.info('【行级权限】加载用户自定义权限成功');
        return response.data;
      })
      .catch((error: unknown) => {
        logger.error('【行级权限】加载用户自定义权限失败', error);
        return null;
      });
  }

  return {
    // 状态
    rowPermissions,
    loading,

    // 计算属性
    applicableRowPermissions,

    // 权限检查
    checkRowPermission,
    filterAccessibleData,
    getApiFilter,

    // 数据加载
    loadRowPermissions,
    saveRowPermissions,
    loadUserCustomPermissions,

    // 内部方法（暴露用于测试）
    evaluateFilterGroup,
    evaluateRule,
  };
}

/**
 * @brief 行级权限指令值解析
 */
export function parseDynamicValue(value: string, context: Record<string, unknown>): unknown {
  // 支持 ${user.xxx} 格式的动态值
  if (typeof value === 'string' && value.startsWith('${') && value.endsWith('}')) {
    const path = value.slice(2, -1).split('.');
    let result: unknown = context;

    for (const key of path) {
      if (result && typeof result === 'object' && key in result) {
        result = (result as Record<string, unknown>)[key];
      } else {
        return value; // 无法解析，返回原始值
      }
    }

    return result;
  }

  return value;
}