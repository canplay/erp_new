/**
 * @file config-service.ts
 * @brief 系统配置服务（config-service）
 * @date 2026-06-18
 * @description 对接 Backend/Rust config-service
 */

import { httpClient } from '@/utils/alova';
import type { HttpResponse } from '@/utils/alova';

// ============ 类型定义 ============

/**
 * @brief 系统配置项
 */
export interface SystemConfig {
  id: number;
  category: string;
  key: string;
  value: string;
  type: 'string' | 'number' | 'boolean' | 'json';
  label: string;
  description?: string;
  sort: number;
  status: number;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 配置分类
 */
export interface ConfigCategory {
  name: string;
  label: string;
  icon: string;
  description?: string;
}

/**
 * @brief 批量更新配置参数
 */
export interface BatchUpdateConfigParams {
  configs: Array<{ key: string; value: string }>;
}

// ============ API 函数 ============

/**
 * @brief 获取所有系统配置（按分类）
 * @description GET /api/config/system-configs
 */
export function getSystemConfigs(): Promise<HttpResponse<SystemConfig[]>> {
  return httpClient.get<SystemConfig[]>('/config/system-configs')
}

/**
 * @brief 获取单个配置
 * @description GET /api/config/system-configs/:key
 */
export function getSystemConfig(key: string): Promise<HttpResponse<SystemConfig>> {
  return httpClient.get<SystemConfig>(`/config/system-configs/${key}`)
}

/**
 * @brief 更新配置
 * @description PUT /api/config/system-configs/:key
 */
export function updateSystemConfig(
  key: string,
  value: string
): Promise<HttpResponse<void>> {
  return httpClient.put<void>(`/config/system-configs/${key}`, { value })
}

/**
 * @brief 批量更新配置
 * @description PUT /api/config/system-configs/batch
 */
export function batchUpdateSystemConfigs(
  params: BatchUpdateConfigParams
): Promise<HttpResponse<void>> {
  return httpClient.put<void>('/config/system-configs/batch', params)
}

/**
 * @brief 重置配置到默认值
 * @description POST /api/config/system-configs/:key/reset
 */
export function resetSystemConfig(key: string): Promise<HttpResponse<void>> {
  return httpClient.post<void>(`/config/system-configs/${key}/reset`, {})
}

/**
 * @brief 获取配置分类列表
 * @description 返回本地静态配置分类
 */
export function getConfigCategories(): ConfigCategory[] {
  return [
    { name: 'basic', label: '基础设置', icon: 'settings', description: '站点基础信息配置' },
    { name: 'security', label: '安全设置', icon: 'security', description: '密码策略、登录限制等' },
    { name: 'email', label: '邮件配置', icon: 'email', description: '邮件服务器和模板设置' },
    { name: 'upload', label: '上传配置', icon: 'cloud_upload', description: '文件上传相关设置' },
    { name: 'api', label: 'API配置', icon: 'api', description: '第三方API配置' },
  ];
}

// ============ 导出 ============

export const configService = {
  getSystemConfigs,
  getSystemConfig,
  updateSystemConfig,
  batchUpdateSystemConfigs,
  resetSystemConfig,
  getConfigCategories,
};
