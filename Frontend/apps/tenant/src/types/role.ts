/**
 * @file role.ts
 * @description 角色相关类型定义
 * @date 2026-04-06
 */

/**
 * @brief 角色详情
 */
export interface RoleDetail {
  /** 角色名称 */
  name: string;
  /** 角色描述 */
  description?: string;
  /** 角色类型 */
  type: RoleType;
  /** 状态 */
  status: RoleStatus;
  /** 用户数量 */
  user_count: number;
  /** 权限列表 */
  permissions: string[];
  /** 是否为预定义角色（不可删除） */
  is_predefined: boolean;
  /** 创建时间 */
  created_at: string;
  /** 创建人 */
  created_by?: string;
  /** 更新时间 */
  updated_at?: string;
}

/**
 * @brief 角色模板
 */
export interface RoleTemplate {
  /** 模板ID */
  id: number;
  /** 模板名称 */
  name: string;
  /** 模板描述 */
  description?: string;
  /** 包含的权限 */
  permissions: string[];
  /** 是否为内置模板 */
  isBuiltIn: boolean;
  /** 适用于哪些角色类型 */
  applicable_roles?: RoleType[];
  /** 创建时间 */
  created_at?: string;
}

/**
 * @brief 角色及其用户列表
 */
export interface RoleWithUsers {
  /** 角色信息 */
  role: RoleDetail;
  /** 该角色的用户列表 */
  users: RoleUserSummary[];
}

/**
 * @brief 角色用户摘要
 */
export interface RoleUserSummary {
  /** 用户ID */
  id: number;
  /** 用户名 */
  username: string;
  /** 头像 */
  avatar?: string;
  /** 邮箱 */
  email?: string;
  /** 部门 */
  department?: string;
  /** 职位 */
  position?: string;
}

/**
 * @brief 角色类型
 */
export enum RoleType {
  /** 普通用户 */
  USER = 'user',
  /** 管理员 */
  ADMIN = 'admin',
  /** VIP用户 */
  VIP = 'vip',
}

/**
 * @brief 角色状态
 */
export enum RoleStatus {
  /** 禁用 */
  DISABLED = 0,
  /** 启用 */
  ENABLED = 1,
}

/**
 * @brief 角色创建表单
 */
export interface RoleCreateForm {
  /** 角色名称 */
  name: string;
  /** 角色描述 */
  description?: string;
  /** 角色类型 */
  type: RoleType;
}

/**
 * @brief 角色更新表单
 */
export interface RoleUpdateForm {
  /** 角色描述 */
  description?: string;
  /** 角色状态 */
  status?: RoleStatus;
  /** 权限列表 */
  permissions?: string[];
}

/**
 * @brief 角色查询参数
 */
export interface RoleQueryParams {
  /** 关键词搜索 */
  keyword?: string;
  /** 角色类型 */
  type?: RoleType;
  /** 角色状态 */
  status?: RoleStatus;
  /** 页码 */
  page?: number;
  /** 每页数量 */
  page_size?: number;
}

/**
 * @brief 角色列表响应
 */
export interface RoleListResponse {
  /** 角色名称列表 */
  roles: string[];
  /** 预定义角色列表 */
  predefined_roles: string[];
}

/**
 * @brief 角色权限统计
 */
export interface RolePermissionStats {
  /** 角色名称 */
  role_name: string;
  /** 功能权限数量 */
  functionPermissionCount: number;
  /** 数据权限数量 */
  dataPermissionCount: number;
  /** 字段权限数量 */
  fieldPermissionCount: number;
  /** 继承的权限数量 */
  inheritedPermissionCount: number;
}

/**
 * @brief 角色复制选项
 */
export interface RoleCopyOptions {
  /** 源角色 */
  sourceRole: string;
  /** 目标角色列表 */
  targetRoles: string[];
  /** 是否包含功能权限 */
  includeFunctionPermission: boolean;
  /** 是否包含数据权限 */
  includeDataPermission: boolean;
  /** 是否包含字段权限 */
  includeFieldPermission: boolean;
  /** 是否包含继承权限 */
  includeInheritPermission: boolean;
}

/**
 * @brief 角色模板应用结果
 */
export interface RoleTemplateApplyResult {
  /** 应用是否成功 */
  success: boolean;
  /** 角色名称 */
  role_name: string;
  /** 新增的权限数量 */
  addedPermissions: number;
  /** 移除的权限数量 */
  removedPermissions: number;
  /** 错误信息（如果有） */
  error?: string;
}
