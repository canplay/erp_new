/**
 * @file user.ts
 * @description 用户相关类型定义
 * @date 2026-04-03
 * @note LoginLog 类型已统一到 types/log.ts 中，此文件仅保留用户相关类型
 */

// ============ 重新导出 log.ts 中的登录日志类型（向后兼容）============
export type { LoginLog, LoginLogParams } from './log';

// ============ 部门相关类型 ============

/**
 * @brief 部门信息
 */
export interface Department {
  /** 部门ID */
  id: number;
  /** 部门名称 */
  name: string;
  /** 父部门ID */
  parent_id?: number;
  /** 子部门 */
  children?: Department[];
  /** 部门编码 */
  code?: string;
  /** 排序 */
  sort?: number;
  /** 排序顺序 */
  sort_order?: number;
  /** 状态 */
  status?: number;
  /** 负责人ID */
  leader_id?: number;
  /** 负责人名称 */
  leader_name?: string;
  /** 描述 */
  description?: string;
  /** 用户数量 */
  user_count?: number;
}

/**
 * @brief 部门查询参数
 */
export interface DepartmentQueryParams {
  /** 关键词搜索 */
  keyword?: string;
  /** 父部门ID */
  parent_id?: number;
  /** 状态 */
  status?: number;
}

/**
 * @brief 创建部门请求参数（与 department.ts 保持一致）
 */
export interface DepartmentCreateParams {
  name: string;
  parent_id?: number;
  leader_id?: number;
  description?: string;
  sort_order?: number;
}

/**
 * @brief 更新部门请求参数（与 department.ts 保持一致）
 */
export interface DepartmentUpdateParams {
  name?: string;
  parent_id?: number;
  leader_id?: number;
  description?: string;
  sort_order?: number;
}

// ============ 用户扩展类型 ============

/**
 * @brief 用户扩展信息（包含部门和职位）
 */
export interface UserExtend extends User {
  /** 部门ID */
  department_id?: number;
  /** 部门名称 */
  department_name?: string;
  /** 职位 */
  position?: string;
  /** 创建人ID */
  created_by?: number;
  /** 创建人名称 */
  created_by_name?: string;
  /** 更新人ID */
  updated_by?: number;
  /** 更新人名称 */
  updated_by_name?: string;
  /** 用户标签 */
  tags?: string[];
}

// ============ 用户查询参数 ============

/**
 * @brief 用户查询参数
 */
export interface UserQueryParams {
  /** 页码 */
  page?: number;
  /** 每页数量 */
  page_size?: number;
  /** 关键词搜索（用户名/邮箱/手机号） */
  keyword?: string;
  /** 用户状态 */
  status?: UserStatus | '';
  /** 角色 */
  role?: string;
  /** 部门ID */
  department_id?: number;
  /** 创建开始时间 */
  created_at_start?: string;
  /** 创建结束时间 */
  created_at_end?: string;
}

// ============ 用户创建/更新表单 ============

/**
 * @brief 用户创建表单
 */
export interface UserCreateForm {
  /** 用户名 */
  username: string;
  /** 密码 */
  password?: string;
  /** 昵称 */
  nickname?: string;
  /** 邮箱 */
  email?: string;
  /** 手机号 */
  phone?: string;
  /** 角色 */
  role: string;
  /** 部门ID */
  department_id?: number;
  /** 职位 */
  position?: string;
}

/**
 * @brief 用户更新表单
 */
export interface UserUpdateForm {
  /** 邮箱 */
  email?: string;
  /** 手机号 */
  phone?: string;
  /** 角色 */
  role?: string;
  /** 部门ID */
  department_id?: number;
  /** 职位 */
  position?: string;
  /** 状态 */
  status?: UserStatus;
}

// ============ 批量操作类型 ============

/**
 * @brief 批量更新角色参数
 */
export interface BatchUpdateRoleParams {
  /** 用户ID列表 */
  user_ids: number[];
  /** 新的角色 */
  role: string;
}

/**
 * @brief 批量更新状态参数
 */
export interface BatchUpdateStatusParams {
  /** 用户ID列表 */
  user_ids: number[];
  /** 新的状态 */
  status: UserStatus;
}

// ============ 用户导入导出 ============

/**
 * @brief 用户导入模板
 */
export interface UserImportTemplate {
  /** Excel列定义 */
  columns: {
    /** 列名 */
    name: string;
    /** 对应字段 */
    field: keyof UserCreateForm;
    /** 是否必填 */
    required: boolean;
    /** 示例值 */
    example?: string;
  }[];
  /** 模板说明 */
  instructions?: string;
}

/**
 * @brief 用户导入结果
 */
export interface UserImportResult {
  /** 总数 */
  total: number;
  /** 成功数 */
  success: number;
  /** 失败数 */
  failed: number;
  /** 错误详情 */
  errors: {
    /** 行号 */
    row: number;
    /** 错误信息 */
    message: string;
  }[];
}

/**
 * @brief 用户信息
 */
export interface UserInfo {
  id: number;
  username: string;
  email?: string;
  phone?: string;
  avatar?: string;
  role: string;
  status: number;
  created_at?: string;
  last_login_at?: string;
  permissions?: string[];
}

/**
 * @brief 登录表单
 */
export interface LoginForm {
  username: string;
  password: string;
  remember_me?: boolean;
}

/**
 * @brief 用户
 */
export interface User {
  id: number;
  username: string;
  nickname?: string;
  email?: string;
  phone?: string;
  avatar?: string;
  role: string;
  status: number;
  created_at: string;
  last_login_at?: string;
  last_login_ip?: string;
  gender?: number;
  /** 部门ID */
  department_id?: number;
  /** 部门名称 */
  department_name?: string;
  /** 用户数 */
  user_count?: number;
  /** 负责人名称 */
  leader_name?: string;
  /** 负责人ID */
  leader_id?: number;
  /** 描述 */
  description?: string;
  /** 排序 */
  sort_order?: number;
}

/**
 * @brief 角色状态
 */
export enum RoleStatus {
  DISABLED = 0,
  ENABLED = 1,
}

/**
 * @brief 预定义角色
 */
export enum PredefinedRole {
  USER = 'user',
  ADMIN = 'admin',
  VIP = 'vip',
}

/**
 * @brief 角色
 */
export interface Role {
  name: string;
  description?: string;
  permissions?: string[];
  user_count?: number;
  status?: number;
  created_at?: string;
  /**
   * @brief 角色类型 (admin/user/vip/system)
   */
  type?: string;
  /**
   * @brief 是否为预定义角色（不可删除）
   */
  is_predefined?: boolean;
}

/**
 * @brief 角色列表响应
 */
export interface RoleListResponse {
  roles: string[];
  predefined_roles: string[];
}

/**
 * @brief 用户状态枚举（与后端对应）
 */
export enum UserStatus {
  DISABLED = 0,
  ENABLED = 1,
  LOCKED = 2,
}

/**
 * @brief 系统统计
 */
export interface Statistics {
  total_users: number;
  active_users: number;
  new_users_today: number;
  login_attempts: number;
  failed_logins: number;
}
