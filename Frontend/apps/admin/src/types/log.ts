/**
 * @file log.ts
 * @description 日志相关类型定义
 * @date 2026-04-03
 */

/**
 * @brief 操作类型
 */
export type OperationType = 'create' | 'update' | 'delete' | 'query' | 'export' | 'login' | 'logout';

/**
 * @brief 操作日志
 */
export interface OperationLog {
  id: number;
  user_id: number;
  username: string;
  realName?: string;
  module: string;
  action: string;
  operation_type: OperationType;
  method: string;
  url: string;
  requestMethod: string;
  requestUrl: string;
  request_params?: Record<string, unknown>;
  request_body?: Record<string, unknown>;
  response_data?: Record<string, unknown>;
  responseCode: number;
  params?: Record<string, unknown>;
  result?: string;
  ip: string;
  location?: string;
  user_agent?: string;
  duration: number;
  status: number;
  success: boolean;
  error_message?: string;
  created_at: string;
  /** 操作者（与 username 相同） */
  operator?: string;
  /** 资源 */
  resource?: string;
  /** 响应结果 */
  response_result?: string;
}

/**
 * @brief 操作日志查询参数
 */
export interface OperationLogParams {
  page?: number;
  page_size?: number;
  keyword?: string;
  username?: string;
  module?: string;
  action?: string;
  operation_type?: OperationType;
  start_date?: string;
  end_date?: string;
  status?: number;
  user_id?: number;
  success?: boolean;
}

/**
 * @brief 审计日志
 */
export interface AuditLog {
  id: number;
  user_id: number;
  username: string;
  realName?: string;
  resource_type: string;
  resourceId: string;
  resource_name?: string;
  action: string;
  beforeData?: Record<string, unknown>;
  afterData?: Record<string, unknown>;
  reason?: string;
  approval_status?: 'pending' | 'approved' | 'rejected';
  approver?: string;
  approved_at?: string;
  created_at: string;
}

/**
 * @brief 审计日志查询参数
 */
export interface AuditLogParams {
  page?: number;
  page_size?: number;
  keyword?: string;
  username?: string;
  resource_type?: string;
  action?: string;
  approval_status?: string;
  start_date?: string;
  end_date?: string;
}

/**
 * @brief 日志统计
 */
export interface LogStatistics {
  todayOperations: number;
  todayLogins: number;
  todayFailedLogins: number;
  weekOperationTrend: Array<{ date: string; count: number }>;
  topOperations: Array<{ action: string; count: number }>;
  topUsers: Array<{ username: string; count: number }>;
}

/**
 * @brief 操作统计参数
 */
export interface OperationStatsParams {
  start_date?: string;
  end_date?: string;
  groupBy?: 'day' | 'module' | 'user';
}

/**
 * @brief 登录日志
 */
export interface LoginLog {
  id: number;
  user_id: number;
  username: string;
  loginType: string;
  provider?: string;
  ip: string;
  location?: string;
  success: boolean;
  /** 状态（1=成功，0=失败） */
  status?: number;
  /** 错误信息 */
  error_msg?: string;
  /** 失败原因 */
  fail_reason?: string;
  user_agent?: string;
  /** 登录时间 */
  login_time?: number;
  created_at: string;
}

/**
 * @brief 登录日志查询参数
 */
export interface LoginLogParams {
  page?: number;
  page_size?: number;
  keyword?: string;
  start_date?: string;
  end_date?: string;
  success?: boolean;
  username?: string;
}
