/**
 * @file system.ts
 * @description 系统配置 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ 类型定义 ============

/**
 * @brief 系统配置接口
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

// ============ API 函数 ============

/**
 * @brief 获取所有系统配置（按分类）
 */
export async function getSystemConfigs() {
  try {
    return await httpClient.get('/config/system-configs');
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 获取配置分类列表
 * @description 返回本地静态配置分类，如需从 API 获取可修改为 async 函数
 */
export async function getConfigCategories(): ConfigCategory[] {
  return [
    { name: 'basic', label: '基础设置', icon: 'settings', description: '站点基础信息配置' },
    { name: 'security', label: '安全设置', icon: 'security', description: '密码策略、登录限制等' },
    { name: 'email', label: '邮件配置', icon: 'email', description: '邮件服务器和模板设置' },
    { name: 'upload', label: '上传配置', icon: 'cloud_upload', description: '文件上传相关设置' },
    { name: 'api', label: 'API配置', icon: 'api', description: '第三方API配置' },
  ];
}

/**
 * @brief 获取单个配置
 */
export async function getConfig(key: string) {
  try {
    return await httpClient.get(`/config/system-configs/${key}`);
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 更新配置
 */
export async function updateConfig(key: string, value: string) {
  try {
    return await httpClient.put(`/config/system-configs/${key}`, { value });
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 批量更新配置
 */
export async function batchUpdateConfigs(configs: Array<{ key: string; value: string }>) {
  try {
    return await httpClient.put('/config/system-configs/batch', { configs });
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 重置配置到默认值
 */
export async function resetConfig(key: string) {
  try {
    return await httpClient.post(`/config/system-configs/${key}/reset`, {});
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

// ============ 公告管理 ============

/**
 * @brief 公告接口
 */
export interface Announcement {
  id: number;
  title: string;
  content: string;
  type: 'info' | 'warning' | 'success' | 'error';
  priority: number;
  isPinned: boolean;
  isActive: boolean;
  start_time?: string;
  end_time?: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 获取公告列表
 */
export async function listAnnouncements(params?: {
  page?: number;
  page_size?: number;
  isActive?: boolean;
}) {
  try {
    return await httpClient.get('/admin/announcements', { params });
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 获取单个公告
 */
export async function getAnnouncement(id: number) {
  try {
    return await httpClient.get(`/admin/announcements/${id}`);
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 创建公告
 */
export async function createAnnouncement(data: {
  title: string;
  content: string;
  type?: 'info' | 'warning' | 'success' | 'error';
  priority?: number;
  isPinned?: boolean;
  isActive?: boolean;
  start_time?: string;
  end_time?: string;
}) {
  try {
    return await httpClient.post('/admin/announcements', data);
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 更新公告
 */
export async function updateAnnouncement(
  id: number,
  data: {
    title?: string;
    content?: string;
    type?: 'info' | 'warning' | 'success' | 'error';
    priority?: number;
    isPinned?: boolean;
    isActive?: boolean;
    start_time?: string;
    end_time?: string;
  }
) {
  try {
    return await httpClient.put(`/admin/announcements/${id}`, data);
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 删除公告
 */
export async function deleteAnnouncement(id: number) {
  try {
    return await httpClient.delete(`/admin/announcements/${id}`);
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}

/**
 * @brief 获取启用的公告（供前端展示）
 */
export async function getActiveAnnouncements() {
  try {
    return await httpClient.get('/announcements/active');
  } catch (error) {
    handleApiError(error, '系统配置');
    throw error;
  }
}