/**
 * @file alova.d.ts
 * @description Alova 请求响应类型声明
 * @date 2026-05-18
 * 
 * 为 alova 请求的响应提供类型定义
 */

import type { Method, Alova as AlovaInstance } from 'alova';

/**
 * @brief Alova 响应类型
 */
export interface AlovaResponse<T = unknown> {
  /** 请求方法 */
  method: Method;
  /** 响应数据 */
  data?: {
    code?: number;
    message?: string;
    data?: T;
    success?: boolean;
    [key: string]: unknown;
  };
  /** 状态码 */
  status: number;
  /** 状态文本 */
  statusText: string;
  /** 响应头 */
  headers: Record<string, string>;
  /** 请求配置 */
  config: Record<string, unknown>;
}

/**
 * @brief 扩展 Window 类型
 */
declare global {
  interface Window {
    /**
     * @brief 导出方法用于开发调试
     */
    $api: AlovaInstance;
  }
}

export {};