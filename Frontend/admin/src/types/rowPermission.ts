/**
 * @file rowPermission.ts
 * @description 行级权限类型定义
 * @date 2026-04-04
 */

export enum FilterOperator {
  EQ = 'eq',
  NE = 'ne',
  GT = 'gt',
  GTE = 'gte',
  LT = 'lt',
  LTE = 'lte',
  IN = 'in',
  NOT_IN = 'not_in',
  LIKE = 'like',
  STARTS_WITH = 'starts_with',
  ENDS_WITH = 'ends_with',
  IS_NULL = 'is_null',
  IS_NOT_NULL = 'is_not_null',
  BETWEEN = 'between',
}

export enum LogicalOperator {
  AND = 'and',
  OR = 'or',
}

export type FilterValue = string | number | boolean | string[] | number[] | null | undefined;

export interface FilterRule {
  field: string;
  operator: FilterOperator;
  value: FilterValue;
  value2?: FilterValue;
  logic?: LogicalOperator;
}

export interface FilterGroup {
  rules: FilterRule[];
  logic: LogicalOperator;
  groups?: FilterGroup[];
}

export interface RowPermission {
  id: number;
  entity_type: string;
  permission: string;
  name: string;
  filterGroup: FilterGroup;
  allowCustomize: boolean;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface UserRowPermission {
  id: number;
  user_id: number;
  role_name: string;
  rowPermissionId: number;
  custom_filter_group?: FilterGroup;
  enabled: boolean;
  created_at: string;
}

export interface RowPermissionCheckResult {
  allowed: boolean;
  applied_filter?: FilterGroup;
  reason?: string;
}

export const OPERATOR_OPTIONS: Array<{ label: string; value: FilterOperator }> = [
  { label: '等于', value: FilterOperator.EQ },
  { label: '不等于', value: FilterOperator.NE },
  { label: '大于', value: FilterOperator.GT },
  { label: '大于等于', value: FilterOperator.GTE },
  { label: '小于', value: FilterOperator.LT },
  { label: '小于等于', value: FilterOperator.LTE },
  { label: '包含', value: FilterOperator.IN },
  { label: '不包含', value: FilterOperator.NOT_IN },
  { label: '模糊匹配', value: FilterOperator.LIKE },
  { label: '开始于', value: FilterOperator.STARTS_WITH },
  { label: '结束于', value: FilterOperator.ENDS_WITH },
  { label: '为空', value: FilterOperator.IS_NULL },
  { label: '不为空', value: FilterOperator.IS_NOT_NULL },
  { label: '在范围内', value: FilterOperator.BETWEEN },
];

export const OPERATORS_REQUIRING_VALUE: FilterOperator[] = [
  FilterOperator.EQ, FilterOperator.NE, FilterOperator.GT, FilterOperator.GTE,
  FilterOperator.LT, FilterOperator.LTE, FilterOperator.IN, FilterOperator.NOT_IN,
  FilterOperator.LIKE, FilterOperator.STARTS_WITH, FilterOperator.ENDS_WITH, FilterOperator.BETWEEN,
];

export function filterGroupToApiFormat(filterGroup: FilterGroup): Record<string, unknown> {
  const conditions: Record<string, unknown>[] = [];

  filterGroup.rules.forEach((rule, index) => {
    let condition: Record<string, unknown> = {};
    switch (rule.operator) {
      case FilterOperator.EQ: condition = { [rule.field]: rule.value }; break;
      case FilterOperator.NE: condition = { [rule.field]: { $ne: rule.value } }; break;
      case FilterOperator.GT: condition = { [rule.field]: { $gt: rule.value } }; break;
      case FilterOperator.GTE: condition = { [rule.field]: { $gte: rule.value } }; break;
      case FilterOperator.LT: condition = { [rule.field]: { $lt: rule.value } }; break;
      case FilterOperator.LTE: condition = { [rule.field]: { $lte: rule.value } }; break;
      case FilterOperator.IN: condition = { [rule.field]: { $in: rule.value } }; break;
      case FilterOperator.NOT_IN: condition = { [rule.field]: { $nin: rule.value } }; break;
      case FilterOperator.LIKE: condition = { [rule.field]: { $like: `%${String(rule.value)}%` } }; break;
      case FilterOperator.STARTS_WITH: condition = { [rule.field]: { $like: `${String(rule.value)}%` } }; break;
      case FilterOperator.ENDS_WITH: condition = { [rule.field]: { $like: `%${String(rule.value)}` } }; break;
      case FilterOperator.IS_NULL: condition = { [rule.field]: null }; break;
      case FilterOperator.IS_NOT_NULL: condition = { [rule.field]: { $ne: null } }; break;
      case FilterOperator.BETWEEN: condition = { [rule.field]: { $gte: rule.value, $lte: rule.value2 } }; break;
    }

    if (index > 0 && rule.logic) {
      const lastCondition = conditions.pop();
      if (lastCondition) {
        const logicalOp = rule.logic === LogicalOperator.AND ? '$and' : '$or';
        conditions.push({ [logicalOp]: [lastCondition, condition] });
      }
    } else {
      conditions.push(condition);
    }
  });

  if (conditions.length === 0) return {};
  if (conditions.length === 1) return conditions[0] ?? {};
  return { $and: conditions };
}

export function createEmptyFilterGroup(): FilterGroup {
  return { rules: [], logic: LogicalOperator.AND };
}

export function createEmptyFilterRule(): FilterRule {
  return { field: '', operator: FilterOperator.EQ, value: '', logic: LogicalOperator.AND };
}
