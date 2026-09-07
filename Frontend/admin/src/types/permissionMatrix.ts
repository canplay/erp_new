/**
 * @file permissionMatrix.ts
 * @description 权限矩阵类型定义
 * @date 2026-04-04
 */

/**
 * @brief 权限矩阵单元格
 */
export interface PermissionMatrixCell {
  /** 角色名称 */
  role: string;
  /** 权限标识 */
  permission: string;
  /** 是否有权限 */
  hasPermission: boolean;
  /** 权限来源（继承/直接分配） */
  source?: 'direct' | 'inherited' | 'denied';
}

/**
 * @brief 权限矩阵配置
 */
export interface PermissionMatrixConfig {
  /** 矩阵ID */
  id: string;
  /** 配置名称 */
  name: string;
  /** 角色列表 */
  roles: string[];
  /** 权限列表 */
  permissions: PermissionMatrixPermission[];
  /** 矩阵数据 */
  matrix: Record<string, Record<string, boolean>>;
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 权限矩阵权限项
 */
export interface PermissionMatrixPermission {
  /** 权限标识 */
  key: string;
  /** 权限名称 */
  name: string;
  /** 所属模块 */
  module: string | undefined;
  /** 类别 */
  category?: 'page' | 'button' | 'field' | 'data';
  /** 描述 */
  description?: string;
}

/**
 * @brief 权限模拟配置
 */
export interface PermissionSimulationConfig {
  /** 模拟类型 */
  type: 'role' | 'user' | 'custom';
  /** 角色名称（type=role时） */
  role_name?: string;
  /** 用户ID（type=user时） */
  user_id?: number;
  /** 自定义权限列表（type=custom时） */
  customPermissions?: string[];
  /** 模拟的场景描述 */
  description?: string;
}

/**
 * @brief 权限模拟结果
 */
export interface PermissionSimulationResult {
  /** 模拟配置 */
  config: PermissionSimulationConfig;
  /** 总权限数 */
  totalPermissions: number;
  /** 有权限数 */
  grantedPermissions: number;
  /** 无权限数 */
  deniedPermissions: number;
  /** 权限列表 */
  permissions: PermissionSimulationItem[];
  /** 模拟时间 */
  simulatedAt: string;
}

/**
 * @brief 权限模拟项
 */
export interface PermissionSimulationItem {
  /** 权限标识 */
  key: string;
  /** 权限名称 */
  name: string;
  /** 所属模块 */
  module: string;
  /** 是否有权限 */
  hasPermission: boolean;
  /** 权限来源 */
  source?: 'direct' | 'inherited' | 'role' | 'denied';
}

/**
 * @brief 权限变更请求
 */
export interface PermissionChangeRequest {
  /** 请求ID */
  id: string;
  /** 申请人 */
  applicant: {
    id: number;
    name: string;
  };
  /** 审批人 */
  approver?: {
    id: number;
    name: string;
  };
  /** 目标角色/用户 */
  target: {
    type: 'role' | 'user';
    id: number;
    name: string;
  };
  /** 变更类型 */
  changeType: 'add' | 'remove' | 'modify';
  /** 变更的权限 */
  permissions: Array<{
    key: string;
    name: string;
    before?: boolean;
    after: boolean;
  }>;
  /** 变更原因 */
  reason: string;
  /** 状态 */
  status: 'pending' | 'approved' | 'rejected';
  /** 创建时间 */
  created_at: string;
  /** 审批时间 */
  approved_at?: string;
}

/**
 * @brief 批量权限操作
 */
export interface BatchPermissionOperation {
  /** 操作类型 */
  type: 'add' | 'remove' | 'replace';
  /** 目标类型 */
  targetType: 'role' | 'user';
  /** 目标ID列表 */
  target_ids: number[];
  /** 权限列表 */
  permissions: string[];
  /** 原因 */
  reason?: string;
}
