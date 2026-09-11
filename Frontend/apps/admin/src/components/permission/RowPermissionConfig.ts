/**
 * @file RowPermissionConfig.ts
 * @description 行级权限配置 - 实体字段定义和选项
 * @date 2026-04-04
 */

import { FilterOperator, LogicalOperator } from '@/types/rowPermission';

export const EntityTypeOptions = [
  { label: '用户', value: 'user' },
  { label: '客户', value: 'customer' },
  { label: '订单', value: 'order' },
  { label: '文档', value: 'document' },
];

export const RoleOptions = [
  { label: '管理员', value: 'admin' },
  { label: '普通用户', value: 'user' },
  { label: '访客', value: 'guest' },
];

export const entityFields: Record<string, Array<{ field: string; label: string; type: string; enumOptions?: Array<{ label: string; value: unknown }> }>> = {
  user: [
    { field: 'department_id', label: '部门ID', type: 'number' },
    { field: 'department_name', label: '部门名称', type: 'string' },
    { field: 'creatorId', label: '创建者ID', type: 'number' },
    { field: 'creatorName', label: '创建者姓名', type: 'string' },
    { field: 'status', label: '状态', type: 'string', enumOptions: [{ label: '启用', value: 1 }, { label: '禁用', value: 0 }] },
    { field: 'role', label: '角色', type: 'string' },
  ],
  customer: [
    { field: 'customerType', label: '客户类型', type: 'string', enumOptions: [{ label: '个人', value: 'personal' }, { label: '企业', value: 'enterprise' }] },
    { field: 'level', label: '客户等级', type: 'string', enumOptions: [{ label: 'VIP', value: 'vip' }, { label: '普通', value: 'normal' }] },
    { field: 'region', label: '地区', type: 'string' },
    { field: 'assignedTo', label: '负责人ID', type: 'number' },
  ],
  order: [
    { field: 'status', label: '订单状态', type: 'string', enumOptions: [{ label: '待支付', value: 'pending' }, { label: '已支付', value: 'paid' }, { label: '已完成', value: 'completed' }, { label: '已取消', value: 'cancelled' }] },
    { field: 'amount', label: '订单金额', type: 'number' },
    { field: 'paymentMethod', label: '支付方式', type: 'string' },
  ],
  document: [
    { field: 'category', label: '文档分类', type: 'string' },
    { field: 'accessLevel', label: '访问级别', type: 'string', enumOptions: [{ label: '公开', value: 'public' }, { label: '内部', value: 'internal' }, { label: '机密', value: 'confidential' }] },
    { field: 'department_id', label: '部门ID', type: 'number' },
  ],
};

export const defaultPermissions = [
  {
    id: 1, name: '仅看本部门用户', permission: 'user:view', entity_type: 'user',
    filterGroup: { rules: [{ field: 'department_id', operator: FilterOperator.EQ, value: '${user.department_id}' }], logic: LogicalOperator.AND },
    enabled: true, allowCustomize: false,
    created_at: '2024-01-01T00:00:00Z', updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: 2, name: '仅看本人客户', permission: 'customer:view', entity_type: 'customer',
    filterGroup: { rules: [{ field: 'assignedTo', operator: FilterOperator.EQ, value: '${user.id}' }], logic: LogicalOperator.AND },
    enabled: true, allowCustomize: true,
    created_at: '2024-01-01T00:00:00Z', updated_at: '2024-01-01T00:00:00Z',
  },
];
