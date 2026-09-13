/**
 * @file types/api.ts
 * @description 统一的 API 响应类型定义
 * @date 2026-08-22
 */

/**
 * 通用 API 响应接口
 * @template T - 响应数据的具体类型
 */
export interface ApiResponse<T = unknown> {
  /** 状态码，0 表示成功 */
  code: number;
  /** 消息 */
  message: string;
  /** 响应数据 */
  data: T;
}

/**
 * 分页响应接口
 * @template T - 列表项的具体类型
 */
export interface PaginatedResponse<T = unknown> {
  /** 列表数据 */
  list: T[];
  /** 总记录数 */
  total: number;
  /** 当前页码 */
  page?: number;
  /** 每页条数 */
  page_size?: number;
  /** 总页数 */
  pages?: number;
}

/**
 * 分页请求参数
 */
export interface PaginationParams {
  /** 页码（从 1 开始） */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 排序字段 */
  sortBy?: string;
  /** 是否降序 */
  descending?: boolean;
}

/**
 * HTTP 响应类型 - 用于 alova 响应转换
 */
export interface HttpResponse<T = unknown> {
  code: number;
  message: string;
  success: boolean;
  data: T;
  list?: unknown[];
  total?: number;
  [key: string]: unknown;
}

/**
 * 请求配置类型
 */
export interface RequestConfig {
  headers?: Record<string, string>;
  params?: Record<string, unknown>;
  timeout?: number;
  [key: string]: unknown;
}

/**
 * 错误响应
 */
export interface ApiError {
  code: number;
  message: string;
  details?: Record<string, string[]>;
}

/**
 * 通用列表响应（兼容旧版 API）
 * @deprecated 使用 ApiResponse<T> 或 PaginatedResponse<T> 替代
 */
export interface LegacyListResponse<T = unknown> {
  list: T[];
  total: number;
}

/**
 * 通用空响应（用于删除、更新等操作）
 */
export interface EmptyResponse {
  code: number;
  message: string;
  data: null;
}
